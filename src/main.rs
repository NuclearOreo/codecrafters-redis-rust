use anyhow::Result;
use redis_starter_rust::{database::DataBase, process::processor};
use std::env::args;
use std::net::TcpListener;
use std::thread;

const IP: &str = "127.0.0.1";
const PORT: &str = "6379";

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");
    let (dir, filename) = parse_args();

    let listener = TcpListener::bind(format!("{IP}:{PORT}"))?;
    let database = DataBase::new(dir, filename);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let clone = database.clone();
                thread::spawn(|| match processor(stream, clone) {
                    Ok(_) => println!("Success"),
                    Err(e) => eprintln!("Failed: {e}"),
                });
            }
            Err(e) => eprintln!("Failed to accept new connection: {}", e),
        }
    }

    Ok(())
}

fn parse_args() -> (String, String) {
    let config_vals = args();
    let (mut dir_flag, mut dir) = (false, "".to_string());
    let (mut name_flag, mut name) = (false, "".to_string());

    for v in config_vals {
        if !dir_flag && v == "--dir".to_string() {
            dir_flag = true;
        } else if dir_flag && dir.is_empty() {
            dir = v;
        } else if !name_flag && v == "--dbfilename".to_string() {
            name_flag = true;
        } else if name_flag && name.is_empty() {
            name = v;
        }
    }

    (dir, name)
}
