use std::io;

use std::str;
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
        let s = str::from_utf8(&buffer).unwrap();
        println!("{s}");
        let message = parse_message(s);
        println!("{s}");
        let lines: Vec<&str> = s.split("\r\n").collect();
        let n_lines = lines[0];

        if lines[2] == "ECHO" {
            stream
                .write_all((format!("+ECHO {}\r\n", lines[4])).as_bytes())
                .await
                .unwrap();
        } else {
            stream.write_all(b"+PONG\r\n").await.unwrap();
        }
        stream.flush().await.unwrap();
        println!("read: {read_bytes}, {s}");
    }
}

struct Message {
    command: String,
    content: String,
}

fn parse_message(input: &str) -> Message {
    let lines: Vec<&str> = input.split("\r\n").collect();
    if lines.len() == 1 {
        Message {
            command: String::from(lines[0]),
            content: String::new(),
        }
    } else {
        let n_commands: i32 = str::parse(&String::from(lines[0])[1..]).unwrap();
        let command_len: i32 = str::parse(&String::from(lines[1])[1..]).unwrap();
        let command = lines[2];
        let content_len: i32 = str::parse(&String::from(lines[3])[1..]).unwrap();
        let content = lines[4];

        Message {
            command: command.to_owned(),
            content: content.to_owned(),
        }
    }
}
