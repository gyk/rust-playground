extern crate memmap;

use std::env;
use std::fs::File;
use std::io::Result;
use std::str;

use memmap::Mmap;

fn lines_backwards<'a>(text: &'a [u8]) -> impl Iterator<Item = &str> + 'a {
    text.rsplit(|&x| x == b'\n')
        .map(|l| unsafe { str::from_utf8_unchecked(l) })
}

// Usage:
//
//     cargo run -- $FILE
fn main() -> Result<()> {
    let path = env::args().skip(1).next().expect("Missing $FILE");
    let file = File::open(&path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    for l in lines_backwards(&mmap) {
        println!("{}", l);
    }
    Ok(())
}
