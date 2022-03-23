use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use image::{DynamicImage, ImageBuffer};
use psd::Psd;

fn main() -> Result<()> {
    let args: Vec<String> = ::std::env::args().collect();
    let in_path = args[1].parse::<PathBuf>()?;
    let out_path = args[2].parse::<PathBuf>()?;

    let t = Instant::now();
    let content = fs::read(&in_path)?;
    dbg!(t.elapsed());
    let psd_image = Psd::from_bytes(&content)?;
    dbg!(t.elapsed());

    let width = psd_image.width();
    let height = psd_image.height();
    let rgba_bytes = psd_image.rgba();
    dbg!(t.elapsed());

    let im = DynamicImage::ImageRgba8(
        ImageBuffer::from_vec(width, height, rgba_bytes)
            .ok_or_else(|| anyhow::Error::msg("Cannot create ImageBuffer"))?,
    );
    dbg!(t.elapsed());

    im.save(&out_path)?;
    Ok(())
}
