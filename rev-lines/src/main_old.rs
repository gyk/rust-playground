mod old_fashioned;

use std::env;
use std::fs::File;

// Usage:
//
//     cargo run --bin rev-lines-old -- $FILE
fn main() {
    let path = env::args().skip(1).next().expect("Missing $FILE");
    let file = File::open(&path).expect("Cannot open file");
    for l in old_fashioned::file_lines_backwards(&file) {
        println!("{}", l);
    }
}
