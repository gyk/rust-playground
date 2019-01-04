//! TokioPool is a library designed to help you multithread your
//! tokio applications by providing a pool of threads which you
//! can distribute your load across.
//!

extern crate futures;
extern crate tokio;

use std::io;
use std::sync::{mpsc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

use futures::Future;
use futures::sync::oneshot;
use tokio::runtime::current_thread::{Handle, Runtime};

// Unlike `tokio_core::reactor::Remote` which is `Sync`, `tokio::runtime::current_thread::Handle` is
// `!Sync`. Maybe we should use `tokio::runtime::TaskExecutor`?

pub struct TokioPool {
    stop_tx: Option<oneshot::Sender<()>>,
    handles: Vec<Mutex<Handle>>,
    next_worker: AtomicUsize,
}

pub struct PoolJoin {
    joiners: Vec<JoinHandle<()>>,
}

// FIXME: poor error handling.

impl TokioPool {
    /// Creates a TokioPool with the given number of workers
    pub fn new(worker_count: usize) -> io::Result<(TokioPool, PoolJoin)> {
        assert!(worker_count != 0);
        let (tx, rx) = mpsc::channel::<io::Result<Mutex<Handle>>>();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let stop_rx_shared = stop_rx.shared();

        let mut joiners = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let tx = tx.clone();
            let stop_rx = stop_rx_shared.clone();

            let join = thread::spawn(move || {
                let mut runtime = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(err) => {
                        tx.send(Err(err)).expect("Channel was closed early");
                        return;
                    }
                };

                let handle = Mutex::new(runtime.handle());
                tx.send(Ok(handle)).expect("Channel was closed early");

                runtime.block_on(stop_rx).unwrap();
            });
            joiners.push(join);
        }

        let handles: io::Result<_> = rx.into_iter()
                                       .take(worker_count)
                                       .collect();

        let pool = TokioPool {
            stop_tx: Some(stop_tx),
            handles: handles?,
            next_worker: AtomicUsize::new(0),
        };
        let join = PoolJoin { joiners: joiners };
        Ok((pool, join))
    }

    pub fn next_worker(&self) -> MutexGuard<Handle> {
        let next = self.next_worker.fetch_add(1, Ordering::SeqCst);
        let idx = next % self.handles.len();
        self.handles[idx].lock().expect("Mutex poisoned")
    }

    /// Stops all of the worker threads
    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            stop_tx.send(()).expect("Stop signal sending error");
        }
    }
}

impl PoolJoin {
    /// Joins on the threads. Can only be called once.
    pub fn join(self) {
        for joiner in self.joiners {
            joiner.join().expect("Worker thread panic");
        }
    }
}
