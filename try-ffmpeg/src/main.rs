use std::path::PathBuf;

use anyhow::Result;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::Flags;
use ffmpeg::util::frame::video::Video;
use image::ImageBuffer;

fn codec_id_to_tag(id: u32) -> String {
    unsafe { std::str::from_utf8_unchecked(id.to_ne_bytes().as_ref()) }.to_owned()
}

fn main() -> Result<()> {
    ffmpeg::init()?;

    let args: Vec<String> = std::env::args().collect();
    let in_path = args[1].parse::<PathBuf>()?;
    let out_dir = &args[2];

    let mut input_ctx = ffmpeg::format::input(&in_path)?;

    for datum in input_ctx.metadata().iter() {
        println!("\t{}: {}", datum.0, datum.1);
    }


    println!("Duration: {}ms", input_ctx.duration() / 1000);
    println!("#Streams: {}", input_ctx.nb_streams());
    println!("#Chapters: {}", input_ctx.nb_chapters());

    dbg!(input_ctx.bit_rate());
    dbg!(input_ctx.format().extensions());
    dbg!(input_ctx.format().name());
    dbg!(input_ctx.format().mime_types());

    for s in input_ctx.streams() {
        println!("Stream #{}", s.index());
        println!("\tt = {}, fps = {}, rate = {}", s.time_base(), s.avg_frame_rate(), s.rate());
        println!("\tframe rate = {}", unsafe { *s.parameters().as_ptr() }.bit_rate);

        for datum in &s.metadata() {
            println!("\t\t{}: {}", datum.0, datum.1);
        }
    }

    if let Some(a_stream) = input_ctx.streams().best(ffmpeg::media::Type::Audio) {
        let decoder_ctx = ffmpeg::codec::Context::from_parameters(a_stream.parameters())?;
        let mut decoder = decoder_ctx.decoder().audio()?;

        dbg!(codec_id_to_tag(unsafe { (*a_stream.parameters().as_ptr()).codec_tag }));

        // dbg!(decoder());
        dbg!(decoder.bit_rate());
        dbg!(decoder.channel_layout());
        dbg!(decoder.channels());
        dbg!(decoder.format().name());
        dbg!(decoder.frame_rate());
        dbg!(decoder.frame_size());
        dbg!(decoder.max_bit_rate());
        dbg!(decoder.profile());
        dbg!(decoder.rate());
        dbg!(decoder.time_base());
    }

    let v_stream = input_ctx.streams().best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let v_index = v_stream.index();

    let decoder_ctx = ffmpeg::codec::Context::from_parameters(v_stream.parameters())?;
    let mut decoder = decoder_ctx.decoder().video()?;

    dbg!(decoder.aspect_ratio());
    dbg!(decoder.bit_rate());
    dbg!(decoder.chroma_location());
    dbg!(decoder.color_primaries().name());
    dbg!(decoder.color_range().name());
    dbg!(decoder.color_space().name());
    dbg!(decoder.color_transfer_characteristic().name());
    dbg!(decoder.format().descriptor().unwrap().name());
    dbg!(decoder.profile());
    if let Some(codec) = decoder.codec() {
        dbg!(codec.name());
        dbg!(codec.description());
        dbg!(codec.id());
        dbg!(codec.capabilities());

        dbg!(codec_id_to_tag(unsafe { (*v_stream.parameters().as_ptr()).codec_tag }));
        dbg!(unsafe { (*v_stream.parameters().as_ptr()).level });

    }
    dbg!(decoder.max_bit_rate());

    let width = decoder.width();
    let height = decoder.height();

    let mut scaler_ctx = ffmpeg::software::scaling::Context::get(
        decoder.format(),
        width,
        height,
        Pixel::RGBA,
        width / 2,
        height / 2,
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

            println!("\tINFO: {:?} {} {} -> {:?} {} {}", v_frame.format(), v_frame.width(), v_frame.height(),
                scaler_ctx.output().format, scaler_ctx.output().width, scaler_ctx.output().height);

            if i_frame > 100 {
                return Ok(());
            }
            println!("#{}", i_frame);

            if i_frame % 20 == 0 {
                use image::{Rgb, Rgba};

                let im: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(width / 2, height / 2, rgb_frame.data(0).to_vec())
                    .ok_or_else(|| anyhow::Error::msg("Cannot create ImageBuffer"))?;
                let out_path = format!("{}/{}.jpg", out_dir, i_frame);
                im.save(&out_path)?;
                return Ok(());
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
