//! # Fhedb Core
//!
//! This crate provides the core functionality for the Fhedb database.

/// The database module - contains the core database structures.
pub mod db;

/// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::db::collection::Collection;
    pub use crate::db::schema::{FieldType, Schema};
}
