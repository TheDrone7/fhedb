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

/// This utility function handles the complete parsing workflow:
/// 1. Trims the input string
/// 2. Applies the provided parser
/// 3. Validates that no content remains after parsing
///
/// ## Arguments
///
/// * `input` - The input string to parse.
/// * `context` - A description of what is being parsed (for error messages).
/// * `parser` - The nom parser function to apply to the trimmed input.
///
/// ## Returns
///
/// Returns `Ok(T)` with the parsed result, or `Err(ParseError)` if parsing fails or unexpected content remains.
pub fn trim_parse<O, F>(input: &str, context: &str, parser: F) -> ParseResult<O>
where
    F: FnOnce(&str) -> IResult<&str, O>,
{
    let input = input.trim();

    let (remaining, result) = parser(input).map_err(|e| ParseError::SyntaxError {
        message: format!("Failed to parse {}: {}", context, e),
    })?;

    if !remaining.trim().is_empty() {
        return Err(ParseError::SyntaxError {
            message: format!("Unexpected input after {}", context),
        });
    }

    Ok(result)
}

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
