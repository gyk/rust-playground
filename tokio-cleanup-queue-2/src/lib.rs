use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::{
    future,
    sync::{mpsc, oneshot},

    Async,
    Future,
    Stream,
};
use lazy_static::lazy_static;
use tokio::runtime::{Runtime, Builder as RuntimeBuilder};
use tokio_timer::{DelayQueue, Interval};

lazy_static! {
    // FIXME: builder
    pub static ref CLEANUP_RUNTIME: Arc<Mutex<Runtime>> = Arc::new(Mutex::new(
        RuntimeBuilder::new()
            .core_threads(2)
            .build()
            .expect("Failed to create Tokio runtime for cleanup queue")));
}

const CLEANUP_INTERVAL: Duration = Duration::from_secs(1);

pub struct CleanupQueue<T> {
    sender: mpsc::UnboundedSender<T>,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl<T> CleanupQueue<T>
    where T: Send + 'static
{
    pub fn new<F>(cleanup_fn: F,
                  delay: Duration) -> Self
        where F: Fn(T) + Send + 'static
    {
        let (sender, receiver) = mpsc::unbounded();
        let (stop_tx, stop_rx) = oneshot::channel();

        let stop_tx = Some(stop_tx);
        let stop_rx_shared_channel = stop_rx.shared();
        let stop_rx_shared_interval = stop_rx_shared_channel.clone();

        let queue = Arc::new(Mutex::new(DelayQueue::new()));
        let queue_channel = Arc::clone(&queue);
        let queue_interval = Arc::clone(&queue);

        // Forwards tasks received from channel to delay queue
        CLEANUP_RUNTIME.lock().unwrap().spawn(stop_rx_shared_channel.select2(
            receiver.for_each(move |task| {
                queue_channel.lock().unwrap().insert(task, delay);
                future::ok(())
            })
        ).map(|_| ()).map_err(|_| ()));

        // Polls the delay queue regularly to cleans up expired tasks
        let cleanup_interval = Interval::new_interval(CLEANUP_INTERVAL);
        CLEANUP_RUNTIME.lock().unwrap().spawn(stop_rx_shared_interval.select2(
            cleanup_interval.for_each(move |_instant| {
                loop {
                    match queue_interval.lock().unwrap().poll() {
                        Ok(Async::Ready(Some(expired))) => {
                            cleanup_fn(expired.into_inner());
                        }

                        Ok(Async::Ready(None)) |
                        Ok(Async::NotReady) => break,

                        Err(e) => return Err(From::from(e)),
                    }
                }

                Ok(())
            })
        ).map(|_| ()).map_err(|_| ()));

        Self {
            sender,
            stop_tx,
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<T> {
        self.sender.clone()
    }
}

impl<T> Drop for CleanupQueue<T> {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
}
