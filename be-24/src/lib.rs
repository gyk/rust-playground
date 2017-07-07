//! Big-endian Signed/unsigned 24-bit Integer IO Benchmark
#![feature(test)]
#[macro_use]
extern crate lazy_static;

extern crate byteorder;
extern crate test;
extern crate rand;

use std::io::{self, Read, Write, Result};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::mem;
use std::ptr;


// Safe, u8 slice
// ================================================================ //

#[inline]
pub fn read_be_u24_slice(i: &[u8]) -> Result<u32> {
    if i.len() < 3 {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Early EOF"))
    } else {
        Ok(((i[0] as u32) << 16) + ((i[1] as u32) << 8) + i[2] as u32)
    }
}

#[inline]
pub fn write_be_u24_slice(o: &mut [u8], n: u32) -> Result<()> {
    if o.len() < 3 {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Early EOF"))
    } else {
        o[0] = (n >> 16) as u8;
        o[1] = (n >> 8) as u8;
        o[2] = n as u8;
        Ok(())
    }
}

// Safe, Read/Write traits, step-by-step
// ================================================================ //

#[inline]
pub fn read_be_u24_step(rd: &mut Read) -> Result<u32> {
    let i0 = rd.read_u8()? as u32;
    let i1 = rd.read_u8()? as u32;
    let i2 = rd.read_u8()? as u32;

    Ok((i0 << 16) + (i1 << 8) + i2)
}

#[inline]
pub fn write_be_u24_step(wr: &mut Write, n: u32) -> Result<()> {
    wr.write_u8((n >> 16) as u8)?;
    wr.write_u8((n >> 8) as u8)?;
    wr.write_u8(n as u8)?;

    Ok(())
}

// Safe, Read/Write traits, step-by-step, without `try!`/`?`
// ================================================================ //

#[inline]
pub fn read_be_u24_step_notry(rd: &mut Read) -> Result<u32> {
    let i0 = rd.read_u8().unwrap() as u32;
    let i1 = rd.read_u8().unwrap() as u32;
    let i2 = rd.read_u8().unwrap() as u32;

    Ok((i0 << 16) + (i1 << 8) + i2)

    // Similar performance:
    //
    //    let mut buf = [0; 1];
    //    rd.read_exact(&mut buf)?;
    //    let i0 = buf[0] as u32;
    //    rd.read_exact(&mut buf)?;
    //    let i1 = buf[0] as u32;
    //    rd.read_exact(&mut buf)?;
    //    let i2 = buf[0] as u32;
    //
    //    Ok((i0 << 16) + (i1 << 8) + i2)
}

#[allow(unused_must_use)]
#[inline]
pub fn write_be_u24_step_notry(wr: &mut Write, n: u32) -> Result<()> {
    wr.write_u8((n >> 16) as u8);
    wr.write_u8((n >> 8) as u8);
    wr.write_u8(n as u8);

    Ok(())
}

// Safe, Read/Write traits, accesses once
// ================================================================ //

#[inline]
pub fn read_be_u24_once(rd: &mut Read) -> Result<u32> {
    let mut buf = [0; 3];
    rd.read_exact(&mut buf)?;
    let i0 = buf[0] as u32;
    let i1 = buf[1] as u32;
    let i2 = buf[2] as u32;

    Ok((i0 << 16) + (i1 << 8) + i2)
}

#[inline]
pub fn write_be_u24_once(wr: &mut Write, n: u32) -> Result<()> {
    let mut buf = [0; 3];
    buf[0] = (n >> 16) as u8;
    buf[1] = (n >> 8) as u8;
    buf[2] = n as u8;

    wr.write_all(&buf)
}

// Unsafe, Read/Write traits
// ================================================================ //

// Read
pub trait ReadBe24: io::Read {
    #[inline]
    fn read_be_u24(&mut self) -> Result<u32> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf)?;
        let mut data: u32 = 0;
        unsafe {
            ptr::copy_nonoverlapping(
                (&buf).as_ptr(),
                (&mut data as *mut u32 as *mut u8).offset(1),
                3
            );
        }
        Ok(data.to_be())
    }

    #[inline]
    fn read_be_i24(&mut self) -> Result<i32> {
        let x = self.read_be_u24()?;
        if x & 0x80_00_00 != 0 {
            Ok((x | 0xFF_00_00_00) as i32)
        } else {
            Ok(x as i32)
        }
    }
}

impl<R: io::Read + ?Sized> ReadBe24 for R {}

// Write
pub trait WriteBe24: io::Write {
    #[inline]
    fn write_be_u24(&mut self, data: u32) -> Result<()> {
        // debug_assert!({
        //     let msb = data & 0xFF_00_00_00;
        //     msb == 0 || msb == 0xFF_00_00_00
        // }, "Data exceeds 24 bits");

        let mut buffer = [0u8; 3];
        unsafe {
            let bytes = mem::transmute::<_, [u8; 4]>(data.to_be());
            ptr::copy_nonoverlapping(
                (&bytes).as_ptr().offset(1),
                (&mut buffer).as_mut_ptr(),
                3
            );
        }

        self.write_all(&buffer)
    }

    #[inline]
    fn write_be_i24(&mut self, data: i32) -> Result<()> {
        self.write_be_u24(data as u32)
    }
}

impl<W: io::Write + ?Sized> WriteBe24 for W {}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use test::Bencher;
    use std::sync::Mutex;

    #[test]
    fn test_be_24() {
        // Read
        let pos_rd_buf = [0x01, 0x23, 0x45];
        {
            let pos_rd: &[u8] = &pos_rd_buf;
            assert_eq!(read_be_u24_slice(pos_rd).unwrap(), 0x012345_u32);
        }
        {
            let mut pos_rd: &[u8] = &pos_rd_buf;
            // Try commenting out the following line:
            assert_eq!(read_be_u24_step(&mut pos_rd).unwrap(), 0x012345_u32);
        }
        {
            let mut pos_rd: &[u8] = &pos_rd_buf;
            assert_eq!(read_be_u24_once(&mut pos_rd).unwrap(), 0x012345_u32);
        }
        {
            let mut pos_rd: &[u8] = &pos_rd_buf;
            assert_eq!(pos_rd.read_be_u24().unwrap(), 0x012345_u32);
        }

        let neg_rd_buf = [0xFE, 0xDC, 0xBA + 1];
        let mut neg_rd: &[u8] = &neg_rd_buf;
        assert_eq!(neg_rd.read_be_i24().unwrap(), (-0x012345) as i32);

        // Write
        let mut wr_buf = [0u8; 3];
        {
            let mut pos_wr = &mut wr_buf[..];
            write_be_u24_slice(pos_wr, 0x012345_u32).unwrap();
        }
        assert_eq!(wr_buf, pos_rd_buf);
        {
            let mut pos_wr = &mut wr_buf[..];
            write_be_u24_step(&mut pos_wr, 0x012345_u32).unwrap();
        }
        assert_eq!(wr_buf, pos_rd_buf);
        {
            let mut pos_wr = &mut wr_buf[..];
            write_be_u24_once(&mut pos_wr, 0x012345_u32).unwrap();
        }
        assert_eq!(wr_buf, pos_rd_buf);
        {
            let mut pos_wr = &mut wr_buf[..];
            pos_wr.write_be_u24(0x012345_u32).unwrap();
        }
        assert_eq!(wr_buf, pos_rd_buf);

        // i24
        {
            let mut neg_wr = &mut wr_buf[..];
            neg_wr.write_be_i24((-0x012345) as i32).unwrap();
        }
        assert_eq!(wr_buf, neg_rd_buf);
    }

    const N_NUMBERS: usize = 50000;
    const BUFFER_LEN: usize = N_NUMBERS * 3;
    lazy_static! {
        static ref BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(BUFFER_LEN));
        static ref NUMBERS: Mutex<Vec<u32>> = Mutex::new(Vec::with_capacity(N_NUMBERS));
    }

    fn setup() {
        let mut rng = rand::thread_rng();

        let mut buffer = BUFFER.lock().unwrap();
        if buffer.is_empty() {
            unsafe {
                let len = buffer.capacity();
                buffer.set_len(len);
            }
            rng.fill_bytes(&mut buffer);
        }

        let mut numbers = NUMBERS.lock().unwrap();
        if numbers.is_empty() {
            for _ in 0..N_NUMBERS {
                numbers.push(rng.next_u32() & 0x00FF_FFFF);
            }
        }
    }

    // Benchmark of Reading

    #[bench]
    fn bench_read_u24_safe_slice(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for _ in 0..N_NUMBERS {
               let _ = read_be_u24_slice(&mut &buffer[i..i+3]).unwrap();
               i += 3;
            }
        });
    }

    #[bench]
    fn bench_read_u24_safe_trait_step(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for _ in 0..N_NUMBERS {
               let _ = read_be_u24_step(&mut &buffer[i..i+3]).unwrap();
               i += 3;
            }
        });
    }

    #[bench]
    fn bench_read_u24_safe_trait_step_notry(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for _ in 0..N_NUMBERS {
               let _ = read_be_u24_step_notry(&mut &buffer[i..i+3]).unwrap();
               i += 3;
            }
        });
    }

    #[allow(non_snake_case)]
    #[bench]
    fn bench_read_u24_safe_trait_once__index(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for _ in 0..N_NUMBERS {
               let _ = read_be_u24_once(&mut &buffer[i..i+3]).unwrap();
               i += 3;
            }
        });
    }

    #[allow(non_snake_case)]
    #[bench]
    fn bench_read_u24_safe_trait_once__rd(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let rd = &mut &buffer[..];
            for _ in 0..N_NUMBERS {
               let _ = read_be_u24_once(rd).unwrap();
            }
        });
    }

    #[bench]
    fn bench_read_u24_unsafe_trait(b: &mut Bencher) {
        setup();
        let buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let rd = &mut &buffer[..];
            for _ in 0..N_NUMBERS {
                let _ = rd.read_be_u24().unwrap();
            }
        });
    }

    // Benchmark of Writing

    #[bench]
    fn bench_write_u24_safe_slice(b: &mut Bencher) {
        setup();
        let numbers = NUMBERS.lock().unwrap();
        let mut buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for n in 0..N_NUMBERS {
               let _ = write_be_u24_slice(&mut buffer[i..i+3], numbers[n]).unwrap();
               i += 3;
            }
        });
    }

    #[bench]
    fn bench_write_u24_safe_trait_step(b: &mut Bencher) {
        setup();
        let numbers = NUMBERS.lock().unwrap();
        let mut buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for n in 0..N_NUMBERS {
               let _ = write_be_u24_step(&mut &mut buffer[i..i+3], numbers[n]).unwrap();
               i += 3;
            }
        });
    }

    #[bench]
    fn bench_write_u24_safe_trait_step_notry(b: &mut Bencher) {
        setup();
        let numbers = NUMBERS.lock().unwrap();
        let mut buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let mut i = 0;
            for n in 0..N_NUMBERS {
               let _ = write_be_u24_step_notry(&mut &mut buffer[i..i+3], numbers[n]).unwrap();
               i += 3;
            }
        });
    }

    #[bench]
    fn bench_write_u24_safe_trait_once(b: &mut Bencher) {
        setup();
        let numbers = NUMBERS.lock().unwrap();
        let mut buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let wr = &mut &mut buffer[..];
            for n in 0..N_NUMBERS {
               let _ = write_be_u24_once(wr, numbers[n]).unwrap();
            }
        });
    }

    #[bench]
    fn bench_write_u24_unsafe_trait(b: &mut Bencher) {
        setup();
        let numbers = NUMBERS.lock().unwrap();
        let mut buffer = BUFFER.lock().unwrap();

        b.iter(|| {
            let wr = &mut &mut buffer[..];
            for n in 0..N_NUMBERS {
                let _ = wr.write_be_u24(numbers[n]).unwrap();
            }
        });
    }
}
