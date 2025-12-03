//! # Contextual Query Parser
//!
//! This module provides parsing functionality for contextual FHEDB queries
//! (collections and documents).

use crate::ast::ContextualQuery;
use crate::error::ParserError;

/// Parses a contextual query string into a [`ContextualQuery`] AST node.
///
/// ## Arguments
///
/// * `input` - The query string to parse.
///
/// ## Returns
///
/// Returns [`Ok`]([`ContextualQuery`]) if parsing succeeds,
/// or [`Err`]([`Vec<ParserError>`]) containing all parsing errors if it fails.
pub fn parse_contextual_query(_input: &str) -> Result<ContextualQuery, Vec<ParserError>> {
    Err(vec![])
}
