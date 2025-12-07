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

impl ServerConfig {
    /// Creates a new server configuration with default values.
    pub fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6907,
        }
    }

    /// Returns the host address of the fhedb server.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Returns the port number of the fhedb server.
    pub fn get_port(&self) -> u32 {
        self.port
    }
}
