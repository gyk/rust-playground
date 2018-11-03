//! This is how men reverse lines before the invention of memory mapped file.
//!
//! **Keywords**: retrospection, nostalgia, reminiscence.
//!

use std::cmp;
use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};

const BLOCK_SIZE: usize = 4096;

pub struct RevLinesIter<'a> {
    file: &'a File,
    cursor: u64,
    buffer: Vec<u8>,
}

impl<'a> RevLinesIter<'a> {
    pub fn new(mut file: &'a File) -> Self {
        let file_len = file.metadata().expect("No metadata").len();
        file.seek(SeekFrom::Start(file_len)).expect("Cannot seek to end");
        Self {
            file,
            cursor: file_len,
            buffer: vec![0; BLOCK_SIZE],
        }
    }

    pub fn next_line(&mut self) -> Result<Option<String>> {
        let old_cursor = self.cursor;
        loop {
            let read_size = cmp::min(BLOCK_SIZE as u64, self.cursor);
            if read_size == 0 {
                return Ok(None);
            }

            // NOTE: if `file_len % BLOCK_SIZE != 0` and `BLOCK_SIZE` is indeed the disk sector
            // size, each read action actually reads 2 sectors.
            self.cursor -= read_size;
            self.file.seek(SeekFrom::Start(self.cursor))?;

            self.file.read_exact(&mut self.buffer[..(read_size as usize)])?;
            self.file.seek(SeekFrom::Start(self.cursor))?;

            if let Some(p) = self.buffer[..(read_size as usize)]
                                 .iter()
                                 .rev()
                                 .position(|&x| x == b'\n')
                                 .or_else(||
                                    if self.cursor == 0 { Some(read_size as usize) } else { None }
                                 ) {
                let cursor_newline: i64 = (self.cursor + read_size)
                    .checked_sub(p as u64 + 1)
                    .map_or(-1, |x| x as i64);

                self.cursor = (cursor_newline + 1) as u64; // after newline
                self.file.seek(SeekFrom::Start(self.cursor))?;

                let mut buffer = vec![0; (old_cursor - self.cursor) as usize];
                self.file.read_exact(&mut buffer[..])?;

                self.cursor = self.cursor.saturating_sub(1); // before newline
                self.file.seek(SeekFrom::Start(self.cursor))?;

                return Ok(Some(unsafe { String::from_utf8_unchecked(buffer) }));
            }
        }
    }

    #[allow(dead_code)]
    fn check_cursor(&self) -> bool {
        let mut file: &File = &mut &self.file;
        let file_cursor = file.seek(SeekFrom::Current(0)).unwrap();
        file_cursor == self.cursor
    }
}

impl<'a> Iterator for RevLinesIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_line().unwrap_or(None)
    }
}

pub fn file_lines_backwards<'a>(file: &'a File) -> RevLinesIter<'a> {
    RevLinesIter::new(file)
}
