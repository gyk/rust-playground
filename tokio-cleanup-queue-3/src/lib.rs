use std::mem;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::{
    sync::mpsc,
    try_ready,

    Async,
    Future,
    Poll,
    Stream,
};
use lazy_static::lazy_static;
use log::{error, info, trace};
use tokio::runtime::{Runtime, Builder as RuntimeBuilder};
use tokio_timer::Delay;

lazy_static! {
    pub static ref CLEANUP_RUNTIME: Arc<Mutex<Runtime>> = Arc::new(Mutex::new(
        RuntimeBuilder::new()
            .core_threads(2)
            .build()
            .expect("Failed to create Tokio runtime for cleanup queue")));
}

#[derive(Debug)]
enum QueueState<T> {
    /// The initial state.
    EmptyChannel,

    /// Has received a message from the channel, waiting it to expire.
    WaitToExpire {
        delay: Delay,
        task: T,
    },

    /// Executes remaining tasks with best effort, and then stops.
    GracefulStopping,

    /// Stops immediately.
    Stopped,
}

struct CleanupFuture<T, F> {
    state: QueueState<T>,
    cleanup_delay: Duration,
    cleanup_fn: F,
    receiver: mpsc::UnboundedReceiver<CleanupMessage<T>>,
}

impl<T, F> Future for CleanupFuture<T, F>
    where F: Fn(T) + Send + 'static
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.state = match self.state {
                QueueState::EmptyChannel => {
                    if let Some(msg) = try_ready!(self.receiver.poll()) {
                        match msg {
                            CleanupMessage::Task { task, time } => {
                                QueueState::WaitToExpire {
                                    delay: Delay::new(time + self.cleanup_delay),
                                    task,
                                }
                            }
                            CleanupMessage::GracefulStop => QueueState::GracefulStopping,
                            CleanupMessage::ForceStop => QueueState::Stopped,
                        }
                    } else {
                        error!("receiver none");
                        break;
                    }
                }

                QueueState::WaitToExpire { ref mut delay, .. } => {
                    let _ = try_ready!(delay.poll().map_err(|_| ()));

                    let old_state = mem::replace(&mut self.state, QueueState::EmptyChannel);
                    if let QueueState::WaitToExpire { task, .. } = old_state {
                        (self.cleanup_fn)(task);
                    } else {
                        unreachable!();
                    }

                    QueueState::EmptyChannel
                }

                QueueState::GracefulStopping => {
                    trace!("Graceful stopping...");
                    while let Some(CleanupMessage::Task { task, .. }) =
                        try_ready!(self.receiver.poll())
                    {
                        (self.cleanup_fn)(task);
                    }

                    QueueState::Stopped
                }

                QueueState::Stopped => {
                    break;
                }
            };
        }
        Ok(Async::Ready(()))
    }
}

#[derive(Debug)]
pub struct CleanupQueue<T> {
    sender: mpsc::UnboundedSender<CleanupMessage<T>>,
}

#[derive(Debug)]
pub enum CleanupMessage<T> {
    Task {
        task: T,
        /// Enqueue time
        time: Instant,
    },
    GracefulStop,
    ForceStop,
}

impl<T> CleanupMessage<T> {
    pub fn new_task(task: T) -> Self {
        CleanupMessage::Task {
            task,
            time: Instant::now(),
        }
    }
}

impl<T> CleanupQueue<T>
    where T: Send + 'static
{
    pub fn new<F>(cleanup_fn: F,
                  cleanup_delay: Duration) -> Self
        where F: Fn(T) + Send + 'static
    {
        let (sender, receiver) = mpsc::unbounded();

        let fut = CleanupFuture {
            state: QueueState::EmptyChannel,
            cleanup_delay,
            cleanup_fn,
            receiver,
        };

        // Forwards tasks received from channel to delay queue
        CLEANUP_RUNTIME.lock().unwrap().spawn(
            fut.map_err(|_| ())
        );

        Self {
            sender,
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<CleanupMessage<T>> {
        self.sender.clone()
    }
}

impl<T> Drop for CleanupQueue<T> {
    fn drop(&mut self) {
        let _ = self.sender.unbounded_send(CleanupMessage::GracefulStop);
    }
}
