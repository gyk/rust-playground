//! Evaluates Tokio's performance of broadcasting length-prefix packets.

extern crate futures;
extern crate bytes;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_timer;

use std::env;
use std::thread;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use bytes::BytesMut;
use futures::{Future, Sink};
use futures::stream::{self, Stream};
use futures::sync::mpsc;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::codec::length_delimited;
use tokio_timer::Timer;

fn listen_to_publisher(tx: mpsc::Sender<BytesMut>) {
    thread::spawn(move || {
        let addr = "0.0.0.0:9090".parse::<SocketAddr>().unwrap();
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let listener = TcpListener::bind(&addr, &handle).unwrap();
        println!("Listening to publisher on: {}", addr);

        let done = listener.incoming()
            .map_err(|_io_err| ())
            .take(1)
            .for_each(move |(socket, _addr)| {
                let framed: length_delimited::Framed<_, BytesMut> =
                    length_delimited::Framed::new(socket);

                // Clone `tx` because it has to be moved to the inner closure. Since we only take
                // one of the incoming connections the cloning actually happens once.
                let tx = tx.clone();
                let (_to_publisher, from_publisher) = framed.split();

                from_publisher
                    .map_err(|_io_err| ())
                    .for_each(move |message| {
                        tx
                            .clone()
                            .send(message.clone())
                            .map(|_| ())
                            .map_err(|_| ())
                    })
                    .map(|_| ())
                    .map_err(|_| ())
            });

        core.run(done).unwrap();
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

    let addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let subscribers = Rc::new(RefCell::new(Vec::<mpsc::Sender<BytesMut>>::with_capacity(1024)));
    let (main_tx, main_rx) = mpsc::channel(8);

    // Sets up the publisher
    if use_external_publisher {
        listen_to_publisher(main_tx);
    } else {
        // FIXME: buggy, may cause high CPU usage at small sending interval.
        let n_bytes_per_packet = args[1].parse::<usize>().unwrap();
        let sending_interval = args[2].parse::<u64>().unwrap();

        let payload = BytesMut::from(vec![0; n_bytes_per_packet]);
        let timer = Timer::default();
        handle.spawn(timer.interval(Duration::from_millis(sending_interval))
            .map_err(|_| ())
            .for_each(move |()| {
                main_tx
                    .clone()
                    .send(payload.clone())
                    .map(|_| ())
                    .map_err(|_| ())
            })
            .map_err(|_| ())
        );
    }

    let subscribers2 = subscribers.clone();
    handle.spawn(main_rx.for_each(move |buf: BytesMut| {
        let subscribers = subscribers2.borrow();
        let all_sendings = subscribers.iter().map(|tx| {
            tx.clone().send(buf.clone())
        });
        stream::futures_unordered(all_sendings).then(|_| Ok(())).for_each(|()| Ok(()))
    }));

    let done = socket.incoming().for_each(move |(socket, _addr)| {
        let framed: length_delimited::Framed<_, BytesMut> =
            length_delimited::Framed::new(socket);

        let (to_subscriber, _from_subscriber) = framed.split();

        let (tx, rx) = mpsc::channel(4);
        subscribers.borrow_mut().push(tx);

        let write_to_subscriber = rx
            .forward(to_subscriber.sink_map_err(|_io_err| ()))
            .map(|_| ())
            .map_err(|_| ());
        handle.spawn(write_to_subscriber);

        Ok(())
    });

    core.run(done).unwrap();
}
