use crate::BUFFER_SIZE;
use anyhow::{bail, Result};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub enum COMMANDS {
    COMMAND,
    PING,
    ECHO,
    GET,
    SET,
    INVALID,
}

impl COMMANDS {
    fn from_str(input: &str) -> COMMANDS {
        let input = input.to_lowercase();
        match &input[..] {
            "command" | "COMMAND" => COMMANDS::COMMAND,
            "ping" | "PING" => COMMANDS::PING,
            "echo" | "ECHO" => COMMANDS::ECHO,
            "get" | "GET" => COMMANDS::GET,
            "set" | "SET" => COMMANDS::SET,
            _ => COMMANDS::INVALID,
        }
    }
}

#[derive(Debug)]
pub struct RedisCommand {
    pub command: COMMANDS,
    pub tokens: Vec<String>,
    pub expiry: SystemTime,
}

impl RedisCommand {
    pub fn new(tokens: Vec<String>) -> Result<RedisCommand> {
        let command = COMMANDS::from_str(&tokens[0][..]);
        let mut expiry = SystemTime::now() + Duration::from_secs(u32::MAX as _);

        match command {
            COMMANDS::ECHO => match tokens.len() {
                3.. => bail!("ECHO - too many tokens"),
                0..=1 => bail!("ECHO - too few tokens"),
                _ => (),
            },
            COMMANDS::GET => match tokens.len() {
                3.. => bail!("GET - too many tokens"),
                0..=1 => bail!("GET - too few tokens"),
                _ => (),
            },
            COMMANDS::SET => match tokens.len() {
                6.. => bail!("SET - too many tokens"),
                0..=2 => bail!("SET - too few tokens"),
                4 => bail!("SET - missing time value"),
                5 => {
                    let val: u64 = match tokens[4].parse() {
                        Ok(v) => v,
                        Err(e) => bail!("SET - {}", e.to_string()),
                    };
                    expiry = SystemTime::now() + Duration::from_millis(val);
                }
                _ => (),
            },
            _ => (),
        }

        Ok(RedisCommand {
            command,
            tokens: tokens[1..].to_vec(),
            expiry,
        })
    }

    pub fn parse_request(request: &[u8; BUFFER_SIZE]) -> Result<RedisCommand> {
        let (mut index, mut bytes) = (1, vec![]);

        while index < BUFFER_SIZE && request[index] as char != '\r' {
            bytes.push(request[index]);
            index += 1;
        }

        let token = String::from_utf8(bytes)?;
        let _size_of_array: usize = token.parse()?;
        bytes = vec![];
        index += 2;

        let mut tokens = vec![];
        if index < BUFFER_SIZE && request[index] as char == '$' {
            index += 1;
            while index < BUFFER_SIZE && request[index] as char != '\r' && request[index] != 0 {
                while index < BUFFER_SIZE && request[index] as char != '\r' {
                    bytes.push(request[index]);
                    index += 1;
                }

                let token = String::from_utf8(bytes)?;
                let size: usize = token.parse()?;
                bytes = vec![];

                index += 2;

                let token = String::from_utf8(request[index..index + size].to_vec())?;
                tokens.push(token);
                index += size + 3;
            }
        }
        let command = RedisCommand::new(tokens)?;
        Ok(command)
    }
}
