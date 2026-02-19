//! # Fhedb Query
//!
//! This crate provides the query parsing functionality for the Fhedb database.

/// Error types and formatting for query parsing.
pub mod error;
/// Lexical analysis and tokenization.
pub mod lexer;
/// Query parsers for all query types.
pub mod parser;

/// Commonly used types re-exported for easy access.
pub mod prelude {
    pub use crate::{
        error::ParserError,
        parser::{contextual::parse_contextual_query, database::parse_database_query},
    };
}
