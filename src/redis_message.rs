use crate::BUFFER_SIZE;
use anyhow::{Error, Result};
use std::str::FromStr;

#[derive(Debug)]
pub enum COMMANDS {
    COMMAND,
    PING,
    ECHO,
    GET,
    SET,
}

impl FromStr for COMMANDS {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<COMMANDS, Self::Err> {
        let input = input.to_lowercase();
        match &input[..] {
            "command" | "COMMAND" => Ok(COMMANDS::COMMAND),
            "ping" | "PING" => Ok(COMMANDS::PING),
            "echo" | "ECHO" => Ok(COMMANDS::ECHO),
            "get" | "GET" => Ok(COMMANDS::GET),
            "set" | "SET" => Ok(COMMANDS::SET),
            _ => Err(Error::msg("invalid command")),
        }
    }
}

#[derive(Debug)]
pub struct RedisCommand {
    pub command: COMMANDS,
    pub tokens: Vec<String>,
}

impl RedisCommand {
    pub fn new(tokens: Vec<String>) -> Result<RedisCommand> {
        let command = COMMANDS::from_str(&tokens[0][..])?;
        Ok(RedisCommand {
            command,
            tokens: tokens[1..].to_vec(),
        })
    }

    pub fn parse_request(request: &[u8; BUFFER_SIZE]) -> Result<RedisCommand> {
        let (mut index, mut bytes) = (1, vec![]);

        while index < BUFFER_SIZE && request[index] as char != '\r' {
            bytes.push(request[index]);
            index += 1;
        }

        let token = String::from_utf8(bytes)?;
        let _size: usize = token.parse()?;
        bytes = vec![];
        index += 2;

        let mut tokens = vec![];
        if index < BUFFER_SIZE && request[index] as char == '$' {
            index += 1;
            while index < BUFFER_SIZE && request[index] as char != '\r' && request[index] != 0 {
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
        let command = RedisCommand::new(tokens)?;
        Ok(command)
    }
}
