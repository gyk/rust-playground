fn main() {
    fs_extra::dir::copy(
        "C:\\Users\\Dell\\AppData\\Roaming\\Pixcall\\roaming\\.pixcall",
        // "C:\\Users\\Dell\\PX_Meta2\\.pixcall",
        r"R:\aaa\.pixcall",
        &fs_extra::dir::CopyOptions {
            overwrite: true,
            content_only: true,
            ..Default::default()
        }
    ).unwrap();
}
