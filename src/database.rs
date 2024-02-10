use crate::redis_message::RedisCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct DataBase {
    pub db: Arc<RwLock<HashMap<String, (String, SystemTime)>>>,
    pub dir: String,
    pub dbfilename: String,
}

impl DataBase {
    pub fn new(dir: String, dbfilename: String) -> Arc<DataBase> {
        Arc::new(DataBase {
            db: Arc::new(RwLock::new(HashMap::new())),
            dir,
            dbfilename,
        })
    }

    pub fn get(&self, redis_cmd: RedisCommand) -> String {
        let now = SystemTime::now();
        let db = self.db.read();
        if db.is_err() {
            return String::new();
        }
        let db = db.unwrap();
        let (val, expiry) = db
            .get(&redis_cmd.tokens[0])
            .unwrap_or(&(String::new(), now))
            .clone();
        if now > expiry {
            return String::new();
        }
        val
    }

    pub fn set(&self, redis_cmd: RedisCommand) {
        let db = self.db.write();
        if db.is_err() {
            return;
        }
        let mut db = db.unwrap();
        db.insert(
            redis_cmd.tokens[0].clone(),
            (redis_cmd.tokens[1].clone(), redis_cmd.expiry),
        );
    }

    pub fn config_get(&self, key: &str) -> Option<Vec<String>> {
        match key {
            "dir" => Some(vec!["dir".to_string(), self.dir.clone()]),
            "dbfilename" => Some(vec!["dbfilename".to_string(), self.dbfilename.to_string()]),
            _ => None,
        }
    }

    pub fn get_key_list(&self) -> Vec<String> {
        let db = self.db.read();
        if db.is_err() {
            return vec![];
        }
        let db = db.unwrap();
        db.keys().cloned().collect()
    }

    // Everything I need to know https://rdb.fnordig.de/file_format.html
    pub fn load(&self) -> Result<()> {
        if self.dbfilename.is_empty() {
            return Ok(());
        }

        let buffer = fs::read(format!("{}/{}", self.dir, self.dbfilename))?;
        let (mut pointer, buffer_size) = (0, buffer.len());
        let mut metadata = HashMap::new();
        let mut data = vec![];

        let magic_str = String::from_utf8(buffer[pointer..pointer + 5].to_vec())?;
        println!("{}", magic_str);
        pointer += 5;

        let rdb_version = String::from_utf8(buffer[pointer..pointer + 4].to_vec())?;
        println!("Version: {}", rdb_version);
        pointer += 4;

        println!();

        while pointer < buffer_size {
            match buffer[pointer] {
                0xFA => {
                    let (k, p) = string_encoding(pointer + 1, &buffer)?;
                    let (v, p) = string_encoding(p, &buffer)?;
                    pointer = p;
                    metadata.insert(k, v);
                }
                0xFE => {
                    pointer += 1;
                    println!("DB number: {}", buffer[pointer]);
                    pointer += 1;
                }
                0xFB => {
                    let (_expire_size, p, _) = length_encoding(pointer, &buffer)?;
                    let (hash_size, p, _) = length_encoding(p, &buffer)?;
                    pointer = p + 1;

                    println!("size - {} / Expire - {}", hash_size, _expire_size);

                    for _ in 0..hash_size {
                        let _val_type = buffer[pointer];
                        pointer += 1;

                        let (key, p) = string_encoding(pointer, &buffer)?;
                        let (val, p) = string_encoding(p, &buffer)?;
                        data.push((key, val));
                        pointer = p;
                    }
                }
                0xFF => break,
                _ => pointer += 1,
            }
        }

        let mut db = self.db.write().unwrap();

        for (k, v) in data {
            db.insert(
                k,
                (v, SystemTime::now() + Duration::from_secs(u32::MAX as _)),
            );
        }

        Ok(())
    }
}

fn length_encoding(p: usize, buffer: &Vec<u8>) -> Result<(usize, usize, bool)> {
    let code = buffer[p] & 192;
    match code {
        0 => Ok((buffer[p] as usize, p + 1, false)),
        64 => {
            let (mut byte1, byte2) = (buffer[p], buffer[p + 1]);
            byte1 ^= 64;
            let val = (byte2 as u16) << 8 | (byte1 as u16);
            return Ok((val as usize, p + 2, false));
        }
        128 => {
            let p = p + 1;
            let (b1, b2, b3, b4) = (buffer[p + 3], buffer[p + 2], buffer[p + 1], buffer[p]);
            let val = (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32);
            Ok((val as usize, p + 4, false))
        }
        192 => {
            let special_code = buffer[p] ^ 192;
            match special_code {
                0 => Ok((buffer[p + 1] as usize, p + 2, true)),
                1 => {
                    let (byte1, byte2) = (buffer[p + 1], buffer[p + 2]);
                    let val = ((byte2 as u16) << 8 | (byte1 as u16)) as usize;
                    Ok((val as usize, p + 2, true))
                }
                2 => {
                    let (b1, b2, b3, b4) =
                        (buffer[p + 4], buffer[p + 3], buffer[p + 2], buffer[p + 1]);

                    let val =
                        (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32);
                    Ok((val as usize, p + 5, true))
                }
                3 => {
                    // TODO Need to implement code 3
                    Ok((0, p + 1, true))
                }
                _ => Ok((0, p + 1, true)),
            }
        }
        _ => return Ok((buffer[p] as usize, p + 1, false)),
    }
}

fn string_encoding(p: usize, buffer: &Vec<u8>) -> Result<(String, usize)> {
    let (l, p, special) = length_encoding(p, buffer)?;
    if special {
        return Ok((l.to_string(), p));
    }
    let s = buffer[p..p + l as usize].to_vec();
    let s = String::from_utf8(s)?;
    Ok((s, p + l as usize))
}
