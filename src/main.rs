mod payload;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::{Decoder, Encoder};

use payload::Payload;

#[derive(Debug, Serialize, Deserialize)]
enum Frame {
    Version(u32, u32, u32),
    Message(Vec<usize>),
    Bye,
}

// what do we do
// we do replies and responses
// the client starts with a version
// error handling is outsourced completely
#[tokio::main]
async fn main() {
    if std::env::args().any(|arg| arg == "--serve") {
        println!("server > ");
        let listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("could not bind socket.");

        loop {
            if let Ok((socket, _)) = listener.accept().await {
                print!("serving... ");
                if let Err(e) = process(socket).await {
                    println!("err: {}", e);
                }
            }
        }
    } else {
        print!("client > ");
        if let Ok(stream) = TcpStream::connect("127.0.0.1:8080").await {
            process_client(stream).await;
        } else {
            println!("could not connect.")
        }
    }
}

async fn process(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = bytes::BytesMut::new();
    let mut payload = Payload::new();

    let frame = loop {
        if 0 == socket.read_buf(&mut buf).await? {
            // EOF
            break Frame::Bye;
        }

        match payload.decode(&mut buf) {
            Ok(Some(frame)) => break frame,
            Ok(None) => continue,
            Err(e) => return Err(Box::new(e)),
        }
    };

    if let Frame::Message(msg) = frame {
        println!("len: {}", msg.len());
    }

    Ok(())
}

async fn process_client(mut stream: TcpStream) {
    let mut buf = bytes::BytesMut::new();
    let mut payload = Payload::new();
    if let Err(e) = payload.encode(Frame::Message(vec![0; 300000]), &mut buf) {
        println!("uh oh. {}", e);
        return;
    }

    match stream.write_buf(&mut buf).await {
        Ok(n) => println!("client: wrote {} to stream", n),
        Err(e) => println!("client: error: {}", e),
    }
}
