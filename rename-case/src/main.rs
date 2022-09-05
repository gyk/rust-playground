use std::io;
use std::path::{Path, PathBuf};

use dunce::realpath;
use normpath::PathExt;

use windows::core::{self, HSTRING, PCWSTR};
use windows::Win32::UI::Shell::{PathAllocCanonicalize, PATHCCH_CANONICALIZE_SLASHES};
use windows::Win32::Storage::FileSystem::GetFullPathNameW;

fn win32_canonicalize(p: &Path) -> io::Result<PathBuf> {
    let p = p.as_os_str().to_str().unwrap();
    let hs_path = HSTRING::from(p);
    let p2 = unsafe {
        PathAllocCanonicalize(&hs_path, PATHCCH_CANONICALIZE_SLASHES).unwrap()
    };
    let p2 = unsafe { p2.to_string().unwrap() };
    Ok(PathBuf::from(p2))
}

fn win32_canonicalize2(p: &Path) -> io::Result<PathBuf> {
    let p = p.as_os_str().to_str().unwrap();
    let hs_path = HSTRING::from(p);
    let mut buffer = vec![0_u16; 256];
    unsafe {
        GetFullPathNameW(&hs_path, Some(&mut buffer), None)
    };
    let p2 = String::from_utf16(&buffer).unwrap();
    Ok(PathBuf::from(p2))
}

fn main() {
    let paths= [
        Path::new(r"r:\smallcase.txt"),
        Path::new(r"s:\fs-tests\smallcase.txt"),
        Path::new(r"R:"),
        Path::new(r"S:"),
        Path::new(r"\\NAS\\home"),
        Path::new(r"X:"),
    ];

    println!();
    for p in paths {
        println!("Path = {}", p.display());
        println!("canonicalize: {:?}", p.canonicalize());
        println!("normalize: {:?}", p.normalize());
        println!("realpath: {:?}", realpath(p));
        println!("win32_canonicalize: {:?}", win32_canonicalize(p));
        // println!("win32_canonicalize2: {:?}", win32_canonicalize2(p));
        println!();
    }
}
