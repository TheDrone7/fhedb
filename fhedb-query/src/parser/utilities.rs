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

/// Parses a quoted string with the given delimiter, handling escape sequences.
///
/// ## Arguments
///
/// * `input` - The input string to parse (must start with the quote character).
/// * `quote_char` - The quote character to look for ('\"' or '\'').
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the complete quoted string.
pub fn parse_quoted_string(input: &str, quote_char: char) -> IResult<&str, &str> {
    if !input.starts_with(quote_char) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }

    let mut chars = input[1..].char_indices();
    let mut escape_next = false;

    while let Some((i, ch)) = chars.next() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            ch if ch == quote_char => {
                let end_pos = i + 2; // +1 for the quote, +1 for the index
                return Ok((&input[end_pos..], &input[..end_pos]));
            }
            _ => {}
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Char,
    )))
}

/// This function processes escape sequences commonly found in string literals:
/// - `\n` → newline character
/// - `\t` → tab character
/// - `\r` → carriage return
/// - `\0` → null character
/// - `\\` → literal backslash
/// - `\"` → literal double quote
/// - `\'` → literal single quote
///
/// ## Arguments
///
/// * `input` - The input string that may contain escape sequences.
///
/// ## Returns
///
/// Returns a new `String` with escape sequences processed into their literal characters.
/// Invalid escape sequences (like `\z`) are left as-is.
pub fn unescape_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('0') => result.push('\0'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

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

/// Parses content between balanced delimiters while properly handling quoted strings.
///
/// ## Arguments
///
/// * `input` - The input string to parse (should start with content after the opening delimiter).
/// * `open_char` - The opening delimiter character (e.g., '{' or '[').
/// * `close_char` - The closing delimiter character (e.g., '}' or ']').
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the content between the delimiters.
pub fn balanced_delimiters_content(
    input: &str,
    open_char: char,
    close_char: char,
) -> IResult<&str, &str> {
    let mut delimiter_count = 0;
    let mut in_string = false;
    let mut string_delimiter = '\0';
    let mut chars = input.char_indices();
    let mut escape_next = false;

    while let Some((i, ch)) = chars.next() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' | '\'' if !in_string => {
                in_string = true;
                string_delimiter = ch;
            }
            ch if in_string && ch == string_delimiter => {
                in_string = false;
                string_delimiter = '\0';
            }
            ch if ch == open_char && !in_string => {
                delimiter_count += 1;
            }
            ch if ch == close_char && !in_string => {
                if delimiter_count == 0 {
                    return Ok((&input[i..], &input[..i]));
                }
                delimiter_count -= 1;
            }
            _ => {}
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Char,
    )))
}

/// Parses content between balanced braces while properly handling quoted strings.
/// Extension of `balanced_delimiters_content` specifically for `{` and `}`.
///
/// ## Arguments
///
/// * `input` - The input string to parse (should start with content after the opening brace).
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the content between the braces.
pub fn balanced_braces_content(input: &str) -> IResult<&str, &str> {
    balanced_delimiters_content(input, '{', '}')
}

/// Splits content by delimiter while respecting nested structures and quoted strings.
///
/// ## Arguments
///
/// * `content` - The input string to split.
/// * `delimiter` - The character to split on.
/// * `track_brackets` - Whether to track bracket depth (for arrays).
///
/// ## Returns
///
/// Returns a vector of trimmed individual items.
pub fn split_respecting_nesting(
    content: &str,
    delimiter: char,
    track_brackets: bool,
) -> Vec<String> {
    let mut items = Vec::new();
    if content.trim().is_empty() {
        return items;
    }

    let mut current_item = String::new();
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_delimiter = '\0';
    let mut chars = content.chars();
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            escape_next = false;
            current_item.push(ch);
            continue;
        }

        match ch {
            '\\' if in_string => {
                escape_next = true;
                current_item.push(ch);
            }
            '"' | '\'' if !in_string => {
                in_string = true;
                string_delimiter = ch;
                current_item.push(ch);
            }
            ch if in_string && ch == string_delimiter => {
                in_string = false;
                string_delimiter = '\0';
                current_item.push(ch);
            }
            '[' if !in_string && track_brackets => {
                bracket_depth += 1;
                current_item.push(ch);
            }
            ']' if !in_string && track_brackets => {
                bracket_depth -= 1;
                current_item.push(ch);
            }
            ch if ch == delimiter && bracket_depth == 0 && !in_string => {
                if !current_item.trim().is_empty() {
                    items.push(current_item.trim().to_string());
                }
                current_item.clear();
            }
            _ => {
                current_item.push(ch);
            }
        }
    }

    if !current_item.trim().is_empty() {
        items.push(current_item.trim().to_string());
    }

    items
}

/// Parses an array literal string into individual element strings.
///
/// ## Arguments
///
/// * `input` - The input string containing the array literal.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and a vector of element strings.
fn parse_array_elements(input: &str) -> IResult<&str, Vec<String>> {
    let trimmed = input.trim();

    if !trimmed.starts_with('[') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }

    let content_after_bracket = &trimmed[1..];
    let (remaining_after_content, content) =
        balanced_delimiters_content(content_after_bracket, '[', ']')?;

    let remaining = if remaining_after_content.starts_with(']') {
        &remaining_after_content[1..]
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };

    let elements = split_respecting_nesting(content, ',', true);
    Ok((remaining, elements))
}

/// Parses an array literal into a BSON array value based on the inner field type.
///
/// ## Arguments
///
/// * `array_str` - The string representation of the array literal.
/// * `inner_type` - The type that each array element should conform to.
///
/// ## Returns
///
/// Returns `Ok(Bson::Array)` if all elements can be parsed according to the inner type,
/// or `Err(ParseError)` if parsing fails for any element.
fn parse_array_bson_value(array_str: &str, inner_type: &FieldType) -> ParseResult<Bson> {
    match parse_array_elements(array_str) {
        Ok((remaining, element_strings)) => {
            if !remaining.trim().is_empty() {
                return Err(ParseError::SyntaxError {
                    message: format!("Unexpected input after array: {}", remaining),
                });
            }

            let mut bson_elements = Vec::new();
            for element_str in element_strings {
                let trimmed = element_str.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let bson_value = match inner_type {
                    FieldType::Array(nested_inner_type) => {
                        parse_array_bson_value(trimmed, nested_inner_type)?
                    }
                    _ => parse_bson_value(trimmed.to_string(), inner_type)?,
                };
                bson_elements.push(bson_value);
            }

            Ok(Bson::Array(bson_elements))
        }
        Err(e) => Err(ParseError::SyntaxError {
            message: format!("Failed to parse array: {}", e),
        }),
    }
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
            if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                let unquoted = &trimmed[1..trimmed.len() - 1];
                Ok(Bson::String(unescape_string(unquoted)))
            } else {
                Err(ParseError::SyntaxError {
                    message: format!(
                        "String values must be quoted with single or double quotes: '{}'",
                        trimmed
                    ),
                })
            }
        }
        FieldType::Nullable(inner_type) => {
            if trimmed.to_lowercase() == "null" {
                Ok(Bson::Null)
            } else {
                parse_bson_value(value_str, inner_type)
            }
        }
        FieldType::Array(inner_type) => {
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                parse_array_bson_value(trimmed, inner_type)
            } else {
                Err(ParseError::SyntaxError {
                    message: format!(
                        "Array values must be enclosed in square brackets: '{}'",
                        trimmed
                    ),
                })
            }
        }
        FieldType::Reference(_) => {
            if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                let unquoted = &trimmed[1..trimmed.len() - 1];
                Ok(Bson::String(unescape_string(unquoted)))
            } else {
                Err(ParseError::SyntaxError {
                    message: format!(
                        "Reference values must be quoted with single or double quotes: '{}'",
                        trimmed
                    ),
                })
            }
        }
    }
}
