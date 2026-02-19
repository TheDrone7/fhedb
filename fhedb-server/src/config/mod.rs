//! # Configuration
//!
//! Server, logging, and storage configuration for the fhedb server.

/// Core configuration combining server, logging, and storage.
pub mod core;
/// Logging configuration.
pub(crate) mod logging;
/// Server configuration.
pub(crate) mod server;
/// Storage configuration.
pub(crate) mod storage;
