#[macro_use]
extern crate chan;
extern crate chan_signal;

use std::io::{self, stdin, BufRead};
use std::thread;

use chan_signal::Signal;

mod bomb;
use bomb::Bomb;

fn spawn_stdin_chan() -> chan::Receiver<io::Result<String>> {
    let (s, r) = chan::sync(0);
    thread::spawn(move || {
        let stdin = stdin();
        for line in stdin.lock().lines() {
            s.send(line);
        }
    });
    r
}

fn run(signal: chan::Receiver<Signal>) -> io::Result<()> {
    let lines = spawn_stdin_chan();
    loop {
        chan_select! {
            signal.recv() -> signal => {
                eprintln!("Received signal: {:?}", signal);
                break;
            },

            lines.recv() -> line => match line {
                Some(line) => {
                    println!("Line length = {}", line?.len());
                }
                None => break,
            }
        }
    }
    Ok(())
}

fn main() {
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[
        Signal::TERM, // kill, without "-9"
        Signal::INT, // Ctrl+C
        Signal::QUIT, // Ctrl+\
        Signal::ALRM, // libc::alarm
    ]);

    let _bomb = Bomb::new("main".to_owned());

    // Run work.
    let handler = thread::spawn(move || {
        let _bomb = Bomb::new("run".to_owned());
        run(signal)
    });
    let _ = handler.join().unwrap();
    eprintln!("Program completed normally.");
}
