use std::convert::From;

use super::*;

// Most significant bit
const MSB: u8 = 0b1000_0000;

#[derive(Debug)]
pub struct BitStream<'a> {
    pub(crate) bs_list: Vec<BitSlice<'a>>,
    pub(crate) cur_slice: usize, // the current index of `BitSlice` to read
    pub(crate) cur_bit: usize, // the current bit position of the slice
}

impl<'a> BitStream<'a> {
    pub fn new(bs_list: Vec<BitSlice<'a>>) -> Self {
        assert!(!bs_list.is_empty());
        let cur_bit = bs_list[0].start_bit;
        Self {
            bs_list,
            cur_slice: 0,
            cur_bit,
        }
    }

    pub fn bit_len(&self) -> usize {
        bit_len_of_bit_slices(&self.bs_list, self.cur_slice, self.cur_bit)
    }

    pub fn into_inner(self) -> Vec<BitSlice<'a>> {
        self.bs_list
    }
}

impl<'a> From<ByteStream<'a>> for BitStream<'a> {
    fn from(bytestream: ByteStream<'a>) -> Self {
        Self {
            bs_list: bytestream.bs_list,
            cur_slice: bytestream.cur_slice,
            cur_bit: bytestream.cur_bit,
        }
    }
}

impl<'a> Iterator for BitStream<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        while self.cur_slice < self.bs_list.len() {
            let bs = &self.bs_list[self.cur_slice];
            debug_assert!(self.cur_bit >= bs.start_bit && self.cur_bit <= bs.end_bit);
            if self.cur_bit == bs.end_bit {
                self.cur_slice += 1;
                self.cur_bit = self.bs_list.get(self.cur_slice).map_or(0, |s| s.start_bit);
            } else {
                let cur_byte = self.cur_bit / 8;
                let cur_bit_in_byte = self.cur_bit % 8;
                self.cur_bit += 1;
                return Some(bs.data[cur_byte] & (MSB >> cur_bit_in_byte) != 0)
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bit_len(), None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn prop_two_bitstreams(xs: Vec<u8>, start_bit: usize, bit_len1: usize, bit_len2: usize)
            -> bool
        {
            if let None = BitSlice::get_end_bit(&xs, start_bit, Some(bit_len1 + bit_len2)) {
                return true; // invalid input, ignored.
            }

            let bs = BitSlice::new(&xs, start_bit, Some(bit_len1 + bit_len2));
            let all_bits: Vec<bool> = BitStream::new(vec![bs]).collect();

            let bs1 = BitSlice::new(&xs, start_bit, Some(bit_len1));
            let bs2 = BitSlice::new(&xs, start_bit + bit_len1, Some(bit_len2));
            let concated_bits: Vec<bool> = BitStream::new(vec![bs1, bs2]).collect();

            all_bits == concated_bits
        }
    }
}
