use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn processor(mut stream: TcpStream) {
    loop {
        let mut buf = vec![0; 512];
        match stream.read(&mut buf) {
            Ok(_) => match pong(&mut stream) {
                Ok(_) => println!("Successful pong"),
                Err(e) => eprintln!("failed to pong: {}", e),
            },
            Err(e) => {
                eprint!("Error with stream: {}", e);
                break;
            }
        }
    }
}

fn pong(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"+PONG\r\n")?;
    stream.flush()?;
    Ok(())
}
