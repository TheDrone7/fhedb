use fhedb_core::prelude::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct ServerState {
    pub databases: Arc<RwLock<HashMap<String, Database>>>,
    pub data_dir: PathBuf,
}

impl ServerState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            databases: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
        }
    }
}
