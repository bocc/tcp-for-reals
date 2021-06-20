mod conn;
mod payload;

use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};

use conn::Conn;
use payload::Payload;

#[derive(Debug, Serialize, Deserialize)]
enum Frame {
    Version(u32),
    Message(String),
    Bye,
}

// what do we do
// we do replies and responses
// then we implement a basic protocol with some context, that can work
// as a state machine to handle communications
// then we generate our protocol from a specification at build time, allowing
// Rust to validate it
#[tokio::main]
async fn main() {
    if std::env::args().any(|arg| arg == "--serve") {
        println!("server > ");
        let listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("could not bind socket.");

        loop {
            if let Ok((stream, _)) = listener.accept().await {
                print!("serving... ");
                process_server(stream).await;
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

async fn process_server(stream: TcpStream) {
    let mut conn: Conn<Frame> = Conn::new(stream);

    loop {
        match conn::receive(&mut conn).await {
            Ok(frame) => match frame {
                Frame::Version(v) => println!("got version: {}", v),
                Frame::Message(m) => println!("got message: {}", m),
                Frame::Bye => {
                    println!("client closed connection");
                    break;
                }
            },
            Err(e) => {
                eprintln!("err: {}", e);
                break;
            }
        }
    }
}

async fn process_client(stream: TcpStream) {
    let mut conn: Conn<Frame> = Conn::new(stream);

    let frames = vec![
        Frame::Version(64),
        Frame::Message("szevasz".to_string()),
        Frame::Bye,
    ];

    for frame in frames.into_iter() {
        if let Err(e) = conn::send(&mut conn, frame).await {
            eprintln!("could not send frame: {}", e);
        }
    }
}
