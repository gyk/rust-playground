use std::io::{self, Write, Result};

use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ReadBytesExt};
use tokio_io::codec::{Encoder, Decoder};

pub struct LengthPrefixCodec;

impl Decoder for LengthPrefixCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<BytesMut>> {
        if buf.len() < 4 {
            return Ok(None);
        }
        let payload_len = {
            let mut rd = buf.as_ref();
            rd.read_u32::<BigEndian>()? as usize
        };
        assert!(payload_len > 0);

        if buf.len() < payload_len {
            return Ok(None);
        }
        buf.split_to(4);
        let mut payload = BytesMut::with_capacity(payload_len);

        // Delibrately copy bytes
        payload.extend_from_slice(&buf.split_to(payload_len));
        Ok(Some(payload))
    }
}

impl Encoder for LengthPrefixCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn encode(&mut self, message: BytesMut, buf: &mut BytesMut) -> Result<()> {
        let payload_len = message.len();
        buf.reserve(payload_len + 4);
        buf.put_u32::<BigEndian>(payload_len as u32);
        unsafe {
            let mut wr = buf.bytes_mut();
            wr.write_all(&message)?;
        }
        unsafe {
            buf.advance_mut(payload_len);
        }
        Ok(())
    }
}
