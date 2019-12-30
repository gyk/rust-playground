use std::io::{Result, Read, Write};

use super::*;

#[derive(Debug)]
pub struct ByteStream<'a> {
    pub(crate) bs_list: Vec<BitSlice<'a>>,
    pub(crate) cur_slice: usize, // the current index of `BitSlice` to read
    pub(crate) cur_bit: usize, // the current bit position of the slice
}

impl<'a> ByteStream<'a> {
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

impl<'a> From<BitStream<'a>> for ByteStream<'a> {
    fn from(bitstream: BitStream<'a>) -> Self {
        Self {
            bs_list: bitstream.bs_list,
            cur_slice: bitstream.cur_slice,
            cur_bit: bitstream.cur_bit,
        }
    }
}

impl<'a> Iterator for ByteStream<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let mut leftover: Option<PartialByte> = None;

        while self.cur_slice < self.bs_list.len() {
            let bs = &self.bs_list[self.cur_slice];
            let next_bit;

            let a_byte = match leftover.take() {
                Some(old_pb) => {
                    let n_remaining = old_pb.n_paddings.get() as usize;
                    let new_pb = match bs.read_byte(self.cur_bit, Some(n_remaining)) {
                        ReadByte::Complete(..) => unreachable!(),
                        ReadByte::Partial(new_pb) => new_pb,
                    };
                    next_bit = self.cur_bit + new_pb.bit_len() as usize;
                    let merged = old_pb.merge(&new_pb).expect("PartialByte merge overflow");
                    merged
                }
                None => {
                    next_bit = self.cur_bit + 8;
                    bs.read_byte(self.cur_bit, None)
                }
            };

            match a_byte {
                ReadByte::Complete(complete_byte) => {
                    self.cur_bit = next_bit;
                    return Some(complete_byte);
                }
                ReadByte::Partial(partial_byte) => {
                    leftover = if partial_byte.is_empty() {
                        None
                    } else {
                        Some(partial_byte)
                    };
                    self.cur_slice += 1;
                    self.cur_bit = self.bs_list.get(self.cur_slice).map_or(0, |s| s.start_bit);
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bit_len(), None)
    }
}

impl<'a> Read for ByteStream<'a> {
    // TODO: optimization
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        let buf_len = buf.len();
        let mut read_len = 0;
        let wr = &mut buf;
        for b in Iterator::take(self, buf_len) {
            wr.write_all(&[b])?;
            read_len += 1;
        }
        Ok(read_len)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn prop_two_bytestreams(xs: Vec<u8>, start_bit: usize, bit_len1: usize, bit_len2: usize)
            -> bool
        {
            if let None = BitSlice::get_end_bit(&xs, start_bit, Some(bit_len1 + bit_len2)) {
                return true; // invalid input, ignored.
            }

            let bs = BitSlice::new(&xs, start_bit, Some(bit_len1 + bit_len2));
            assert_eq!(bs.bit_len(), bit_len1 + bit_len2);
            let all_bytes: Vec<u8> = ByteStream::new(vec![bs]).collect();

            let bs1 = BitSlice::new(&xs, start_bit, Some(bit_len1));
            let bs2 = BitSlice::new(&xs, start_bit + bit_len1, Some(bit_len2));
            let concated_bytes: Vec<u8> = ByteStream::new(vec![bs1, bs2]).collect();

            all_bytes == concated_bytes
        }
    }
}
