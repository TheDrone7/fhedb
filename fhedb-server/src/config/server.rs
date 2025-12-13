//! The server configuration for the fhedb server.

use serde::{Deserialize, Serialize};

/// The fhedb server's configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The host address of the fhedb server.
    host: String,
    /// The port number of the fhedb server.
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
    /// Returns the host address of the fhedb server.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the port number of the fhedb server.
    pub fn port(&self) -> u32 {
        self.port
    }
}
