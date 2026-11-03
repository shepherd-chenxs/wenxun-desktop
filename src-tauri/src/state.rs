use std::sync::{Arc, Mutex};
use crate::database::Database;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}

impl AppState {
    pub fn with_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Database) -> T,
    {
        let db = self.db.lock().map_err(|e| format!("数据库锁定失败: {:?}", e))?;
        Ok(f(&db))
    }
}
