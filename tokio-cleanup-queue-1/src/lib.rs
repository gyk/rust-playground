use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::{
    future,
    sync::{mpsc, oneshot},

    Async,
    Future,
    Stream,
};
use tokio::runtime;
use tokio_timer::{DelayQueue, Interval};


const CLEANUP_INTERVAL: Duration = Duration::from_secs(1);

pub fn run<T, F>(cleanup_fn: F,
                 delay: Duration,
                 receiver: mpsc::UnboundedReceiver<T>,
                 stop_receiver: future::Shared<oneshot::Receiver<()>>)
    where T: 'static,
          F: Fn(T) + 'static + Copy
{
    let stop_rx_shared_channel = stop_receiver;
    let stop_rx_shared_interval = stop_rx_shared_channel.clone();

    let queue = Arc::new(Mutex::new(DelayQueue::new()));
    let queue_channel = Arc::clone(&queue);
    let queue_interval = Arc::clone(&queue);

    let cleanup_interval = Interval::new_interval(CLEANUP_INTERVAL);
    runtime::current_thread::spawn(stop_rx_shared_interval.select2(
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

    runtime::current_thread::spawn(stop_rx_shared_channel.select2(
        receiver.for_each(move |task| {
            queue_channel.lock().unwrap().insert(task, delay);
            future::ok(())
        })
    ).map(|_| ()).map_err(|_| ()));
}
