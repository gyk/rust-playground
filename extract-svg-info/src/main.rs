//! Extract the dimension of SVG images
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use svgtypes::{Length, LengthUnit, ViewBox};
use xml::reader::{EventReader, XmlEvent};

// https://svgwg.org/specs/integration/#svg-css-sizing
const DEFAULT_SVG_WIDTH: usize = 300;
const DEFAULT_SVG_HEIGHT: usize = 150;

// See https://developer.mozilla.org/en-US/docs/Web/SVG/Content_type#length.
fn calc_dimension_in_px(
    width: Option<Length>,
    height: Option<Length>,
    view_box: Option<ViewBox>,
) -> Option<(usize, usize)> {
    match (width, height, view_box) {
        (Some(w), Some(h), _) if w.unit == h.unit => {
            if matches!(w.unit, LengthUnit::None | LengthUnit::Px) {
                Some((w.num.round() as usize, h.num.round() as usize))
            } else {
                let (w, h) = (w.num, h.num);
                if w <= 0.0 || h <= 0.0 {
                    None
                } else {
                    let h = h * (DEFAULT_SVG_WIDTH as f64) / w;
                    Some((DEFAULT_SVG_WIDTH, h.round() as usize))
                }
            }
        }
        (_, _, Some(ViewBox { w, h, .. })) => {
            if w > 0.0 && h > 0.0 {
                Some((w.round() as usize, h.round() as usize))
            } else {
                None
            }
        }
        (Some(w), None, _) if matches!(w.unit, LengthUnit::None | LengthUnit::Px) => {
            Some((w.num.round() as usize, DEFAULT_SVG_HEIGHT))
        }
        (None, Some(h), _) if matches!(h.unit, LengthUnit::None | LengthUnit::Px) => {
            Some((DEFAULT_SVG_WIDTH, h.num.round() as usize))
        }
        _ => None,
    }
}

fn extract_dimension(source: impl Read) -> Result<(usize, usize)> {
    let parser = EventReader::new(source);
    for el in parser {
        match el {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                if &name.local_name == "svg" {
                    let mut width: Option<Length> = None;
                    let mut height: Option<Length> = None;
                    let mut view_box: Option<ViewBox> = None;
                    for attr in attributes {
                        if &attr.name.local_name == "width" {
                            width = attr.value.parse().ok();
                        } else if &attr.name.local_name == "height" {
                            height = attr.value.parse().ok();
                        } else if &attr.name.local_name == "viewBox" {
                            view_box = attr.value.parse().ok();
                        }
                    }

                    if let Some((width, height)) = calc_dimension_in_px(width, height, view_box) {
                        return Ok((width, height));
                    }
                    break;
                }
            }
            Err(_) => break,
            _ => {}
        }
    }

    Ok((DEFAULT_SVG_WIDTH, DEFAULT_SVG_HEIGHT))
}

fn main() -> Result<()> {
    let args: Vec<String> = ::std::env::args().collect();
    let file_path = args[1].parse::<PathBuf>().unwrap();
    let file = File::open(file_path)?;

    let (w, h) = extract_dimension(file)?;
    println!("The dimension: width = {:?}, height = {:?}", w, h);

    Ok(())
}
