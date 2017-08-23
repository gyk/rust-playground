extern crate futures;
#[macro_use]
extern crate lazy_static;

extern crate bytes;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_pool;

use std::sync::Mutex;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use futures::{Stream, Future};

use bytes::BytesMut;
use tokio_core::net::TcpStream;
use tokio_io::codec::length_delimited;
use tokio_pool::TokioPool;

#[derive(Default)]
struct Statistics {
    n_connections: usize,
    message_counts: Vec<usize>,
    byte_counts: Vec<usize>,
}

lazy_static! {
    static ref STATISTICS: Mutex<Statistics> = Mutex::new(Statistics::default());
}

#[derive(Default)]
struct Inspector {
    message_count: usize,
    byte_count: usize,
}

impl Inspector {
    pub fn check(&mut self, message: &BytesMut) {
        self.message_count += 1;
        self.byte_count += message.len();
    }
}

impl Drop for Inspector {
	fn drop(&mut self) {
        let mut stat = STATISTICS.lock().unwrap();
        stat.message_counts.push(self.message_count);
        stat.byte_counts.push(self.byte_count);

        if stat.message_counts.len() == stat.n_connections {
            let total_message_count: usize = stat.message_counts.iter().sum();
            let total_byte_count: usize = stat.byte_counts.iter().sum();

            println!("\n# connections = {}, # messages = {}, # bytes = {}\n\
                # messages per connection = {}, # bytes per message = {}",
                stat.n_connections,
                total_message_count,
                total_byte_count,
                total_message_count as f32 / stat.n_connections as f32,
                total_byte_count as f32 / total_message_count as f32);
        }
    }
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    let n_threads = args[1].parse::<usize>().unwrap();
    let n_subscribers = args[2].parse::<usize>().unwrap();
    let n_subscribers_per_thread = (n_subscribers + n_threads - 1) / n_threads;

    let (pool, join) = TokioPool::new(n_threads).expect("Failed to create event loop");
    let addr: SocketAddr = "0.0.0.0:9000".parse().unwrap();

    let mut i_subscriber = 0;
    'outer: for i in 0..n_threads {
        println!("Starting thread #{}...", i);
        let worker = pool.next_worker();

        for j in 0..n_subscribers_per_thread {
            println!("    Spawning subscriber #{} ({}-{})...", i_subscriber, i, j);
            worker.spawn(move |handle| {
                TcpStream::connect(&addr, handle)
                    .and_then(move |socket| {
                        let mut inspector = Inspector::default();
                        let mut stat = STATISTICS.lock().unwrap();
                        stat.n_connections += 1;
                        if stat.n_connections == n_subscribers {
                            println!("All {} connections established", n_subscribers);
                        }

                        let framed: length_delimited::Framed<_, BytesMut> =
                            length_delimited::Framed::new(socket);
                        framed.for_each(move |message| {
                            inspector.check(&message);
                            Ok(())
                        })
                    })
                    .map_err(|_| ())
                }
            );

            i_subscriber += 1;
            if i_subscriber >= n_subscribers {
                break 'outer;
            }

            thread::sleep(Duration::from_millis(5));
        }
    }

    join.join();
}
