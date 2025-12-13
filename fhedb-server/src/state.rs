//! # Server State
//!
//! This module provides the shared server state that is passed to all request handlers.
//! It maintains a thread-safe cache of loaded databases and the base data directory path.

use fhedb_core::prelude::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// The shared state for the fhedb server.
///
/// This struct is cloned and passed to all request handlers via Axum's state mechanism.
/// It contains thread-safe references to loaded databases and configuration paths.
#[derive(Debug, Clone)]
pub struct ServerState {
    /// Thread-safe cache of loaded databases, keyed by database name.
    pub databases: Arc<RwLock<HashMap<String, Database>>>,
    /// The base directory path where database files are stored.
    pub data_dir: PathBuf,
}

impl ServerState {
    /// Creates a new [`ServerState`] with the given data directory.
    ///
    /// ## Arguments
    ///
    /// * `data_dir` - The base [`PathBuf`] for database storage.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            databases: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
        }
    }
}
