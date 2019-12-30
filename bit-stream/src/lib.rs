#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use std::borrow::Cow;
use std::cmp;
use std::num::NonZeroU8;

mod bitstream;
mod bytestream;

pub use bitstream::*;
pub use bytestream::*;

#[derive(Clone, Debug)]
pub struct BitSlice<'a> {
    data: Cow<'a, [u8]>,
    start_bit: usize,
    end_bit: usize,
}

impl<'a> BitSlice<'a> {
    pub fn new<T>(data: T, start_bit: usize, bit_len: Option<usize>) -> Self
        where Cow<'a, [u8]>: From<T>
    {
        let data = Cow::from(data);
        let end_bit = Self::get_end_bit(&data, start_bit, bit_len).expect("invalid arguments");

        BitSlice {
            data,
            start_bit,
            end_bit,
        }
    }

    /// Initializes a `BitSlice` by taking the `[start_bit, end_bit)` bits from `data: &[u8]`.
    pub fn from_slice_owned(data: &[u8], start_bit: usize, end_bit: usize) -> Self {
        let skip_start_len = start_bit / 8;
        let skip_end_len = (data.len() * 8 - end_bit) / 8;
        Self {
            data: Cow::from((&data[skip_start_len .. (data.len() - skip_end_len)]).to_vec()),
            start_bit: start_bit - skip_start_len * 8,
            end_bit: end_bit - skip_end_len * 8,
        }
    }

    pub fn from_partial_byte(pb: PartialByte) -> Self {
        Self {
            data: Cow::from(vec![pb.value]),
            start_bit: 0,
            end_bit: pb.bit_len() as usize,
        }
    }

    #[inline]
    fn get_end_bit(data: &[u8], start_bit: usize, bit_len: Option<usize>) -> Option<usize> {
        let data_bit_len = data.len() * 8;
        if start_bit > data_bit_len {
            return None;
        }

        let end_bit = match bit_len {
            Some(bit_len) => {
                start_bit + bit_len
            }
            None => data_bit_len
        };
        if end_bit > data_bit_len {
            return None;
        }

        Some(end_bit)
    }

    pub fn bit_len(&self) -> usize {
        self.end_bit - self.start_bit
    }

    /// Reads a (potentially partial) byte.
    ///
    /// # Parameters
    ///
    /// - `from_bit`: the start bit to read.
    /// - `len`: the length of bits to read. If `None` is given, the length defaults to 8.
    #[inline]
    pub fn read_byte(&self, start_bit: usize, len: Option<usize>) -> ReadByte {
        let from_bit = start_bit;
        let to_bit = {
            let len = len.unwrap_or(8);
            debug_assert!(len <= 8, "Can at most read one byte");
            cmp::min(from_bit + len, self.end_bit)
        };
        if from_bit == to_bit {
            return ReadByte::Partial(
                PartialByte::new(0, 8)
            );
        }
        let partial: bool = from_bit + 8 != to_bit;

        let from_byte = from_bit / 8;
        let to_byte = to_bit.saturating_sub(1) / 8;
        let left_shift = from_bit % 8;

        let mut val = self.data[from_byte] << left_shift;
        if from_byte != to_byte {
            val |= self.data[to_byte] >> (8 - left_shift);
        }

        if partial {
            let n_tailing_zeros = from_bit + 8 - to_bit;
            ReadByte::Partial(PartialByte::new(val, n_tailing_zeros as u8))
        } else {
            ReadByte::Complete(val)
        }
    }
}

#[derive(Debug)]
pub enum ReadByte {
    Complete(u8),
    Partial(PartialByte),
}

// ===== PartialByte =====
#[derive(Clone, Copy, Debug)]
pub struct PartialByte {
    value: u8,
    n_paddings: NonZeroU8,
}

impl PartialByte {
    pub fn new(value: u8, n_paddings: u8) -> PartialByte {
        debug_assert!(n_paddings > 0, "Try to initialize a `PartialByte` which is indeed complete");
        debug_assert!(n_paddings <= 8, "`n_paddings` exceeds 8");

        PartialByte {
            value: value & (!0_u8).wrapping_shl(n_paddings.into()), // normalizes it
            n_paddings: NonZeroU8::new(n_paddings as u8).unwrap(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.n_paddings.get() == 8
    }
}

impl PartialByte {
    #[inline]
    pub fn bit_len(&self) -> u8 {
        8 - self.n_paddings.get()
    }

    pub fn merge(&self, other: &Self) -> Option<ReadByte> {
        let bl = self.bit_len();
        let new_value = self.value | (other.value >> bl);
        match other.n_paddings.get().checked_sub(bl) {
            Some(0) => Some(ReadByte::Complete(new_value)),
            Some(n) => Some(
                ReadByte::Partial(PartialByte {
                    value: new_value,
                    n_paddings: NonZeroU8::new(n).unwrap(),
                })
            ),
            None => None,
        }
    }

    #[inline]
    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn shl1(&mut self) {
        self.value <<= 1;
        self.n_paddings = unsafe { NonZeroU8::new_unchecked(self.n_paddings.get() + 1) };
    }
}

fn bit_len_of_bit_slices(bs_list: &[BitSlice], cur_slice: usize, cur_bit: usize) -> usize {
    let remaining_slice = &bs_list[cur_slice..];
    if remaining_slice.is_empty() {
        return 0;
    }
    let n_slice_bits_read = cur_bit - remaining_slice[0].start_bit;
    remaining_slice
        .iter()
        .map(|bs| bs.bit_len() as usize)
        .sum::<usize>()
        - n_slice_bits_read
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let data1: &[u8] = &[0b0001_1111, 0b1111_1111, 0b1111_1000];
        let bs_all_ones = BitSlice::new(data1, 3, Some(18));

        let data2: Vec<u8> = vec![0xFF, 0, 0, 0];
        let bs_all_zeros = BitSlice::new(data2, 8, None);

        let mut bytes = ByteStream::new(vec![bs_all_ones, bs_all_zeros]);
        assert_eq!(bytes.next(), Some(0xFF));
        assert_eq!(bytes.next(), Some(0xFF));
        let mut bits: BitStream = bytes.into(); // switches to bit stream
        assert_eq!(bits.next(), Some(true));

        let bytes: ByteStream = bits.into(); // switches to byte stream
        assert_eq!(bytes.collect::<Vec<_>>(), [0b1000_0000, 0, 0]);
    }
}
