//! # Fhedb Query
//!
//! This crate provides the query parsing functionality for the Fhedb database.

pub mod error;
pub mod lexer;
pub mod parser;

/// Re-exports commonly used items from this crate.
pub mod prelude {
    pub use crate::error::ParserError;
    pub use crate::parser::contextual::parse_contextual_query;
    pub use crate::parser::database::parse_database_query;
}
