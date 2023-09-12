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


        if lines[2].to_uppercase() == "ECHO" {
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
        let n_commands: usize = str::parse(&String::from(lines[0])[1..]).unwrap();
        println!("n_commands: {n_commands}");
        let mut array_sizes: Vec<i32> = vec![0; n_commands]; 


        for i in 0..n_commands {
            let line = &lines[i * 2 + 1][1..];
            println!("n_commands: {line}");
            array_sizes[i] = str::parse(line).unwrap(); 
        }

        let mut content: Vec<String> = vec![String::new(); n_commands];

        for i in 0..n_commands {
            let line = lines[(i + 1) * 2];
            println!("commands: {line}");
            content[i] = line.to_string();
        }
        if n_commands == 1 {
            return Message {
                command: content[0].to_owned(),
                content: String::new(), 
            };
        }
        Message {
            command: content[0].to_owned(),
            content: content[1].to_owned(),
        }
    }
}
