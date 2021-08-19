use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::hash::Hash;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct MessageMap<K, V> {
    pub data: Arc<Mutex<HashMap<K, (V, u64)>>>,
}

impl<K, V> MessageMap<K, V>
where
    K: Eq + Hash,
{
    pub fn clone(&self) -> Self {
        MessageMap {
            data: Arc::clone(&self.data),
        }
    }
}

pub struct Message<K, V> {
    pub key: K,
    pub value: V,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ScheduledEvent<K> {
    when: Instant,
    id: u64,
    key: K,
}

struct WorkerSharedState<K> {
    is_stopped: bool,
    queue: VecDeque<ScheduledEvent<K>>,
}

impl<K> Default for WorkerSharedState<K> {
    fn default() -> Self {
        WorkerSharedState {
            is_stopped: false,
            queue: VecDeque::new(),
        }
    }
}

fn lock_queue_and_map<'a, 'b, K, V>(
    state: &'a Arc<(Mutex<WorkerSharedState<K>>, Condvar)>,
    state_guard: MutexGuard<'a, WorkerSharedState<K>>,
    msg_map: &'b MessageMap<K, V>,
) -> (
    MutexGuard<'a, WorkerSharedState<K>>,
    MutexGuard<'b, HashMap<K, (V, u64)>>,
) {
    // Simple deadlock avoidance loop.
    let mut state_guard = Some(state_guard);
    loop {
        let state_guard = state_guard
            .take()
            .unwrap_or_else(|| state.0.lock().unwrap());

        // To avoid deadlock, we do a `try_lock`, and on `WouldBlock`, we unlock the
        // events Mutex, and retry after yielding.
        match msg_map.data.try_lock() {
            Ok(msg_map) => return (state_guard, msg_map),
            Err(::std::sync::TryLockError::Poisoned { .. }) => panic!("FIXME"),
            Err(::std::sync::TryLockError::WouldBlock) => {
                // Drop the lock before yielding to give other threads a chance to complete
                // their work.
                drop(state_guard);
                ::std::thread::yield_now();
            }
        }
    }
}

struct ScheduleWorker<K, V>
where
    K: Eq + Hash,
{
    state: Arc<(Mutex<WorkerSharedState<K>>, Condvar)>,
    tx: mpsc::Sender<Message<K, V>>,
    message_map: MessageMap<K, V>,
}

impl<K, V> ScheduleWorker<K, V>
where
    K: Eq + Hash,
{
    fn fire_due_events<'a>(
        &'a self,
        now: Instant,
        state: MutexGuard<'a, WorkerSharedState<K>>,
    ) -> (Option<Instant>, MutexGuard<'a, WorkerSharedState<K>>) {
        let (mut state, mut msg_map) = lock_queue_and_map(&self.state, state, &self.message_map);

        while let Some(event) = state.queue.pop_front() {
            if event.when <= now {
                self.fire_event(event, &mut msg_map)
            } else {
                // Not due yet, put it back.
                let next_when = event.when;
                state.queue.push_front(event);
                return (Some(next_when), state);
            }
        }
        (None, state)
    }

    fn fire_event(&self, ev: ScheduledEvent<K>, msg_map: &mut HashMap<K, (V, u64)>) {
        let ScheduledEvent { key, id, .. } = ev;
        match msg_map.entry(key) {
            Entry::Occupied(o) => {
                let (_, timer_id) = *o.get();
                if timer_id == id {
                    let (key, (value, _)) = o.remove_entry();
                    let _ = self.tx.send(Message { key, value });
                } else if id > timer_id {
                    o.remove();
                }
            }
            Entry::Vacant(_) => (), // report error?
        }
    }

    fn run(&mut self) {
        let mut state = self.state.0.lock().unwrap();
        loop {
            let now = Instant::now();
            let (next_when, state_out) = self.fire_due_events(now, state);
            state = state_out;

            if state.is_stopped {
                break;
            }

            state = if let Some(next_when) = next_when {
                // Wait for stop notification or timeout to send next event.
                self.state.1.wait_timeout(state, next_when - now).unwrap().0
            } else {
                // No pending events.
                //
                // Wait for new event, to check when it should be send and then wait to send it
                self.state.1.wait(state).unwrap()
            };
        }
    }
}

pub struct WatchTimer<K, V> {
    state: Arc<(Mutex<WorkerSharedState<K>>, Condvar)>,
    message_map: MessageMap<K, V>,
    counter: u64,
    delay: Duration,
}

impl<K, V> WatchTimer<K, V>
where
    K: Eq + Hash + Send + Clone + 'static,
    V: Send + 'static,
{
    pub fn new(
        tx: mpsc::Sender<Message<K, V>>,
        message_map: MessageMap<K, V>,
        delay: Duration,
    ) -> Self {
        let state = Arc::new((Mutex::new(WorkerSharedState::default()), Condvar::new()));

        let worker_state = Arc::clone(&state);
        let worker_msg_map = message_map.clone();
        std::thread::spawn(move || {
            ScheduleWorker {
                state: worker_state,
                tx,
                message_map: worker_msg_map,
            }
            .run();
        });

        WatchTimer {
            state,
            message_map,
            counter: 0,
            delay,
        }
    }

    pub fn schedule(&mut self, key: K, value: V) {
        self.counter = self.counter.wrapping_add(1);

        {
            let mut state = self.state.0.lock().unwrap();
            let (state_out, mut msg_map) =
                lock_queue_and_map(&self.state, state, &self.message_map);
            state = state_out;
            state.queue.push_back(ScheduledEvent {
                when: Instant::now() + self.delay,
                id: self.counter,
                key: key.clone(),
            });
            msg_map.insert(key, (value, self.counter));
        }
        self.state.1.notify_one();
    }

    pub fn ignore(&self, key: &K) {
        let mut msg_map = self.message_map.data.lock().unwrap();
        msg_map.remove(key);
    }
}

impl<K, V> Drop for WatchTimer<K, V> {
    fn drop(&mut self) {
        {
            let mut state = self.state.0.lock().unwrap();
            state.is_stopped = true;
        }
        self.state.1.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (tx, rx) = mpsc::channel();

        let msg_map = MessageMap::<String, String>::default();

        let mut timer = WatchTimer::new(tx, msg_map.clone(), Duration::from_secs(1));
        let start_time = Instant::now();
        timer.schedule("Cancel".into(), "cancel".into());
        timer.schedule("Delay".into(), "delay".into());
        timer.schedule("Hello".into(), "nihao".into());
        timer.schedule(" ".into(), "SPACE".into());
        timer.schedule("World".into(), "shijie".into());
        timer.schedule("!".into(), "EXCLAMATION".into());
        timer.ignore(&"Cancel".into());
        std::thread::sleep(Duration::from_millis(500));
        timer.ignore(&"Delay".into());
        timer.schedule("Delay".into(), "delay2".into());

        for msg in rx {
            println!(
                "{:?} - {:?} => {:?}",
                start_time.elapsed(),
                msg.key,
                msg.value
            );
        }
    }
}
