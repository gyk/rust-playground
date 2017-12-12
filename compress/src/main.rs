extern crate flate2;
extern crate itertools;

use std::io::prelude::*;
use std::io::{stdin, Result};
use std::sync::mpsc;
use std::thread;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use itertools::Itertools;


fn compress<I>(lines: I, batch_size: usize, tx: mpsc::Sender<Vec<u8>>) -> Result<()>
    where I: IntoIterator<Item = String>,
{
    let write_buffer = Vec::with_capacity(8192);
    for batch in lines.into_iter().chunks(batch_size).into_iter() {
        let mut encoder = DeflateEncoder::new(write_buffer.clone(), Compression::default());
        let mut input_size = 0;

        for line in batch {
            let line_bytes = line.as_bytes();
            input_size += line_bytes.len();
            encoder.write(line_bytes)?;
            encoder.write(b"\n")?;
        }

        let write_buffer = encoder.finish()?;
        println!("Compress: {} -> {}", input_size, write_buffer.len());
        tx.send(write_buffer).unwrap();
    }

    Ok(())
}

fn decompress(rx: mpsc::Receiver<Vec<u8>>) -> Box<Iterator<Item = String>> {
    Box::new(rx
        .into_iter()
        .flat_map(move |packet| {
            let rd = &mut &packet[..];
            let mut read_buffer = String::with_capacity(8192);
            let mut decoder = DeflateDecoder::new(rd);
            decoder.read_to_string(&mut read_buffer).unwrap();
            read_buffer
                .lines()
                .map(|s| s.to_owned())
                .collect::<Vec<_>>()
        })
    )
}

fn compress_and_decompress<I>(lines: I)
    where I: IntoIterator<Item = String>,
{
    const BATCH_SIZE: usize = 10;
    let (tx, rx) = mpsc::channel();

    let receive_handle = thread::spawn(move || {
        for s in decompress(rx) {
            println!("{}", s);
        }
    });

    compress(lines, BATCH_SIZE, tx).unwrap();

    receive_handle.join().unwrap();
}

fn main() {
    let stdin = stdin();
    compress_and_decompress(stdin.lock().lines().filter_map(|res| res.ok()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        const DATA: &[u8] = include_bytes!("elements.json");

        compress_and_decompress(String::from_utf8_lossy(DATA).lines().map(|s| s.to_owned()));
    }
}
