extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

use std::io;
use std::thread;
use std::time::Duration;

use futures::{stream, Future, Sink, Stream};
use futures::sync::mpsc;
use tokio_core::reactor::{Core};
use tokio_timer::{Timer};


const CHAN_BUF_SIZE: usize = 8;
const SEND_DELAY_MS: u64 = 3000;
const TIMEOUT_MS: u64 = 1000;

fn main() {
    let mut core = Core::new().unwrap();
    let _handle = core.handle();

    let (tx, rx) = mpsc::channel(CHAN_BUF_SIZE);

    thread::spawn(move || {
        tx.clone().send(42).wait().expect("sender error");
        thread::sleep(Duration::from_millis(SEND_DELAY_MS));
        tx.send(100).wait().expect("sender error");
    });

    let timeout_stream = Timer::default().timeout_stream(
        rx.map_err(|_| io::Error::new(io::ErrorKind::Other, "receiver error")),
        Duration::from_millis(TIMEOUT_MS));
    let should_timeout = timeout_stream
        .for_each(|x| {
            println!("Received {:?}", x);
            futures::future::ok(())
        })
        .map_err(|err| {
            println!("Error {:?}", err);
            ()
        });

    core.run(should_timeout);
}
