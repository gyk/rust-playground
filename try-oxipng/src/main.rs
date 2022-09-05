use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;

fn main() -> Result<()> {
    let path = PathBuf::from(std::env::args().skip(1).next().ok_or(anyhow::Error::msg("Missing path"))?);
    let buffer = std::fs::read(path)?;
    // let im = image::open(&path)?;
    // println!("Image loaded, w= {}, h = {}", im.width(), im.height());

    // let mut buffer = Vec::new();
    // im.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
    println!("PNG buffer len = {}", buffer.len());

    let x = oxipng::PngData::from_slice(&buffer, false)?;
    println!("PngData len = {}", x.idat_data.len());

    // let mut compression = IndexSet::new();
    // compression.insert(11);

    let opts = oxipng::Options {
        fix_errors: false,
        // deflate: oxipng::Deflaters::Libdeflater,
        // use_heuristics: false,
        // palette_reduction: false,
        // bit_depth_reduction: false,
        // color_type_reduction: false,
        ..Default::default()
    };
    println!("opts = {:#?}", opts);
    // opts.verbosity = None;

    let optimized_buffer = match oxipng::optimize_from_memory(&buffer, &opts) {
        Ok(optimized) => optimized,
        Err(_e) => buffer,
    };

    println!("Optimized PNG buffer len = {}", optimized_buffer.len());

    Ok(())
}
