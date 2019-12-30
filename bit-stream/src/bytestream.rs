use std::io::{Result, Read, Write};

use super::*;

#[derive(Debug)]
pub struct ByteStream<'a> {
    pub(crate) bs: &'a BStream<'a>,
    pub(crate) cur_bit: usize, // the current bit position of `bs`
    pub(crate) leftover: Option<PartialByte>,
}

impl<'a> ByteStream<'a> {
    pub fn new(bs: &'a BStream<'a>) -> Self {
        Self {
            bs,
            cur_bit: bs.start_bit,
            leftover: None,
        }
    }

    pub fn bit_len(&self) -> usize {
        (self.leftover.map_or(0, |pb| pb.bit_len()) as usize + self.bs.bit_len() + 7) / 8
    }
}

impl<'a> From<BitStream<'a>> for ByteStream<'a> {
    fn from(bitstream: BitStream<'a>) -> Self {
        Self {
            bs: bitstream.bs,
            cur_bit: bitstream.cur_bit,
            leftover: bitstream.leftover,
        }
    }
}

impl<'a> Iterator for ByteStream<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        debug_assert!(self.leftover.is_none() || self.cur_bit == self.bs.start_bit);

        let bs = self.bs;
        let from_bit = self.cur_bit;
        let next_bit;
        let a_byte = match self.leftover.take() {
            Some(old_pb) => {
                let new_pb = match bs.read_byte(from_bit, Some(old_pb.n_paddings.get() as usize)) {
                    ReadByte::Complete(..) => panic!(),
                    ReadByte::Partial(new_pb) => new_pb,
                };
                next_bit =  self.cur_bit + new_pb.bit_len() as usize;
                let merged = old_pb.merge(&new_pb).expect("PartialByte merge overflow");
                merged
            }
            None => {
                next_bit =  self.cur_bit + 8;
                bs.read_byte(from_bit, None)
            }
        };

        match a_byte {
            ReadByte::Complete(complete_byte) => {
                self.cur_bit = next_bit;
                return Some(complete_byte);
            }
            ReadByte::Partial(partial_byte) => {
                let leftover = if partial_byte.is_empty() {
                    None
                } else {
                    Some(partial_byte)
                };
                let new_bs = match bs.next {
                    Some(ref next_bs) => next_bs,
                    None => return None,
                };
                *self = ByteStream::new(new_bs);
                self.leftover = leftover;
                self.next()
            }
        }
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
            if let None = BStream::get_end_bit(&xs, start_bit, Some(bit_len1 + bit_len2)) {
                return true; // invalid input, ignored.
            }

            let bs = BStream::new(&xs, start_bit, Some(bit_len1 + bit_len2));
            assert_eq!(bs.bit_len(), bit_len1 + bit_len2);
            let all_bytes: Vec<u8> = ByteStream::new(&bs).collect();

            let bs1 = BStream::new(&xs, start_bit, Some(bit_len1));
            let bs2 = Box::new(BStream::new(&xs, start_bit + bit_len1, Some(bit_len2)));
            let mut bs_concat = bs1;
            bs_concat.append(bs2);
            assert_eq!(bs_concat.bit_len(), bit_len1 + bit_len2);
            let concated_bytes: Vec<u8> = ByteStream::new(&bs_concat).collect();

            all_bytes == concated_bytes
        }
    }
}
