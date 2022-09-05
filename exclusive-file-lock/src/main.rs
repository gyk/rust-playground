use std::fs::{self, OpenOptions};
use std::io::Result;
use std::path::Path;

use std::os::windows::fs::OpenOptionsExt;

mod lock;

fn main() -> Result<()>  {
    let path = r"C:\Users\Dell\Downloads\sysimg\LOCK";
    let path2 = r"C:\Users\Dell\Downloads\sysimg\LOCK2";

    let l1 = lock::acquire_lock(path);
    let l2 = lock::acquire_lock(path2);

    if l1.is_some() && l2.is_some() {
        println!("locked");
        drop(l1);
        drop(l2);
        std::fs::rename(path, path2)?;
    } else {
        println!("Cannot lock");
    }

    Ok(())
}
