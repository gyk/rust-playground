use std::io::Write;
use std::thread;
use std::net::TcpStream;
use std::time::{Instant, Duration};

use byteorder::{ByteOrder, BigEndian};

struct Inspector {
    start_time: Instant,
    bytes_sent: usize,
}

impl Inspector {
    pub fn new(now: Instant) -> Inspector {
        Inspector {
            start_time: now,
            bytes_sent: 0,
        }
    }

    pub fn check(&mut self, message: &[u8]) {
        self.bytes_sent += message.len();
    }
}

impl Drop for Inspector {
	fn drop(&mut self) {
        let duration_sec: f32 = {
            let elapsed = self.start_time.elapsed();
            elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1_000_000_000_f32
        };

        // byte -> KBit per second
        const BYTE_TO_KBPS: f32 = 8_f32 / 1000_f32;

        println!("# bytes sent = {} during {} seconds, bit rate = {} kbps",
            self.bytes_sent,
            duration_sec,
            self.bytes_sent as f32 * BYTE_TO_KBPS / duration_sec);
    }
}

// Keep sending 10KB packets at an interval of 50ms (bit rate = 1600kbps):
//
//     cargo run --release --example publisher -- 10000 50

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    let n_bytes_per_packet = args[1].parse::<usize>().unwrap();
    let sending_interval = args[2].parse::<u64>().unwrap();

    let mut payload = vec![0; n_bytes_per_packet + 4];
    BigEndian::write_u32(&mut payload, n_bytes_per_packet as u32);

    let iter_interval = Duration::from_millis(sending_interval);

    let join_handle = thread::spawn(move || {
        let mut stream = TcpStream::connect("0.0.0.0:9090").unwrap();
        let start_time = Instant::now();
        let mut inspector = Inspector::new(start_time);
        let mut iter_end_time = Duration::from_millis(0);

        loop {
            iter_end_time += iter_interval;
            if let Err(..) =  stream.write_all(&payload[..]) {
                eprintln!("Write error!");
                break;
            }
            inspector.check(&payload[..]);

            match iter_end_time.checked_sub(start_time.elapsed()) {
                Some(wait_time) => thread::sleep(wait_time),
                None => println!("Lagging"),
            }
        }
    });
    join_handle.join().unwrap();
}
