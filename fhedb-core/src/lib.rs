//! # Fhedb Core
//!
//! This crate provides the core functionality for the Fhedb database.

/// The database module - contains the core database structures.
pub mod db;

/// The file module - contains the file operations for the database.
pub mod file;

/// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::db::collection::Collection;
    pub use crate::db::database::Database;
    pub use crate::db::document::{DocId, Document};
    pub use crate::db::schema::{FieldType, IdType, Schema};
    pub use crate::file::collection::CollectionFileOps;
    pub use crate::file::types::{LogEntry, Operation};
}
