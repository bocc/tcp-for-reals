use bincode::deserialize;
use bytes::{Buf, BytesMut};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::{io, marker::PhantomData};
use tokio_util::codec::Decoder;

const MAX: usize = 8196 * 1024;

struct Payload<T: DeserializeOwned>(PhantomData<T>);

impl<T: DeserializeOwned> From<T> for Payload<T> {
    fn from(_: T) -> Self {
        Payload::<T>(PhantomData)
    }
}

#[derive(Debug, Deserialize)]
struct Example {
    field: String,
}

impl<T: DeserializeOwned> Decoder for Payload<T> {
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

        match deserialize(&data) {
            Ok(res) => Ok(Some(res)),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "couldn't digest input",
            )),
        }
    }
}

fn main() {
    let b1 = b"asdflkj";
    let mut b2 = BytesMut::from(b1.as_ref());
    let a3 = Payload::<Example>(PhantomData).decode(&mut b2);
    println!("{:?}", &a3);
}
