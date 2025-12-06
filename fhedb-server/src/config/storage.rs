use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    base_dir: PathBuf,
}

impl StorageConfig {
    pub fn default() -> Self {
        let mut base_dir = data_local_dir().expect("Failed to locate local data directory.");
        base_dir.push("fhedb");
        base_dir.push("data");
        Self { base_dir }
    }

    pub fn ensure_base_dir(&self) {
        if !self.base_dir.exists() {
            create_dir_all(&self.base_dir).expect("Failed to create storage directory");
        }
    }
}
