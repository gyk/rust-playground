//! Evaluates Tokio's performance of broadcasting length-prefix packets.

mod codec;

use std::env;
use std::iter::FromIterator;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use bytes::BytesMut;
use futures::prelude::*;
use futures::channel::mpsc;
use futures::stream::FuturesUnordered;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio::sync::Mutex;
use tokio::time;
use tokio_util::codec::Decoder;

use codec::LengthPrefixCodec;

fn make_single_threaded_runtime() -> runtime::Runtime {
    runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .expect("Cannot build Runtime")
}

fn listen_to_publisher(mut tx: mpsc::Sender<BytesMut>) {
    // Running in another thread
    let _ = thread::spawn(move || {
        let mut rt = make_single_threaded_runtime();

        rt.block_on(async move {
            let addr = "0.0.0.0:9090".parse::<SocketAddr>().unwrap();
            let mut listener = TcpListener::bind(&addr).await?;
            println!("Listening to publisher on: {}", addr);

            let (socket, _addr) = listener.accept().await?;
            let framed = LengthPrefixCodec.framed(socket);

            let (_to_publisher, mut from_publisher) = framed.split();

            while let Some(message) = from_publisher.try_next().await? {
                tx.send(message).await.unwrap();
            }

            Ok::<(), tokio::io::Error>(())
        }).unwrap();
    });
}

// Send 1KB packet every 100 ms:
//
//     cargo run --release -- 1000 100
//
// Use external publisher:
//
//     cargo run --release
fn main() {
    let args: Vec<String> = env::args().collect();
    let use_external_publisher = args.len() == 1;

    let mut rt = make_single_threaded_runtime();

    let addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
    let mut socket = rt.block_on(TcpListener::bind(&addr)).unwrap();
    println!("Listening on: {}", addr);

    let subscribers = Arc::new(Mutex::new(
        Vec::<Arc<Mutex<
            mpsc::Sender<BytesMut>
        >>>::with_capacity(64)
    ));
    let (mut main_tx, mut main_rx) = mpsc::channel(8);

    // Sets up the publisher
    if use_external_publisher {
        listen_to_publisher(main_tx);
    } else {
        println!("Use internal publisher");
        let n_bytes_per_packet = args[1].parse::<usize>().unwrap();
        let sending_interval = args[2].parse::<u64>().unwrap();

        // NOTE: Bytes has changed its design in v0.5 and the original code (`impl From<Vec<u8>> for
        // BytesMut`) doesn't work any more. See https://github.com/tokio-rs/bytes/pull/298.
        let payload = BytesMut::from_iter(std::iter::repeat(0).take(n_bytes_per_packet));
        rt.spawn(async move {
            let mut timer = time::interval(Duration::from_millis(sending_interval));
            while let Some(_instant) = timer.next().await {
                main_tx.send(payload.clone()).await.unwrap();
            }
        });
    }

    let subscribers2 = Arc::clone(&subscribers);
    rt.spawn(async move {
        while let Some(buf) = main_rx.next().await {
            let subscribers = subscribers2.lock().await;
            let mut broadcast = FuturesUnordered::new();
            for sub in subscribers.iter() {
                let buf = buf.clone();
                broadcast.push(async move {
                    let mut tx = sub.lock().await;
                    tx.send(buf).await.unwrap();
                });
            }
            while let Some(_) = broadcast.next().await {}
        }
    });

    rt.block_on(async move {
        while let Some(socket) = socket.incoming().try_next().await? {
            let framed = LengthPrefixCodec.framed(socket);
            let (to_subscriber, _from_subscriber) = framed.split();

            let (tx, rx) = mpsc::channel::<BytesMut>(4);
            let tx = Arc::new(Mutex::new(tx));
            subscribers.lock().await.push(tx);

            let write_to_subscriber = rx
                .map(|x| Ok(x))
                .forward(to_subscriber.sink_map_err(|_io_err| ()))
                .map_err(|_| ());
            let _ = tokio::spawn(write_to_subscriber);
        }
        Ok::<(), tokio::io::Error>(())
    }).unwrap();
}
