use bincode::deserialize;
use bytes::{Buf, BytesMut};
use serde::{de::DeserializeOwned, Serialize};
use std::{io, marker::PhantomData};
use tokio_util::codec::{Decoder, Encoder};

const MAX: usize = 8196 * 1024;

#[derive(Default)]
pub(super) struct Payload<T>(pub(super) PhantomData<T>);

impl<T> Payload<T> {
    pub(super) fn new() -> Self {
        Payload::<T>(PhantomData)
    }
}

impl<T: DeserializeOwned> From<T> for Payload<T> {
    fn from(_: T) -> Self {
        Payload::<T>(PhantomData)
    }
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

impl<T: Serialize> Encoder<T> for Payload<T> {
    type Error = std::io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = bincode::serialize(&item)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame too large"))?;

        if encoded.len() > MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("frame length is too large: {}", encoded.len()),
            ));
        }

        let len_slice = u32::to_le_bytes(encoded.len() as u32);

        dst.reserve(4 + encoded.len());

        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}
