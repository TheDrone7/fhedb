//! The configuration module for the fhedb server.
//!
//! This module provides the configuration for the fhedb server.
//! It includes the logging configuration, server configuration, and storage configuration.

/// The core configuration for the fhedb server.
pub mod core;
/// The logging configuration for the fhedb server.
pub(crate) mod logging;
/// The server configuration for the fhedb server.
pub(crate) mod server;
/// The storage configuration for the fhedb server.
pub(crate) mod storage;
