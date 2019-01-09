use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use futures::sync::{mpsc, oneshot};
use futures::{future, Future};

use tokio_cleanup_queue_2::CleanupQueue;

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

fn rotate_files(root: &str) {
    let cleanup_queue = CleanupQueue::new(remove_file, CLEANUP_DELAY);
    let sender = cleanup_queue.sender();

    let mut file_list = VecDeque::new();
    for file_id in 0..10 {
        let file = File {
            path: format!("{}/{}", root, file_id),
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
}

// cargo run --example purge_file
fn main() {
    const ALICE_ROOT: &str = "/alice/files";
    const BOB_ROOT: &str = "/bob/files";

    let alice_join = thread::spawn(|| {
        rotate_files(ALICE_ROOT)
    });
    thread::sleep(ONE_SECOND * 2);
    let bob_join = thread::spawn(|| {
        rotate_files(BOB_ROOT)
    });

    alice_join.join();
    bob_join.join();

    thread::sleep(ONE_SECOND * 2);
}
