#![allow(dead_code)]

use super::Payload;
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_util::codec::{Decoder, Encoder};

pub(crate) struct Conn<T>
where
    T: Serialize,
{
    buf: bytes::BytesMut,
    payload: Payload<T>,
    stream: TcpStream,
}

impl<T> Conn<T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(stream: TcpStream) -> Conn<T>
    where
        T: Serialize + DeserializeOwned,
    {
        Conn {
            buf: bytes::BytesMut::new(),
            payload: Payload::new(),
            stream,
        }
    }
}

pub(crate) async fn send<T>(comm: &mut Conn<T>, frame: T) -> io::Result<usize>
where
    T: Serialize + DeserializeOwned,
{
    comm.payload.encode(frame, &mut comm.buf)?;

    comm.stream.write_buf(&mut comm.buf).await
}

pub(crate) async fn receive<T>(comm: &mut Conn<T>) -> io::Result<T>
where
    T: Serialize + DeserializeOwned,
{
    loop {
        if 0 == comm.stream.read_buf(&mut comm.buf).await? && comm.buf.is_empty() {
            break Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }

        match comm.payload.decode(&mut comm.buf) {
            Ok(Some(frame)) => break Ok(frame),
            Ok(None) => continue,
            Err(e) => break Err(e),
        }
    }
}
