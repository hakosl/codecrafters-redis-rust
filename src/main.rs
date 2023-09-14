use std::io;

use anyhow::Error;
use anyhow::Result;
use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
#[tokio::main]
async fn main() -> io::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let state = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await?;
        let local_state = state.clone();
        tokio::spawn(async move { process_request(socket, local_state).await });
    }
}

async fn process_request(mut stream: TcpStream, state: Arc<Mutex<HashMap<String, String>>>) {
    loop {
        let mut buffer = [0; 512];
        let read_bytes = stream.read(&mut buffer).await.unwrap();
        if read_bytes == 0 {
            return;
        }
        let s = str::from_utf8(&buffer).unwrap();
        println!("{s}");
        let lines: Vec<&str> = s.split("\r\n").collect();
        let command = lines[2].to_uppercase();

        match command.as_str() {
            "ECHO" => {
                stream
                    .write_all((format!("+{}\r\n", lines[4])).as_bytes())
                    .await
                    .unwrap();
            }
            "SET" => {
                let key = lines[4].clone().to_string();
                let value = lines[6].clone().to_string();
                set_value(state.clone(), key, value).unwrap();
                stream.write_all(b"+OK\r\n").await.unwrap();
            }
            "GET" => {
                println!("getting");
                let key = lines[4].clone().to_string();
                println!("getting {key}");
                let response = get_value(state.clone(), key);
                match response {
                    Ok(resp) => {
                        println!("{}", str::from_utf8(&resp).unwrap());
                        stream.write_all(&resp).await.unwrap();
                    }
                    Err(err) => {
                        println!("{}", str::from_utf8(err.as_bytes()).unwrap());
                        stream.write_all(b"-Not found\r\n").await.unwrap();
                    }
                };
                // println!("{}", str::from_utf8(val.clone()).unwrap());
            }
            _ => {
                stream.write_all(b"+PONG\r\n").await.unwrap();
            }
        };
        println!("outside");
        stream.flush().await.unwrap();
        println!("read: {read_bytes}, {s}");
    }
}

fn set_value(
    state: Arc<Mutex<HashMap<String, String>>>,
    key: String,
    value: String,
) -> Result<&'static [u8; 2]> {
    let mut locked_state = state.lock().unwrap();

    locked_state.insert(key, value);

    Ok(b"OK")
}

fn get_value(state: Arc<Mutex<HashMap<String, String>>>, key: String) -> Result<Vec<u8>, String> {
    let locked_state = state.lock().unwrap();

    match locked_state.get(&key) {
        Some(val) => {
            let value = val.clone();
            Ok(("+".to_owned() + &value + "\r\n").into_bytes())
        }
        None => Err("-Not found".to_string()),
    }
}
