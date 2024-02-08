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
                    COMMANDS::Command => result("CONNECTED", &mut stream)?,
                    COMMANDS::Ping => result("PONG", &mut stream)?,
                    COMMANDS::Keys => {
                        let keys = database.get_key_list();
                        array(keys, &mut stream)?
                    }
                    COMMANDS::Get => {
                        let val = database.get(redis_cmd);
                        if val.is_empty() {
                            null(&mut stream)?;
                            continue;
                        }
                        result(&val, &mut stream)?;
                    }
                    COMMANDS::Set => {
                        database.set(redis_cmd);
                        result("OK", &mut stream)?
                    }
                    COMMANDS::Echo => result(&redis_cmd.tokens[0], &mut stream)?,
                    COMMANDS::ConfigGet => {
                        if let Some(v) = database.config_get(&redis_cmd.tokens[0]) {
                            array(v, &mut stream)?;
                            continue;
                        }
                        error("Unsupported key", &mut stream)?
                    }
                    COMMANDS::Invalid => error("INVALID COMMAND", &mut stream)?,
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

fn array(array: Vec<String>, stream: &mut TcpStream) -> Result<()> {
    let s = array
        .iter()
        .fold(format!("*{}\r\n", array.len()), |mut v, w| {
            v += &format!("${}\r\n{}\r\n", w.len(), w);
            v
        });
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
