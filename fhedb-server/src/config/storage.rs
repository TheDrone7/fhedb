//! # Storage Configuration

use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::{fs::create_dir_all, path::PathBuf};

/// Data storage path configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    base_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let mut base_dir = data_local_dir().expect("Failed to locate local data directory.");
        base_dir.push("fhedb");
        base_dir.push("data");
        Self { base_dir }
    }
}

impl StorageConfig {
    /// Ensures that the base directory exists.
    pub fn ensure_base_dir(&self) {
        if !self.base_dir.exists() {
            create_dir_all(&self.base_dir).expect("Failed to create storage directory");
        }
    }

    /// Returns the base directory.
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}
