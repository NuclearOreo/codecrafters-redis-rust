use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct DataBase {
    pub db: Arc<Mutex<HashMap<String, String>>>,
}

impl DataBase {
    pub fn new() -> DataBase {
        DataBase {
            db: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
