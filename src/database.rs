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

    // Everything I need to know https://rdb.fnordig.de/file_format.html
    pub fn load(&self) -> Result<()> {
        if self.dbfilename.is_empty() {
            return Ok(());
        }

        // let mut index = 9;
        // let bytes = fs::read(format!("{}/{}", self.dir, self.dbfilename))?;

        Ok(())
    }
}
