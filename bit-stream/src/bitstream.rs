use std::convert::From;

use super::*;

// Most significant bit
const MSB: u8 = 0b1000_0000;

#[derive(Debug)]
pub struct BitStream<'a> {
    pub(crate) bs: &'a BStream<'a>,
    pub(crate) cur_bit: usize, // the current bit position of `bs`
    pub(crate) leftover: Option<PartialByte>, // mostly `None` unless converted from `ByteStream`
}

impl<'a> BitStream<'a> {
    pub fn new(bs: &'a BStream<'a>) -> Self {
        Self {
            bs,
            cur_bit: bs.start_bit,
            leftover: None,
        }
    }

    pub fn bit_len(&self) -> usize {
        self.leftover.map_or(0, |pb| pb.bit_len()) as usize + self.bs.bit_len()
    }
}

impl<'a> From<ByteStream<'a>> for BitStream<'a> {
    fn from(bytestream: ByteStream<'a>) -> Self {
        Self {
            bs: bytestream.bs,
            cur_bit: bytestream.cur_bit,
            leftover: bytestream.leftover,
        }
    }
}

impl<'a> Iterator for BitStream<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if let Some(mut pb) = self.leftover.take() {
            debug_assert!(pb.bit_len() >= 1);
            let bit = pb.value & MSB != 0;
            pb.shl1();
            if pb.bit_len() > 0 {
                self.leftover = Some(pb);
            }
            return Some(bit);
        }

        let bs = self.bs;
        debug_assert!(self.cur_bit >= bs.start_bit && self.cur_bit <= bs.end_bit);
        if self.cur_bit == bs.end_bit {
            let new_bs = match bs.next {
                Some(ref next_bs) => next_bs,
                None => return None,
            };
            *self = BitStream::new(new_bs);
            self.next()
        } else {
            let cur_byte = self.cur_bit / 8;
            let cur_bit_in_byte = self.cur_bit % 8;
            self.cur_bit += 1;
            Some(bs.data[cur_byte] & (MSB >> cur_bit_in_byte) != 0)
        }
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
            if let None = BStream::get_end_bit(&xs, start_bit, Some(bit_len1 + bit_len2)) {
                return true; // invalid input, ignored.
            }

            let bs = BStream::new(&xs, start_bit, Some(bit_len1 + bit_len2));
            let all_bits: Vec<bool> = BitStream::new(&bs).collect();

            let bs1 = BStream::new(&xs, start_bit, Some(bit_len1));
            let bs2 = Box::new(BStream::new(&xs, start_bit + bit_len1, Some(bit_len2)));
            let mut bs_concat = bs1;
            bs_concat.append(bs2);
            let concated_bits: Vec<bool> = BitStream::new(&bs_concat).collect();

            all_bits == concated_bits
        }
    }
}
