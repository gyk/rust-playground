#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate fuzztest;

use fuzztest::Decoder;

fuzz_target!(|data: &[u8]| {
    let _ = Decoder::decode(data);
});
