//! # Fhedb Server
//!
//! This crate provides the HTTP server for the Fhedb database.
//! It handles incoming requests, parses queries, and executes database operations.

/// Configuration module for server, logging, and storage settings.
pub mod config;
/// Query extractor for parsing incoming requests.
pub mod extractor;
/// Request handlers for database and collection operations.
pub mod handlers;
/// Logging setup utilities.
pub mod logger;
/// Middleware for request processing.
pub mod middleware;
/// Unified response types for consistent API responses.
pub mod response;
/// Server state management.
pub mod state;

/// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::config::core::CoreConfig;
    pub use crate::handlers::*;
    pub use crate::logger::setup_logger;
    pub use crate::middleware::check_database;
    pub use crate::response::ApiResponse;
    pub use crate::state::ServerState;
    pub use crate::{error, internal_error, success};
}
