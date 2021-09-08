mod color_thief;

use std::path::PathBuf;

use crate::color_thief::ColorFormat;
use image::{self, ColorType, GenericImageView};

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    let file_path = args[1].parse::<PathBuf>().unwrap();
    let picture = image::open(&file_path).unwrap();

    println!("Color type = {:?}", picture.color());
    let color_type = match picture.color() {
        ColorType::Bgr8 => ColorFormat::Bgr,
        ColorType::Bgra8 => ColorFormat::Bgra,
        ColorType::Rgb8 => ColorFormat::Rgb,
        ColorType::Rgba8 => ColorFormat::Rgba,
        _ => panic!("Unsupported color type"),
    };

    let (width, height) = picture.dimensions();
    for i_tenth in 1..=10 {
        let scale = f64::from(i_tenth) / 10.0;
        let width_s = (f64::from(width) * scale) as u32;
        let thumb = if width_s == width {
            picture.clone()
        } else {
            picture.thumbnail(width_s, height)
        };

        println!("\n/* Scale = {}, size = {:?} */", scale, thumb.dimensions());
        let colors = color_thief::get_palette(thumb.as_bytes(), color_type, 10, 13).unwrap();
        println!("* {{");
        for color in colors {
            println!("  color: #{:02X}{:02X}{:02X};", color.r, color.g, color.b);
        }
        println!("}}");
    }
}
