//! Common parsing utilities for FHEDB queries.
//!
//! This module provides reusable parsing functions that are used across
//! different query parsers in the FHEDB query language.

use crate::error::ParseError;
use bson::Bson;
use fhedb_core::db::schema::FieldType;
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

/// Parses a default value string into a BSON value based on the field type.
///
/// ## Arguments
///
/// * `value_str` - The string representation of the default value.
/// * `field_type` - The type that the default value should conform to.
///
/// ## Returns
///
/// Returns `Ok(Bson)` if the value can be parsed according to the field type,
/// or `Err(ParseError)` with an error message if parsing fails.
pub fn parse_bson_value(value_str: String, field_type: &FieldType) -> ParseResult<Bson> {
    let trimmed = value_str.trim();

    match field_type {
        FieldType::Int | FieldType::IdInt => {
            trimmed
                .parse::<i64>()
                .map(Bson::Int64)
                .map_err(|_| ParseError::SyntaxError {
                    message: format!("Cannot parse '{}' as integer", trimmed),
                })
        }
        FieldType::Float => {
            trimmed
                .parse::<f64>()
                .map(Bson::Double)
                .map_err(|_| ParseError::SyntaxError {
                    message: format!("Cannot parse '{}' as float", trimmed),
                })
        }
        FieldType::Boolean => match trimmed.to_lowercase().as_str() {
            "true" => Ok(Bson::Boolean(true)),
            "false" => Ok(Bson::Boolean(false)),
            _ => Err(ParseError::SyntaxError {
                message: format!(
                    "Cannot parse '{}' as boolean (expected 'true' or 'false')",
                    trimmed
                ),
            }),
        },
        FieldType::String | FieldType::IdString => {
            let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            };
            Ok(Bson::String(unquoted.to_string()))
        }
        FieldType::Nullable(inner_type) => {
            if trimmed.to_lowercase() == "null" {
                Ok(Bson::Null)
            } else {
                parse_bson_value(value_str, inner_type)
            }
        }
        FieldType::Array(_) => Err(ParseError::SyntaxError {
            message: "Array default values are not supported yet".to_string(),
        }),
        FieldType::Reference(_) => {
            let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            };
            Ok(Bson::String(unquoted.to_string()))
        }
    }
}
