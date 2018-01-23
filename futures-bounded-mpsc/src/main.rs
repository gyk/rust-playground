extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

mod inspect;

use std::env;
use std::mem;
use std::time::Duration;

use futures::stream;
use futures::sync::mpsc::{self, Sender};
use futures::{Future, Sink, Stream};
use tokio_core::reactor::Core;
use tokio_timer::Timer;

fn inpect_channel<T>(tx: &Sender<T>) {
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

fn custom_clone<T>(tx: &Sender<T>) -> Sender<T> {
    let naked_tx = unsafe {
        mem::transmute::<&Sender<T>, &inspect::Sender<T>>(&tx)
    };
    let sender_task = naked_tx.sender_task.clone();

    let mut tx2 = tx.clone();
    let naked_tx2 = unsafe {
        mem::transmute::<&mut Sender<T>, &mut inspect::Sender<T>>(&mut tx2)
    };
    naked_tx2.sender_task = sender_task;
    // Always sets `maybe_parked` to true so `sender_task` is locked every time. It degrades the
    // performance but is logically correct.
    naked_tx2.maybe_parked = true;
    tx2
}

fn main() {
    let use_custom_clone = match env::var("CUSTOM_CLONE") {
        Ok(_) => true,
        Err(_) => false,
    };

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

            let tx2;
            match use_custom_clone {
                true => {
                    tx2 = custom_clone(&tx);
                    print!("[!CLONED] ");
                }
                false => {
                    tx2 = tx.clone();
                    print!("[CLONED]  ");
                }
            }
            inpect_channel(&tx2);

            tx2
                .send(x)
                .map(move |tx| {
                    print!("[SENT]    ");
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
