use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

fn check_gif(rd: impl Read) {
    let decoder = gif::DecodeOptions::new();
    let mut decoder = decoder.read_info(rd).unwrap();
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        println!("Delay = {}, transparent = {:?}",
            frame.delay,
            frame.transparent);
    }
}

fn check_png(rd: impl Read) {
    let decoder = png::Decoder::new(rd);
    let reader = decoder.read_info().unwrap();
    let info = reader.info();
    println!("Color type = {:?}, transparency = {:?}", info.color_type, info.trns);
}

fn get_extension_lowercase(path: &Path) -> Option<String> {
    let ext = path.extension()?;
    let ext_str = ext.to_str()?;
    Some(ext_str.to_ascii_lowercase())
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    let file_path = args[1].parse::<PathBuf>().unwrap();
    let file = File::open(&file_path).unwrap();

    match get_extension_lowercase(&file_path).as_deref() {
        Some("gif") => check_gif(file),
        Some("png") => check_png(file),
        _ => {
            println!("Not GIF or PNG.");
        }
    };
}
