use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn processor(mut stream: TcpStream) -> Result<()> {
    loop {
        let mut buf = [0; 512];
        match stream.read(&mut buf) {
            Ok(_) => {
                if buf[0] == 0 {
                    continue;
                }

                let tokens = parse_request(&buf)?;
                match &tokens[0][..] {
                    "COMMAND" | "command" => connected(&mut stream)?,
                    "PING" | "ping" => pong(&mut stream)?,
                    "ECHO" | "echo" => echo(tokens[1..].join(" "), &mut stream)?,
                    _ => invalid_command(&mut stream)?,
                }
            }
            Err(e) => {
                eprint!("Error with stream: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn parse_request(request: &[u8; 512]) -> Result<Vec<String>> {
    let (mut index, mut bytes) = (1, vec![]);

    while index < 512 && request[index] as char != '\r' {
        bytes.push(request[index]);
        index += 1;
    }

    let token = String::from_utf8(bytes)?;
    let _size: usize = token.parse()?;
    bytes = vec![];
    index += 2;

    let mut tokens = vec![];
    if request[index] as char == '$' {
        index += 1;
        while index < 512 && request[index] as char != '\r' && request[index] != 0 {
            bytes.push(request[index]);
            index += 1;

            let token = String::from_utf8(bytes)?;
            let size: usize = token.parse()?;
            bytes = vec![];

            index += 2;

            let token = String::from_utf8(request[index..index + size].to_vec())?;
            tokens.push(token);
            index += size + 3;
        }
    }
    Ok(tokens)
}

fn connected(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"+CONNECTED\r\n")?;
    stream.flush()?;
    Ok(())
}

fn pong(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"+PONG\r\n")?;
    stream.flush()?;
    Ok(())
}

fn echo(token: String, stream: &mut TcpStream) -> Result<()> {
    let s = format!("+{}\r\n", token);
    stream.write(s.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn invalid_command(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"+invalid command\r\n")?;
    stream.flush()?;
    Ok(())
}
