//! A simple delay queue that is not as simple as `simple_delay_queue`.
#![allow(dead_code)]

use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;

use crate::delay_task::*;

const SLEEP_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct DelayQueue<T: Task> {
    delay: Duration,
    receiver: Arc<Mutex<mpsc::Receiver<DelayTask<T>>>>,
    state: Arc<RwLock<QueueState>>,
}

#[derive(PartialEq, Eq, Debug)]
enum QueueState {
    Running,
    // Executes remaining tasks with best effort, and then stops.
    GracefulStopping,
    // Stops immediately.
    ForceStopping,
    Stopped,
}

impl<T: Task + 'static> DelayQueue<T> {
    /// Constructs a new `DelayQueue`.
    ///
    /// # Parameters
    ///
    /// - `delay`: delayed time to remove tasks, in seconds.
    /// - `receiver`: the receiving side of an MPSC channel of `DelayTask`.
    pub fn new(delay: u32, receiver: mpsc::Receiver<DelayTask<T>>) -> Self {
        Self {
            delay: Duration::from_secs(u64::from(delay)),
            receiver: Arc::new(Mutex::new(receiver)),
            state: Arc::new(RwLock::new(QueueState::Stopped)),
        }
    }

    pub fn run(&mut self) {
        let delay = self.delay;
        let receiver = Arc::clone(&self.receiver);
        let state = Arc::clone(&self.state);

        // In case the delay queue has not been stopped from the last run
        if *state.read().unwrap() == QueueState::Running {
            *state.write().unwrap() = QueueState::ForceStopping;
        }
        // Waits the previous thread stops
        while *state.read().unwrap() != QueueState::Stopped {}

        *state.write().unwrap() = QueueState::Running;
        thread::spawn(move || {
            'outer: loop {
                match receiver.lock().unwrap().try_recv() {
                    Ok(dt) => {
                        let expire = dt.time + delay;
                        loop {
                            match *state.read().unwrap() {
                                QueueState::Running => {
                                    if Instant::now() > expire {
                                        break;
                                    }
                                    thread::sleep(SLEEP_DURATION);
                                }

                                QueueState::GracefulStopping => break,

                                QueueState::ForceStopping => break 'outer,

                                QueueState::Stopped => (), // unreachable
                            }
                        }
                        dt.task.execute();
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        if *state.read().unwrap() == QueueState::Running {
                            thread::sleep(SLEEP_DURATION);
                        } else {
                            break;
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }

            *state.write().unwrap() = QueueState::Stopped;
        });
    }

    pub fn stop(&mut self, wait: bool) {
        if wait {
            let state = Arc::clone(&self.state);
            thread::spawn(move || {
                // Sleeps for a while in a new thread, so the caller can send remaining tasks to the
                // channel.
                thread::sleep(SLEEP_DURATION);
                *state.write().unwrap() = QueueState::GracefulStopping;
            });
        } else {
            *self.state.write().unwrap() = QueueState::ForceStopping;
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mock_file::*;

    #[test]
    fn smoke_complex() {
        const DELAY: u32 = 3;
        const INTERVAL: Duration = Duration::from_millis(200);

        let (sender, receiver) = mpsc::channel();
        let mut delay_queue = DelayQueue::new(DELAY, receiver);
        delay_queue.run();

        thread::spawn(move || {
            for y in 1922.. {
                let f = File {
                    content: vec![0; 0],
                    path: format!("/ussr/kgb/{}.doc", y),
                };
                let t = DelayTask::new(f);
                sender.send(t).unwrap();
                thread::sleep(INTERVAL);
            }
        });

        thread::sleep(INTERVAL * (1991 - 1922));
        let wait = true;
        delay_queue.stop(wait);
        println!("Stop the queue.");
        thread::sleep(Duration::from_secs(DELAY as u64));

        println!("Done.");
    }
}
