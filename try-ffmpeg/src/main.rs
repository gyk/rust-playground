use std::path::PathBuf;

use anyhow::Result;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::Flags;
use ffmpeg::util::frame::video::Video;
use image::{DynamicImage, ImageBuffer};

fn main() -> Result<()> {
    ffmpeg::init()?;

    let args: Vec<String> = std::env::args().collect();
    let in_path = args[1].parse::<PathBuf>()?;
    let out_dir = &args[2];

    let mut input_ctx = ffmpeg::format::input(&in_path)?;

    for datum in input_ctx.metadata().iter() {
        println!("\t{}: {}", datum.0, datum.1);
    }


    println!("Duration: {}ms", input_ctx.duration());
    println!("#Streams: {}", input_ctx.nb_streams());
    println!("#Chapters: {}", input_ctx.nb_chapters());

    for s in input_ctx.streams() {
        println!("Stream #{}", s.index());
        println!("\tt = {}, fps = {}, rate = {}", s.time_base(), s.avg_frame_rate(), s.rate());
    }

    // input_ctx.streams().best(ffmpeg::media::Type::Video)
    let v_stream = input_ctx.stream(0)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let v_index = v_stream.index();

    let decoder_ctx = ffmpeg::codec::Context::from_parameters(v_stream.parameters())?;
    let mut decoder = decoder_ctx.decoder().video()?;

    let width = decoder.width();
    let height = decoder.height();
    // decoder.

    let mut scaler_ctx = ffmpeg::software::scaling::Context::get(
        decoder.format(),
        width,
        height,
        Pixel::RGB24,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let mut i_frame = 0;

    for packet in input_ctx.packets().filter_map(|(stream, packet)| {
        if stream.index() == v_index {
            Some(packet)
        } else {
            None
        }
    }) {
        decoder.send_packet(&packet)?;

        let mut v_frame = Video::empty();
        while decoder.receive_frame(&mut v_frame).is_ok() {
            let mut rgb_frame = Video::empty();
            scaler_ctx.run(&v_frame, &mut rgb_frame)?;

            if i_frame > 100 {
                return Ok(());
            }
            println!("#{}", i_frame);

            if i_frame % 20 == 0 {
                // let raw_data: &[[u8; 3]] = unsafe {
                //     std::slice::from_raw_parts(rgb_frame.data(0).as_ptr() as _, (width * height) as usize)
                // };

                use image::Rgb;

                let im: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(width, height, rgb_frame.data(0).to_vec())
                    .ok_or_else(|| anyhow::Error::msg("Cannot create ImageBuffer"))?;
                let out_path = format!("{}/{}.jpg", out_dir, i_frame);
                im.save(&out_path)?;
            }

            i_frame += 1;
        }
    }

    println!("Done.");

    Ok(())
}

// fn save_image(frame: &Video, index: usize) {
//     let im = DynamicImage::ImageRgba8(
//         ImageBuffer::from_raw(width, height, rgba_bytes)
//             .ok_or_else(|| anyhow::Error::msg("Cannot create ImageBuffer"))?,
//     );
//     dbg!(t.elapsed());

//     im.save(&out_path)?;
// }
