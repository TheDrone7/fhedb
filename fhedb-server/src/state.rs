//! # Server State
//!
//! Shared state passed to all request handlers, including database cache and data directory.

use fhedb_core::prelude::Database;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

/// Shared state for the fhedb server, cloned into each handler via Axum state.
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
