use crate::database::DataBase;
use crate::redis_message::{RedisCommand, COMMANDS};
use crate::BUFFER_SIZE;
use anyhow::Result;
use std::sync::Arc;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn processor(mut stream: TcpStream, database: Arc<DataBase>) -> Result<()> {
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
                    COMMANDS::GET => {
                        if command.tokens.len() == 0 || command.tokens.len() > 1 {
                            error("get - invalid number of tokens", &mut stream)?;
                        } else {
                            let val = database.get(command.tokens[0].clone());
                            result(&val, &mut stream)?
                        }
                    }
                    COMMANDS::SET => {
                        if command.tokens.len() == 0 || command.tokens.len() > 2 {
                            error("set - invalid number of tokens", &mut stream)?;
                        } else {
                            database.set(command.tokens[0].clone(), command.tokens[1].clone());
                            result("OK", &mut stream)?
                        }
                    }
                    COMMANDS::ECHO => echo(&command.tokens[0], &mut stream)?,
                    COMMANDS::INVALID => invalid_command(&mut stream)?,
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
    let s = format!("+{}\r\n", token);
    stream.write(s.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn result(msg: &str, stream: &mut TcpStream) -> Result<()> {
    let s = format!("+{msg}\r\n");
    stream.write(s.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn error(msg: &str, stream: &mut TcpStream) -> Result<()> {
    let s = format!("-{msg}\r\n");
    stream.write(s.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn invalid_command(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"-INVALID COMMAND\r\n")?;
    stream.flush()?;
    Ok(())
}
