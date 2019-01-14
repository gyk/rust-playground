use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::{
    sync::{mpsc, oneshot},
    try_ready,

    Async,
    Future,
    Poll,
    Stream,
};
use lazy_static::lazy_static;
use log::{error, trace};
use tokio::runtime::{Runtime, Builder as RuntimeBuilder};
use tokio_timer::Delay;

lazy_static! {
    pub static ref CLEANUP_RUNTIME: Arc<Runtime> = Arc::new(
        RuntimeBuilder::new()
            .core_threads(2)
            .build()
            .expect("Failed to create Tokio runtime for cleanup queue"));
}

#[derive(Debug)]
pub struct CleanupTask<T> {
    pub task: T,
    /// Enqueue time
    pub time: Instant,
}

impl<T> CleanupTask<T> {
    pub fn new(task: T) -> Self {
        CleanupTask {
            task,
            time: Instant::now(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StopMessage {
    Graceful,
    Force,
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

impl<T> QueueState<T> {
    fn is_running(&self) -> bool {
        match *self {
            QueueState::EmptyChannel |
            QueueState::WaitToExpire { .. } => true,

            QueueState::GracefulStopping |
            QueueState::Stopped => false,
        }
    }
}

struct CleanupFuture<T, F> {
    state: QueueState<T>,
    cleanup_delay: Duration,
    cleanup_fn: F,
    receiver: mpsc::UnboundedReceiver<CleanupTask<T>>,
    stop_rx: oneshot::Receiver<StopMessage>,
}

impl<T, F> Future for CleanupFuture<T, F>
    where F: Fn(T) + Send + 'static
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if self.state.is_running() {
                // Handles stop message first.
                match self.stop_rx.poll() {
                    Ok(Async::Ready(StopMessage::Graceful)) => {
                        let old_state = mem::replace(&mut self.state, QueueState::GracefulStopping);
                        if let QueueState::WaitToExpire { task, .. } = old_state {
                            (self.cleanup_fn)(task);
                        }
                    }
                    Ok(Async::Ready(StopMessage::Force)) => {
                        self.state = QueueState::Stopped;
                    }
                    Ok(Async::NotReady) => (),
                    Err(..) => break,
                }
            }

            self.state = match self.state {
                QueueState::EmptyChannel => {
                    match try_ready!(self.receiver.poll()) {
                        Some(CleanupTask { task, time }) => {
                            QueueState::WaitToExpire {
                                delay: Delay::new(time + self.cleanup_delay),
                                task,
                            }
                        }
                        None => {
                            error!("receiver none");
                            break;
                        }
                    }
                }

                QueueState::WaitToExpire { ref mut delay, .. } => {
                    let _ = try_ready!(delay.poll().map_err(|_| ()));

                    let old_state = mem::replace(&mut self.state, QueueState::EmptyChannel);
                    if let QueueState::WaitToExpire { task, .. } = old_state {
                        (self.cleanup_fn)(task);
                    }

                    continue;
                }

                QueueState::GracefulStopping => {
                    trace!("Graceful stopping...");
                    while let Some(CleanupTask { task, .. }) = try_ready!(self.receiver.poll()) {
                        (self.cleanup_fn)(task);
                    }

                    QueueState::Stopped
                }

                QueueState::Stopped => {
                    break;
                }
            };
        }
        trace!("CleanupFuture resolved");
        Ok(Async::Ready(()))
    }
}

#[derive(Debug)]
pub struct CleanupQueue<T> {
    sender: mpsc::UnboundedSender<CleanupTask<T>>,
    stop_tx: Option<oneshot::Sender<StopMessage>>,
}

impl<T> CleanupQueue<T>
    where T: Send + 'static
{
    pub fn new<F>(cleanup_fn: F,
                  cleanup_delay: Duration) -> Self
        where F: Fn(T) + Send + 'static
    {
        let (sender, receiver) = mpsc::unbounded();
        let (stop_tx, stop_rx) = oneshot::channel();
        let stop_tx = Some(stop_tx);

        let fut = CleanupFuture {
            state: QueueState::EmptyChannel,
            cleanup_delay,
            cleanup_fn,
            receiver,
            stop_rx,
        };

        CLEANUP_RUNTIME.executor().spawn(fut);

        Self {
            sender,
            stop_tx,
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<CleanupTask<T>> {
        self.sender.clone()
    }
}

impl<T> CleanupQueue<T> {
    pub fn stop(&mut self, stop_message: StopMessage) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(stop_message);
        }
    }
}

impl<T> Drop for CleanupQueue<T>{
    fn drop(&mut self) {
        self.stop(StopMessage::Graceful);
    }
}
