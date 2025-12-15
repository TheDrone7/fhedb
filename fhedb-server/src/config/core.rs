//! The core configuration for the fhedb server.
//!
//! This module provides the core configuration for the fhedb server.
//! It includes the logging configuration, server configuration, and storage configuration.

use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use serde_saphyr;
use std::fs::{create_dir_all, read_to_string, write};

use super::{logging::LoggingConfig, server::ServerConfig, storage::StorageConfig};

/// The core configuration for the fhedb server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CoreConfig {
    /// The server configuration for the fhedb server.
    pub server: ServerConfig,
    /// The logging configuration for the fhedb server.
    pub logging: LoggingConfig,
    /// The storage configuration for the fhedb server.
    pub storage: StorageConfig,
}

impl CoreConfig {
    /// Ensures that all directories required by the core config exist.
    pub fn ensure_dirs(&self) {
        self.storage.ensure_base_dir();
        self.logging.ensure_log_dir();
    }

    /// Reads the core config from the config file.
    ///
    /// Parses and generates the server, storage and logging configurations.
    /// Creates default config if file doesn't exist.
    ///
    /// ## Returns
    ///
    /// The loaded or newly created [`CoreConfig`].
    pub fn read_from_file() -> Self {
        let mut config_dir = config_local_dir().expect("Failed to locate config file's directory.");
        config_dir.push("fhedb");
        let mut config_file = config_dir.clone();
        config_file.push("config.yml");

        if !config_dir.exists() {
            println!(
                "Config directory not found, creating config directory at '{}'.",
                config_dir.display()
            );
            create_dir_all(config_dir).expect("Failed to create config directory.");
        }

        if !config_file.exists() {
            println!(
                "Config file not found, creating config file at '{}'.",
                config_file.display()
            );
            let config = Self::default();
            let config_str = serde_saphyr::to_string(&config).expect("Failed to serialize config.");
            write(&config_file, config_str).expect("Failed to write config file.");
            config
        } else {
            let config_str = read_to_string(&config_file).expect("Failed to read config file.");

            serde_saphyr::from_str(&config_str).expect("Failed to deserialize config file.")
        }
    }
}
