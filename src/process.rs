use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn processor(mut stream: TcpStream) -> Result<()> {
    let mut buf = vec![0; 30];
    stream.read(&mut buf)?;

    let s = String::from_utf8(buf)?;
    let s = s.trim_matches('\0');
    let tokens: Vec<&str> = s.split("\r\n").collect();

    println!("{:?}", tokens);

    stream.write(b"+PONG\r\n")?;
    Ok(())
}
