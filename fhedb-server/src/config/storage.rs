//! The storage configuration for the fhedb server.

use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::PathBuf;

/// The fhedb server's storage configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    base_dir: PathBuf,
}

impl StorageConfig {
    /// Creates a new default storage configuration.
    pub fn default() -> Self {
        let mut base_dir = data_local_dir().expect("Failed to locate local data directory.");
        base_dir.push("fhedb");
        base_dir.push("data");
        Self { base_dir }
    }

    /// Ensures that the base directory exists.
    pub fn ensure_base_dir(&self) {
        if !self.base_dir.exists() {
            create_dir_all(&self.base_dir).expect("Failed to create storage directory");
        }
    }

    /// Returns the base directory.
    pub fn get_base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}
