//! # Fhedb Query
//!
//! This crate provides the query parsing functionality for the Fhedb database.

/// The AST module - contains the abstract syntax tree definitions.
pub mod ast;
/// The error module - contains error definitions for parsing.
pub mod error;
/// The lexer module - contains the lexer for tokenizing input.
pub mod lexer;
/// The parser module - contains the query parser implementations.
pub mod parser;

/// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::ast::*;
    pub use crate::error::ParserError;
    pub use crate::parser::database::parse_database_query;
}
