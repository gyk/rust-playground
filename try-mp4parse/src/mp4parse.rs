extern crate mp4parse;

use std::io;

use mp4parse::read_mp4;
use mp4parse::MediaContext;

// Usage:
//
// `cat $MOVIE.mp4 | cargo run --bin mp4parse`
//
// For fragmented MP4:
//
// `cat init.mp4 segment1.m4s | cargo run --bin mp4parse`
fn main() {
    let mut mp4_movie = io::stdin();
    let mut context = MediaContext::new();
    read_mp4(&mut mp4_movie, &mut context).unwrap();
    println!("{:#?}", context);
}
