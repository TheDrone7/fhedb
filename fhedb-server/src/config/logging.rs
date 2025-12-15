//! The logging configuration for the fhedb server.
//!
//! This module provides the logging configuration for the fhedb server.
//! It includes the log level and the directory where the logs will be stored.

use chrono;
use dirs::data_local_dir;
use log::LevelFilter;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};
use std::{fs::create_dir_all, path::PathBuf};

/// Serializes the log level to a string.
mod log_level_serde {
    use super::*;
    pub fn serialize<S>(level: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let level_str = match level {
            LevelFilter::Off => "OFF",
            LevelFilter::Trace => "TRACE",
            LevelFilter::Debug => "DEBUG",
            LevelFilter::Info => "INFO",
            LevelFilter::Warn => "WARN",
            LevelFilter::Error => "ERROR",
        };
        serializer.serialize_str(level_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "off" => Ok(LevelFilter::Off),
            "trace" => Ok(LevelFilter::Trace),
            "debug" => Ok(LevelFilter::Debug),
            "info" => Ok(LevelFilter::Info),
            "warn" => Ok(LevelFilter::Warn),
            "error" => Ok(LevelFilter::Error),
            _ => Err(DeError::custom("Invalid log level")),
        }
    }
}

/// The logging configuration for the fhedb server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// The log level for the fhedb server.
    #[serde(with = "log_level_serde")]
    level: LevelFilter,
    /// The directory where the logs will be stored.
    dir: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut log_dir = data_local_dir().expect("Failed to locate the local data directory.");
        log_dir.push("fhedb");
        log_dir.push("logs");
        Self {
            level: LevelFilter::Warn,
            dir: Some(log_dir),
        }
    }
}

impl LoggingConfig {
    /// Returns the log level for the fhedb server.
    pub fn level(&self) -> LevelFilter {
        self.level
    }

    /// Ensures that the log directory exists.
    pub fn ensure_log_dir(&self) {
        if let Some(ref dir) = self.dir
            && !dir.exists()
        {
            create_dir_all(dir).expect("Failed to create log directory");
        }
    }

    /// Returns the log file path for the fhedb server.
    /// Based on current time in the logs directory.
    pub fn file(&self) -> Option<PathBuf> {
        if let Some(mut file) = self.dir.clone() {
            file.push(
                chrono::Local::now()
                    .format("%Y-%m-%d_%H-%M-%S.log")
                    .to_string(),
            );
            return Some(file);
        }
        None
    }
}
