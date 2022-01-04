use std::path::PathBuf;

use clipboard_win::{Clipboard, Getter};
use clipboard_win::formats::*;

fn main() {
    let _clipboard = Clipboard::new().unwrap();

    let mut image_buffer = Vec::new();
    match RawData(CF_DIB).read_clipboard(&mut image_buffer) {
        Ok(len) => {
            println!("Read dib, len = {}", len);
        }
        Err(e) => println!("Read dib error {:?}", e),
    }

    let mut image_buffer = Vec::new();
    match Bitmap.read_clipboard(&mut image_buffer) {
        Ok(len) => {
            println!("Read bitmap, len = {}", len);
            return;
        }
        Err(e) => println!("Read bitmap error {:?}", e),
    }

    let mut file_list = Vec::<PathBuf>::new();
    match FileList.read_clipboard(&mut file_list) {
        Ok(len) => {
            println!("Read file list, len = {}, list = {:#?}", len, file_list);
            return;
        }
        Err(e) => println!("Read file list error {:?}", e),
    }

    let mut s = String::new();
    if let Ok(len) = Unicode.read_clipboard(&mut s) {
        println!("Read string, len = {}, content = '{}'", len, s);
        return;
    }

    println!("Nothing in clipboard");
}
