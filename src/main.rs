use anyhow::Result;
use redis_starter_rust::process::processor;
use std::net::TcpListener;
use std::thread;

const IP: &str = "127.0.0.1";
const PORT: &str = "6379";

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind(format!("{IP}:{PORT}"))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread::spawn(move || match processor(stream) {
                    Ok(_) => println!("Success"),
                    Err(e) => eprintln!("failed: {e}"),
                });
            }
            Err(e) => eprintln!("Failed to accept new connection: {}", e),
        }
    }

    Ok(())
}
