//! Evaluates Tokio's performance of broadcasting length-prefix packets.

extern crate futures;
extern crate bytes;
extern crate byteorder;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_timer;

mod codec;

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
use tokio::net::TcpListener;
use tokio::runtime::current_thread::Runtime;
use tokio_codec::Decoder;
use tokio_timer::Interval;

use codec::LengthPrefixCodec;

fn listen_to_publisher(tx: mpsc::Sender<BytesMut>) {
    thread::spawn(move || {
        let addr = "0.0.0.0:9090".parse::<SocketAddr>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let listener = TcpListener::bind(&addr).unwrap();
        println!("Listening to publisher on: {}", addr);

        let tx = Rc::new(RefCell::new(Some(tx)));

        let done = listener.incoming()
            .map_err(|_io_err| ())
            .take(1)
            .for_each(move |socket| {
                let framed = LengthPrefixCodec.framed(socket);

                // Clone `tx` because it has to be moved to the inner closure. Since we only take
                // one of the incoming connections the cloning actually happens once.
                let tx_before = Rc::clone(&tx);
                let (_to_publisher, from_publisher) = framed.split();

                from_publisher
                    .map_err(|_io_err| ())
                    .for_each(move |message| {
                        let tx_after = Rc::clone(&tx_before);

                        tx_before
                            .borrow_mut()
                            .take()
                            .expect("tx has already been taken")
                            .send(message.clone())
                            .map(move |tx| {
                                *tx_after.borrow_mut() = Some(tx);
                            })
                            .map_err(|_| ())
                    })
                    .map(|_| ())
                    .map_err(|_| ())
            });

        rt.block_on(done).unwrap();
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

    let mut rt = Runtime::new().unwrap();
    let handle = rt.handle();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let subscribers = Rc::new(RefCell::new(
        Vec::<Rc<RefCell<
            Option<mpsc::Sender<BytesMut>>
        >>>::with_capacity(64)
    ));
    let (main_tx, main_rx) = mpsc::channel(8);

    // Sets up the publisher
    if use_external_publisher {
        listen_to_publisher(main_tx);
    } else {
        let n_bytes_per_packet = args[1].parse::<usize>().unwrap();
        let sending_interval = args[2].parse::<u64>().unwrap();

        let payload = BytesMut::from(vec![0; n_bytes_per_packet]);
        rt.spawn(Interval::new_interval(Duration::from_millis(sending_interval))
            .map_err(|_| ())
            .for_each(move |_instant| {
                main_tx
                    .clone()
                    .send(payload.clone())
                    .map(|_| ())
                    .map_err(|_| ())
            })
            .map_err(|_| ())
        );
    }

    let subscribers2 = Rc::clone(&subscribers);
    rt.spawn(main_rx.for_each(move |buf: BytesMut| {
        let subscribers = subscribers2.borrow();
        let all_sendings = subscribers.iter().map(|tx| {
            let tx_before = {
                let tx2 = Rc::clone(tx);
                let tx_before = tx2
                    .borrow_mut()
                    .take()
                    .expect("tx has already been taken");
                tx_before
            };
            let tx_after = Rc::clone(tx);

            tx_before
                .send(buf.clone())
                .map(move |tx| {
                    *tx_after.borrow_mut() = Some(tx);
                })
        });
        stream::futures_unordered(all_sendings).then(|_| Ok(())).for_each(|()| Ok(()))
    }));

    let done = socket.incoming().for_each(move |socket| {
        let framed = LengthPrefixCodec.framed(socket);

        let (to_subscriber, _from_subscriber) = framed.split();

        let (tx, rx) = mpsc::channel(4);
        let tx = Rc::new(RefCell::new(Some(tx)));
        subscribers.borrow_mut().push(tx);

        let write_to_subscriber = rx
            .forward(to_subscriber.sink_map_err(|_io_err| ()))
            .map(|_| ())
            .map_err(|_| ());
        let _ = handle.spawn(write_to_subscriber);

        Ok(())
    });

    rt.block_on(done).unwrap();
}
