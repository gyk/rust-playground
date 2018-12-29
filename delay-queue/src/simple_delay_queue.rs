//! A simple delay queue.
#![allow(dead_code)]

use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread::{self, JoinHandle};

use crate::delay_task::*;

const SLEEP_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct DelayHandle(JoinHandle<()>);
impl DelayHandle {
    fn join(self) -> thread::Result<()> {
        self.0.join()
    }
}

/// Runs the delay queue.
///
/// # Parameters
///
/// - `delay`: delayed time to execute tasks, in seconds.
/// - `receiver`: the receiving side of an MPSC channel of `DelayTask`.
pub fn run<T>(delay: u32, receiver: mpsc::Receiver<DelayTask<T>>) -> DelayHandle
    where T: Task + 'static
{
    let join = thread::spawn(move || {
        let delay = Duration::from_secs(u64::from(delay));
        // Blocks on the receiver, but it's OK since we don't need to cancel the queue.
        for dt in receiver.iter() {
            let expire = dt.time + delay;
            loop {
                if Instant::now() > expire {
                    break;
                } else {
                    thread::sleep(SLEEP_DURATION);
                }
            }
            dt.task.execute();
        }
    });

    DelayHandle(join)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mock_file::*;

    #[test]
    fn smoke_simple() {
        const DELAY: u32 = 3;

        let handle = {
            let (sender, receiver) = mpsc::channel();
            let handle = crate::simple_delay_queue::run(DELAY, receiver);

            let f1 = File {
                content: Vec::from(&b"Hello World!"[..]),
                path: "/ussr/kgb/welcome.txt".to_owned(),
            };
            let t1 = DelayTask::new(f1);
            sender.send(t1).unwrap();

            thread::sleep(Duration::from_secs(DELAY as u64));

            let f2 = File {
                content: Vec::from(&b"Bomb the World!"[..]),
                path: "/ussr/kgb/knowledge-base.txt".to_owned(),
            };
            let t2 = DelayTask::new(f2);
            sender.send(t2).unwrap();

            handle
        };

        handle.join().unwrap();
        println!("Done.");
    }
}
