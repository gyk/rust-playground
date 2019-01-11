use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use log::{debug, info, warn};
use pretty_env_logger;
use tokio_cleanup_queue_3::{CleanupQueue, CleanupMessage};

/// A mock file
#[derive(Debug)]
pub struct File {
    pub path: String,
}

fn remove_file(file: File) {
    thread::sleep(Duration::from_millis(500));
    warn!("(!) Shredding {:?}...", file);
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
        debug!("Adding {:?}", file);
        file_list.push_back(file);

        while file_list.len() > 5 {
            let old_file = file_list.pop_front().unwrap();
            info!("Sending {:?} to cleanup queue", old_file);
            sender.unbounded_send(CleanupMessage::new_task(old_file)).unwrap();
        }

        thread::sleep(ONE_SECOND);
    }

    info!("Sending GracefulStop message to cleanup queue");
    sender.unbounded_send(CleanupMessage::GracefulStop).unwrap();

    // Sends remaining files
    while let Some(file) = file_list.pop_front() {
        info!("Sending leftover {:?} to cleanup queue", file);
        sender.unbounded_send(CleanupMessage::new_task(file)).unwrap();
    }
}

// RUST_LOG=purge_file=DEBUG,tokio_cleanup_queue=TRACE cargo run --example purge_file
fn main() {
    pretty_env_logger::init_timed();

    const ALICE_ROOT: &str = "/alice/files";
    const BOB_ROOT: &str = "/bob/files";

    let alice_join = thread::spawn(|| {
        rotate_files(ALICE_ROOT)
    });
    thread::sleep(ONE_SECOND * 2);
    let bob_join = thread::spawn(|| {
        rotate_files(BOB_ROOT)
    });


    alice_join.join().unwrap();
    bob_join.join().unwrap();
    info!("===== Done =====");

    thread::sleep(ONE_SECOND * 5);
    info!("===== Exit =====");
}
