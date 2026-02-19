//! # Value Parsing
//!
//! Provides utilities for parsing string values into BSON.

use crate::schema::validate_bson_type;
use bson::Bson;
use fhedb_types::FieldType;

/// Trait for parsing string values into BSON.
pub trait ValueParseable {
    /// Parses this string value into BSON based on the expected field type.
    ///
    /// ## Arguments
    ///
    /// * `expected_type` - The expected field type for validation.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Bson`]) with the parsed value, or [`Err`]\([`String`]) if
    /// parsing or type validation fails.
    fn parse_as_bson(&self, expected_type: &FieldType) -> Result<Bson, String>;
}

impl ValueParseable for str {
    fn parse_as_bson(&self, expected_type: &FieldType) -> Result<Bson, String> {
        let input = self.trim();
        let value = parse_value_string(input)?;
        validate_bson_type(&value, expected_type)?;
        Ok(value)
    }
}

/// Parses a value string to BSON without type validation.
///
/// ## Arguments
///
/// * `input` - Raw value string.
///
/// ## Returns
///
/// Returns the parsed [`Bson`] value.
fn parse_value_string(input: &str) -> Result<Bson, String> {
    if input.eq_ignore_ascii_case("null") {
        return Ok(Bson::Null);
    }

    if input.eq_ignore_ascii_case("true") {
        return Ok(Bson::Boolean(true));
    }
    if input.eq_ignore_ascii_case("false") {
        return Ok(Bson::Boolean(false));
    }

    if (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''))
    {
        let inner = &input[1..input.len() - 1];
        return Ok(Bson::String(inner.unescape()));
    }

    if input.starts_with('[') && input.ends_with(']') {
        return parse_array(input);
    }

    if let Ok(n) = input.parse::<i64>() {
        return Ok(Bson::Int64(n));
    }

    if let Ok(n) = input.parse::<f64>() {
        return Ok(Bson::Double(n));
    }

    Err(format!("Cannot parse value: {input}"))
}

/// Parses an array string to BSON array.
///
/// ## Arguments
///
/// * `input` - Array string like "[1, 2, 3]" or "[\"a\", \"b\"]".
///
/// ## Returns
///
/// Returns the parsed [`Bson::Array`].
fn parse_array(input: &str) -> Result<Bson, String> {
    let inner = &input[1..input.len() - 1];
    let elements = split_array_elements(inner)?;
    let mut result = Vec::new();
    for elem in elements {
        result.push(parse_value_string(elem.trim())?);
    }
    Ok(Bson::Array(result))
}

/// Splits array content into individual elements, respecting nested brackets and quotes.
///
/// ## Arguments
///
/// * `inner` - The content inside the array brackets.
///
/// ## Returns
///
/// Returns a vector of element strings.
fn split_array_elements(inner: &str) -> Result<Vec<&str>, String> {
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut elements = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut start = 0;
    let chars: Vec<char> = inner.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        match c {
            '"' | '\'' if !in_string => {
                in_string = true;
                string_char = c;
            }
            c if in_string && c == string_char => {
                if i == 0 || chars[i - 1] != '\\' {
                    in_string = false;
                }
            }
            '[' if !in_string => depth += 1,
            ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                elements.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < inner.len() {
        elements.push(&inner[start..]);
    }

    Ok(elements)
}

/// Trait for unescaping string content.
pub trait Unescapable {
    /// Unescapes this string, processing escape sequences.
    ///
    /// Handles `\n`, `\t`, `\r`, `\0`, `\\`, `\"`, and `\'` sequences.
    ///
    /// ## Returns
    ///
    /// Returns the unescaped string.
    fn unescape(&self) -> String;
}

impl Unescapable for str {
    fn unescape(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut chars = self.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.peek() {
                    Some('n') => {
                        chars.next();
                        result.push('\n');
                    }
                    Some('t') => {
                        chars.next();
                        result.push('\t');
                    }
                    Some('r') => {
                        chars.next();
                        result.push('\r');
                    }
                    Some('0') => {
                        chars.next();
                        result.push('\0');
                    }
                    Some('\\') => {
                        chars.next();
                        result.push('\\');
                    }
                    Some('"') => {
                        chars.next();
                        result.push('"');
                    }
                    Some('\'') => {
                        chars.next();
                        result.push('\'');
                    }
                    _ => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}
