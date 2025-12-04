//! Parsers for different types of FHEDB queries.

/// Common utilities for parsers.
pub(crate) mod common;

/// The database module - contains parsers for database operation queries.
pub mod database;

/// The collection module - contains parsers for collection operation queries.
pub(crate) mod collection;

/// The contextual module - contains parsers for contextual queries (collections, documents).
pub mod contextual;
