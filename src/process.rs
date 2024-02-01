use crate::redis_message::{RedisCommand, COMMANDS};
use crate::BUFFER_SIZE;
use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn processor(mut stream: TcpStream) -> Result<()> {
    loop {
        let mut buf = [0; BUFFER_SIZE];
        match stream.read(&mut buf) {
            Ok(_) => {
                if buf[0] == 0 {
                    continue;
                }

                let command = RedisCommand::parse_request(&buf)?;
                match command.command {
                    COMMANDS::COMMAND => connected(&mut stream)?,
                    COMMANDS::PING => pong(&mut stream)?,
                    COMMANDS::ECHO => echo(&command.tokens[0], &mut stream)?,
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

fn echo(token: &str, stream: &mut TcpStream) -> Result<()> {
    let s = format!("+\"{}\"\r\n", token);
    stream.write(s.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn invalid_command(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"-INVALID COMMAND\r\n")?;
    stream.flush()?;
    Ok(())
}
