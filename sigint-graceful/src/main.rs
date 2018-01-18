extern crate libc;
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate crossbeam;

use chan_signal::Signal;

/// Makes some noise when dropped.
struct Bomb {
    label: String,
}

impl Bomb {
    fn new(label: String) -> Bomb {
        Bomb {
            label
        }
    }
}

impl Drop for Bomb {
    fn drop(&mut self) {
        println!("Boom! from {}", self.label);
    }
}

fn main() {
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[
        Signal::TERM, // kill, without "-9"
        Signal::INT, // Ctrl+C
        Signal::QUIT, // Ctrl+\
        Signal::ALRM, // libc::alarm
    ]);

    let bomb = Bomb::new("main".to_owned());

    crossbeam::scope(|scope| {
        // When our work is complete, send a sentinel value on `sdone`.
        let (sdone, rdone) = chan::sync(0);

        unsafe {
            libc::alarm(5);
        }

        // Run work.
        scope.spawn(move || {
            let bomb = Bomb::new("run".to_owned());
            run(sdone)
        });

        scope.spawn(move || {
            let bomb = Bomb::new("chan_select".to_owned());
            loop {
                // Wait for a signal or for work to be done.
                chan_select! {
                    signal.recv() -> signal => {
                        println!("Received signal: {:?}", signal)
                    },
                    rdone.recv() => {
                        println!("Program completed normally.");
                        break;
                    }
                }
            }
        });
    });
}

fn run(_sdone: chan::Sender<()>) {
    // Do some work.
    ::std::thread::sleep(std::time::Duration::new(8, 0));
    // Quit normally.
    // Note that we don't need to send any values. We just let the sending channel drop, which
    // closes the channel, which causes the receiver to synchronize immediately and always.
    println!("Thread exiting!");
}
