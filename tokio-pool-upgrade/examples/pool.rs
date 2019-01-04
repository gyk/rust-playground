extern crate futures;
extern crate tokio;
extern crate tokio_pool_upgrade;

use futures::Future;
use futures::Stream;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio_pool_upgrade::TokioPool;

fn main() {
    // Create a pool with 4 workers
    let (pool, join) = TokioPool::new(4).expect("Failed to create event loop");
    // Wrap it in an Arc to share it with the listener worker
    let pool = Arc::new(RwLock::new(pool));
    // We can listen on 8080
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

    // Clone the pool reference for the listener worker
    let pool_inner = Arc::clone(&pool);

    {
        // Use the first pool worker to listen for connections
        let pool_guard = pool.read().unwrap();
        pool_guard.next_worker().spawn({
            // Bind a TCP listener to our address
            let listener = TcpListener::bind(&addr).unwrap();

            // Listen for incoming clients
            listener.incoming().for_each(move |_socket| {
                let pool_guard_inner = pool_inner.read().unwrap();
                pool_guard_inner.next_worker().spawn({
                    // Do work with a client socket
                    futures::future::ok(())
                }).unwrap();

                Ok(())
            }).map_err(|err| {
                eprintln!("Error with TcpListener: {:?}", err);
            })
        }).unwrap();
    }

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(10));
        pool.write().unwrap().stop();
    });

    join.join();
}
