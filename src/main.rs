use redis_starter_rust::process::processor;
use std::net::TcpListener;

fn main() {
    println!("**Logs from your program will appear here!**");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                match processor(stream) {
                    Ok(_) => println!("processed connection"),
                    Err(e) => eprint!("failed to process connection: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to accept new connection: {}", e);
            }
        }
    }
}
