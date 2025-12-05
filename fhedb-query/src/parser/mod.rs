//! # Query Parsers
//!
//! This module provides parsers for different types of FHEDB queries.

/// Common utilities for parsers.
pub(crate) mod common;

/// Parsers for database-level queries.
pub mod database;

/// Parsers for collection-level queries.
pub(crate) mod collection;

/// Parsers for document-level queries.
pub(crate) mod document;

/// Parsers for contextual queries.
pub mod contextual;
