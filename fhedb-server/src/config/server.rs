//! # Server Configuration

use serde::{Deserialize, Serialize};

/// Server host and port configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The host address.
    host: String,
    /// The port number.
    port: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6907,
        }
    }
}

impl ServerConfig {
    /// Returns the host address.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the port number.
    pub fn port(&self) -> u32 {
        self.port
    }
}
