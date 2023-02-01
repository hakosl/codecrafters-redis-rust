use std::io::{self, prelude::*};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_request(socket).await });
    }
}

async fn process_request(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 512];
        let read_bytes = stream.read(&mut buffer).await.unwrap();
        if read_bytes == 0 {
            return;
        }
        stream.write_all(b"+PONG\r\n").await.unwrap();
        stream.flush().await.unwrap();
        println!("read: {read_bytes}, ");
    }
}
