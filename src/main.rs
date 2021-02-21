use bincode::deserialize;
use bytes::{Buf, BytesMut};
use serde::{Deserialize, Serialize};
use std::io;
use tokio_util::codec::Decoder;

const MAX: usize = 8196 * 1024;

struct Payload<'a, T: Deserialize<'a>> {
    _type: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Deserialize<'a>> Decoder for Payload<'a, T> {
    type Item = T;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length > MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame too large",
            ));
        }

        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());

            return Ok(None);
        }

        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let a1: Result<Self::Item, _> = deserialize(&data);
        match a1 {
            Ok(res) =>
                return Ok(Some(res)),
            Err(_) => panic!(),
        }
    }
}

fn main() {
    println!("Hello, world!");
}
