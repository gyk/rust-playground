use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use futures::sync::{mpsc, oneshot};
use futures::{future, Future};
use tokio::runtime::current_thread;

use tokio_cleanup_queue_1 as cleanup_queue;

/// A mock file
#[derive(Debug)]
pub struct File {
    pub path: String,
}

fn remove_file(file: File) {
    println!("(!) Shredding {:?}...", file);
}

const CLEANUP_DELAY: Duration = Duration::from_secs(3);
const ONE_SECOND: Duration = Duration::from_secs(1);

// cargo run --example purge_file
fn main() {
    const ROOT: &str = "/usr/files";

    let (sender, receiver) = mpsc::unbounded();
    let (stop_sender, stop_receiver) = oneshot::channel::<()>();
    let stop_receiver_shared = stop_receiver.shared();

    thread::spawn(move || {
        let mut rt = current_thread::Runtime::new().unwrap();

        rt.block_on(
            future::lazy(move || {
                let stop_rx_shared2 = stop_receiver_shared.clone();
                cleanup_queue::run(remove_file, CLEANUP_DELAY, receiver, stop_rx_shared2);
                stop_receiver_shared
            }).map(|_| ())
        ).unwrap();
    });

    let mut file_list = VecDeque::new();
    for file_id in 0..10 {
        let file = File {
            path: format!("{}/{}", ROOT, file_id),
        };
        println!("Adding {:?}", file);
        file_list.push_back(file);

        while file_list.len() > 5 {
            let old_file = file_list.pop_front().unwrap();
            println!("Sending {:?} to cleanup queue", old_file);
            sender.unbounded_send(old_file).unwrap();
        }

        thread::sleep(ONE_SECOND);
    }

    // Sends remaining files
    while let Some(file) = file_list.pop_front() {
        println!("Sending leftover {:?} to cleanup queue", file);
        sender.unbounded_send(file).unwrap();
    }

    thread::sleep(ONE_SECOND * 3);
    stop_sender.send(()).unwrap();
}
