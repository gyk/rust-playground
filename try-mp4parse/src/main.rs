extern crate mp4parse;
extern crate mp4parse_capi;

use std::fs::File;
use std::io::Read;

use mp4parse::MediaContext;
use mp4parse_capi::*;

// ===== Helpers =====
extern fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
   let input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
   let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
   match input.read(&mut buf) {
       Ok(n) => n as isize,
       Err(_) => -1,
   }
}

mod naked {
    use super::*;

    use std::collections::HashMap;

    pub struct Mp4parseParserNaked {
        pub context: MediaContext,
        _io: Mp4parseIo,
        _poisoned: bool,
        _opus_header: HashMap<u32, Vec<u8>>,
        _pssh_data: Vec<u8>,
        _sample_table: HashMap<u32, Vec<Mp4parseIndice>>,
    }

    pub fn get_context(undisclosed: &Mp4parseParser) -> &MediaContext {
        use std::mem;

        let naked = unsafe {
            mem::transmute::<&Mp4parseParser, &Mp4parseParserNaked>(undisclosed)
        };

        &naked.context
    }
}

// Issue the command `cargo run movie.mp4` to dump the MP4 file.
fn main() {
    let args: Vec<String> = ::std::env::args().collect();

    let mut mp4_movie = File::open(&args[1]).unwrap();

    let io = Mp4parseIo {
        read: Some(buf_read),
        userdata: &mut mp4_movie as *mut _ as *mut std::os::raw::c_void,
    };
    unsafe {
        let parser = mp4parse_new(&io);
        match mp4parse_read(parser) {
            Mp4parseStatus::Ok => (),
            not_ok => {
                println!("Fail to parse, status = {:?}", not_ok);
                return;
            }
        }

        let mut frag_info = Mp4parseFragmentInfo::default();
        match mp4parse_get_fragment_info(parser, &mut frag_info) {
            Mp4parseStatus::Ok => {
                println!("[parse_fragment_info] {:?}", frag_info);
            },
            _ => {
                println!("[parse_fragment_info] failed");
                return;
            }
        }

        let mut counts: u32 = 0;
        match mp4parse_get_track_count(parser, &mut counts) {
            Mp4parseStatus::Ok => (),
            _ => {
                println!("-- mp4parse_get_track_count failed");
                return;
            }
        }

        for i in 0 .. counts {
            let mut track_info = Mp4parseTrackInfo {
                track_type: Mp4parseTrackType::Audio,
                codec: Mp4parseCodec::Unknown,
                track_id: 0,
                duration: 0,
                media_time: 0,
            };
            match mp4parse_get_track_info(parser, i, &mut track_info) {
                Mp4parseStatus::Ok => {
                    println!("[parse_get_track_info] {:?}", track_info);
                },
                _ => {
                    println!("[parse_get_track_info] failed, track id: {}", i);
                    return;
                }
            }

            match track_info.track_type {
                Mp4parseTrackType::Audio => {
                    let mut audio_info = Mp4parseTrackAudioInfo::default();
                    match mp4parse_get_track_audio_info(parser, i, &mut audio_info) {
                        Mp4parseStatus::Ok => {
                          println!("[get_track_audio_info] {:#?}", audio_info);
                        },
                        _ => {
                          println!("[get_track_audio_info] failed, track id: {}", i);
                          return;
                        }
                    }
                },
                Mp4parseTrackType::Video => {
                    let mut video_info = Mp4parseTrackVideoInfo::default();
                    match mp4parse_get_track_video_info(parser, i, &mut video_info) {
                        Mp4parseStatus::Ok => {
                          println!("[get_track_video_info] {:#?}", video_info);
                        },
                        _ => {
                          println!("[get_track_video_info] failed, track id: {}", i);
                          return;
                        }
                    }
                },
            }

            let mut indices = Mp4parseByteData::default();
            match mp4parse_get_indice_table(parser, track_info.track_id, &mut indices) {
                Mp4parseStatus::Ok => {
                  println!("[get_indice_table] track_id {} indices {:#?}", track_info.track_id, indices);
                },
                _ => {
                  println!("[get_indice_table] failed, track_info.track_id: {}", track_info.track_id);
                  return;
                }
            }
        }

        println!("\n{:#?}", naked::get_context(&*parser));
        mp4parse_free(parser);
    }
}
