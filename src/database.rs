use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DataBase {
    pub db: Arc<RwLock<HashMap<String, String>>>,
}

impl DataBase {
    pub fn new() -> Arc<DataBase> {
        Arc::new(DataBase {
            db: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn get(&self, key: String) -> String {
        let db = self.db.read();
        if db.is_err() {
            "".to_string();
        }

        let db = db.unwrap();
        let res = db.get(&key).unwrap_or(&"".to_string()).clone();
        res
    }

    pub fn set(&self, key: String, val: String) {
        let db = self.db.write();
        if db.is_err() {
            return;
        }

        let mut db = db.unwrap();
        db.insert(key, val);
    }
}
