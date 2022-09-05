use std::fs::{self, File, OpenOptions};
use std::io::Result;
use std::path::Path;

use std::os::windows::fs::OpenOptionsExt;

pub struct FileLock(Option<File>);

#[cfg(not(windows))]
pub fn acquire_lock<P: AsRef<Path>>(_path: P) -> Option<FileLock> {
    Some(FileLock(None))
}

#[cfg(windows)]
pub fn acquire_lock<P: AsRef<Path>>(path: P) -> Option<FileLock> {
    use std::io;

    let path = path.as_ref();
    let mut opts = OpenOptions::new();
    // opts.read(true).share_mode(0).custom_flags(0x02000000);
    opts.read(true).share_mode(0x00000001 | 0x00000004).custom_flags(0x02000000);
    // opts.read(true).write(true).custom_flags(0x02000000);
    let file = match dbg!(opts.open(path)) {
        Ok(file) => Some(file),
        Err(e) => {
            println!("e: {}", e.kind());
            if e.kind() == io::ErrorKind::NotFound {
                None
            } else {
                return None;
            }
        }
    };
    Some(FileLock(file))
}
