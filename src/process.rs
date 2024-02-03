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

                let redis_cmd = match RedisCommand::parse_request(&buf) {
                    Ok(v) => v,
                    Err(e) => {
                        error(
                            &format!("Failed to parse message: {}", e.to_string()),
                            &mut stream,
                        )?;
                        continue;
                    }
                };

                match redis_cmd.command {
                    COMMANDS::COMMAND => result("CONNECTED", &mut stream)?,
                    COMMANDS::PING => result("PONG", &mut stream)?,
                    COMMANDS::GET => {
                        let val = database.get(redis_cmd);
                        if val.is_empty() {
                            null(&mut stream)?
                        } else {
                            result(&val, &mut stream)?
                        }
                    }
                    COMMANDS::SET => {
                        database.set(redis_cmd);
                        result("OK", &mut stream)?
                    }
                    COMMANDS::ECHO => result(&redis_cmd.tokens[0], &mut stream)?,
                    COMMANDS::INVALID => error("INVALID COMMAND", &mut stream)?,
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

fn result(msg: &str, stream: &mut TcpStream) -> Result<()> {
    let s = format!("+{msg}\r\n");
    write(&s, stream)
}

fn null(stream: &mut TcpStream) -> Result<()> {
    let s = format!("$-1\r\n");
    write(&s, stream)
}

fn error(msg: &str, stream: &mut TcpStream) -> Result<()> {
    let s = format!("-{msg}\r\n");
    write(&s, stream)
}

fn write(msg: &str, stream: &mut TcpStream) -> Result<()> {
    stream.write(msg.as_bytes())?;
    stream.flush()?;
    Ok(())
}
