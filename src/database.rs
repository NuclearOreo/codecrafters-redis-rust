use crate::redis_message::RedisCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

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

    pub fn load(&self) -> Result<()> {
        if self.dbfilename.is_empty() {
            return Ok(());
        }

        let mut index = 9;
        let bytes = fs::read(format!("{}/{}", self.dir, self.dbfilename))?;

        let s = String::from_utf8(bytes[..index].to_vec())?;
        index += 1;
        println!("{:?}", s);

        let fb_pos = bytes.iter().position(|&b| b == 0xfb).unwrap() + 1;
        let s = String::from_utf8(bytes[fb_pos + 8..fb_pos + 21].to_vec())?;
        println!("{:?}", s);

        // let code = (bytes[index] as u16) << 8 | (bytes[index + 1] as u16);
        // println!("{:}", code);

        Ok(())
    }
}

// Ok(file) => {
//     let mut buffer: [u8; 1024] = [0; 1024];
//     let mut reader = std::io::BufReader::new(file);
//     reader.read(&mut buffer).unwrap();
//     println!("{:x?}", buffer);
//     let fb_pos = buffer.iter().position(|&b| b == 0xfb).unwrap();
//     let mut pos = fb_pos + 4;
//     let len = buffer[pos];
//     pos += 1;
//     let key = &buffer[pos..(pos + len as usize)];
//     println!("1 {:x?}", key);
//     let pars = std::str::from_utf8(key).unwrap();
//     println!("2 {:?}", pars);
//     return Request::KEYS(vec![pars.to_string()]);
// }
