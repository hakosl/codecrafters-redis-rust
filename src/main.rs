use std::io;
use std::time::Duration;
use std::time::Instant;

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

async fn process_request(mut stream: TcpStream, state: Arc<Mutex<HashMap<String, Entry>>>) {
    loop {
        let mut buffer = [0; 512];
        let read_bytes = stream.read(&mut buffer).await.unwrap();
        if read_bytes == 0 {
            return;
        }
        let s = str::from_utf8(&buffer).unwrap();
        println!("{s}");
        let lines: Vec<&str> = s.split("\r\n").collect();
        println!("lines length {}", lines.len());
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
                let expiry = if lines.len() > 9 {
                    let second_option = lines[8].to_uppercase();
                    match second_option.as_str() {
                        "PX" => {
                            let expiry_str = &lines[10];
                            println!("expiry str: {}", expiry_str);
                            Some(expiry_str.parse::<u64>().unwrap())
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                set_value(state.clone(), key, value, expiry).unwrap();
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
                        stream.write_all(b"$-1\r\n").await.unwrap();
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

struct Entry {
    value: String,
    expiry: Option<Instant>,
}

fn set_value(
    state: Arc<Mutex<HashMap<String, Entry>>>,
    key: String,
    value: String,
    expiry: Option<u64>,
) -> Result<&'static [u8; 2]> {
    let mut locked_state = state.lock().unwrap();

    let val = match expiry {
        Some(exp) => Entry {
            value: value,
            expiry: Instant::now().checked_add(Duration::from_millis(exp)),
        },
        None => Entry {
            value: value,
            expiry: None,
        },
    };
    locked_state.insert(key, val);

    Ok(b"OK")
}

fn get_value(state: Arc<Mutex<HashMap<String, Entry>>>, key: String) -> Result<Vec<u8>, String> {
    let locked_state = state.lock().unwrap();

    match locked_state.get(&key) {
        Some(val) => {
            match val.expiry {
                Some(expiry) => {
                    println!("expiry set");
                    if expiry < Instant::now() {
                        return Err("-Not found".to_string());
                    }
                }
                _ => {
                    println!("Expiry not set")
                }
            }
            let entry = val.clone();
            Ok(("+".to_owned() + &entry.value + "\r\n").into_bytes())
        }
        None => Err("-Not found".to_string()),
    }
}
