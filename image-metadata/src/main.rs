use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use image::{self, ImageDecoder};
use walkdir::WalkDir;

const CHUNK_SIZE: usize = 32;

#[derive(Debug, Default)]
pub struct ImageMetadata {
    pub format: Option<image::ImageFormat>,
    pub dimension: Option<(u32, u32)>,
    pub color_type: Option<image::ColorType>,
    pub consumed_bytes: Option<u64>,
}

#[derive(Clone)]
struct SharedReader {
    inner: Arc<Mutex<File>>,
}

impl Read for SharedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().read(buf)
    }
}

impl Seek for SharedReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.lock().unwrap().seek(pos)
    }
}

pub fn image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetadata> {
    let mut shared_reader = SharedReader {
        inner: Arc::new(Mutex::new(File::open(&path)?)),
    };

    let reader =
        image::io::Reader::new(BufReader::with_capacity(CHUNK_SIZE, shared_reader.clone()))
            .with_guessed_format()?;
    let format = match reader
        .format()
        .or_else(|| image::ImageFormat::from_path(path).ok())
    {
        Some(format) => format,
        None => return Ok(ImageMetadata::default()),
    };

    let mut md = image_metadata_with_format(reader.into_inner(), format).unwrap_or_default();
    md.format.replace(format);
    md.consumed_bytes = shared_reader.stream_position().ok();

    Ok(md)
}

// image-rs currently only supports getting dimension lazily.
fn image_metadata_with_format<R: BufRead + Seek>(
    fin: R,
    format: image::ImageFormat,
) -> Option<ImageMetadata> {
    match format {
        #[cfg(feature = "avif-decoder")]
        image::ImageFormat::Avif => {
            let d = image::avif::AvifDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Jpeg => {
            let d = image::jpeg::JpegDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Png => {
            // image-rs reads too many bytes.
            let mut md = image_metadata_by_imagesize(fin)?;
            // imagesize can't probe color type.
            md.color_type = Some(image::ColorType::Rgba8);
            Some(md)
        }
        image::ImageFormat::Gif => {
            let d = image::gif::GifDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::WebP => {
            let d = image::webp::WebPDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Tiff => {
            // image-rs reads too many bytes.
            image_metadata_by_imagesize(fin)
        }
        image::ImageFormat::Tga => {
            let d = image::tga::TgaDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Dds => {
            let d = image::dds::DdsDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Bmp => {
            let d = image::bmp::BmpDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Ico => {
            let d = image::ico::IcoDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Hdr => {
            let d = image::hdr::HdrAdapter::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        image::ImageFormat::Pnm => {
            let d = image::pnm::PnmDecoder::new(fin).ok()?;
            image_metadata_from_decoder(d)
        }
        _unsupported_format => None,
    }
}

fn image_metadata_from_decoder<'a, R: Read>(
    decoder: impl ImageDecoder<'a, Reader = R>,
) -> Option<ImageMetadata> {
    let mut md = ImageMetadata::default();

    md.dimension.replace(decoder.dimensions());
    md.color_type.replace(decoder.color_type());

    Some(md)
}

fn image_metadata_by_imagesize<R: BufRead + Seek>(fin: R) -> Option<ImageMetadata> {
    const CHUNK_SIZE: usize = 4096;
    let mut blob = [0; CHUNK_SIZE];
    let mut r = fin.take(CHUNK_SIZE as u64);

    #[allow(clippy::unused_io_amount)]
    {
        r.read(&mut blob).ok()?;
    }

    let mut md = ImageMetadata::default();
    let size = imagesize::blob_size(&blob).ok()?;
    md.dimension
        .replace((size.width as u32, size.height as u32));
    Some(md)
}

fn main() -> Result<()> {
    let args: Vec<String> = ::std::env::args().collect();
    let p = args[1].parse::<PathBuf>().unwrap();

    if p.is_file() {
        println!("{:#?}", image_metadata(&p));
    } else {
        for entry in WalkDir::new(&p).into_iter().flatten() {
            let is_file = entry.metadata()?.is_file();
            if is_file {
                let p = entry.path();
                println!("\n{}", p.to_string_lossy().as_ref());
                println!("{:#?}", image_metadata(&p));
            }
        }
    }

    Ok(())
}
