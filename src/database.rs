use crate::rdb::RDB;
use crate::redis_message::RedisCommand;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Database {
    pub db: Arc<RwLock<HashMap<String, (String, SystemTime)>>>,
    pub dir: String,
    pub dbfilename: String,
}

impl Database {
    pub fn new(dir: String, dbfilename: String) -> Arc<Database> {
        Arc::new(Database {
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
        let rdb_file = RDB::new(self.dir.clone(), self.dbfilename.clone())?;

        let db = self.db.write();
        if db.is_err() {
            bail!("nope");
        }
        let mut db = db.unwrap();

        for (k, v, e) in rdb_file.data.iter() {
            db.insert(k.clone(), (v.clone(), e.clone()));
        }

        Ok(())
    }
}
