//! Common parsing utilities for FHEDB queries.
//!
//! This module provides reusable parsing functions that are used across
//! different query parsers in the FHEDB query language.

use crate::error::ParseError;
use nom::{IResult, bytes::complete::take_while1};

/// Type alias for parsing results.
///
/// This represents the result of a parsing operation, returning either
/// the parsed value of type `T` or a [`ParseError`].
pub type ParseResult<T> = Result<T, ParseError>;

/// Parses an identifier from the input string.
///
/// An identifier is a sequence of alphanumeric characters and underscores.
/// It must contain at least one character and cannot be empty.
///
/// ## Arguments
///
/// * `input` - The input string to parse an identifier from.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed identifier as a string slice.
pub fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}
