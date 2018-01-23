extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

mod inspect;

use std::time::Duration;

use futures::stream;
use futures::sync::mpsc::{self, Sender};
use futures::{Future, Sink, Stream};
use tokio_core::reactor::Core;
use tokio_timer::Timer;

fn inpect_channel<T>(tx: &Sender<T>) {
    use std::mem;
    use std::sync::atomic::Ordering;

    let naked_sender = unsafe {
        mem::transmute::<&Sender<T>, &inspect::Sender<T>>(tx)
    };
    let maybe_parked = naked_sender.maybe_parked;
    let state = naked_sender
        .inner
        .state
        .load(Ordering::SeqCst);
    println!("sender.state = {:?}, maybe_parked = {:?}",
        inspect::decode_state(state), maybe_parked);
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let (tx, rx) = mpsc::channel::<usize>(8);

    let timer = Timer::default();
    let timer2 = timer.clone();
    let producer = stream::iter_ok(0..)
        .and_then(move |x| {
            timer
                .sleep(Duration::from_millis(1000))
                .map(move |()| x)
                .map_err(|_| ())
        })
        .for_each(move |x| {
            let x_clone = x;

            let tx2 = tx.clone();
            print!("[CLONE] ");
            inpect_channel(&tx2);

            tx2
                .send(x)
                .map(move |tx| {
                    print!("[SENT]  ");
                    inpect_channel(&tx);
                    println!("tx -> {}", x_clone);
                    ()
                })
                .map_err(|_| ())
        });

    handle.spawn(producer);

    let consumer = rx
        .for_each(move |x| {
            timer2
                // Sleeps longer than producing interval, so the channel will be gradually filled
                // with in-flight messages.
                .sleep(Duration::from_millis(2000))
                .map(move |()| {
                    println!("    rx <- {}\n", x);
                })
                .map_err(|_| ())
        });

    core.run(consumer).unwrap();
}
