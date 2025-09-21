//! Helper functions for parsing document content.

use crate::{
    ast::{FieldCondition, FieldSelector, QueryOperator},
    error::ParseError,
    parser::utilities::{
        ParseResult, balanced_delimiters_content, identifier, parse_quoted_string,
        split_respecting_nesting, trim_parse,
    },
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::{map_res, rest},
    multi::separated_list0,
    sequence::delimited,
};
use std::collections::HashMap;

/// Parses a field value.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed value.
fn parse_field_value(input: &str) -> IResult<&str, &str> {
    let input = input.trim_start();

    if input.starts_with('[') {
        let content_after_bracket = &input[1..];
        let (remaining_after_content, _) =
            balanced_delimiters_content(content_after_bracket, '[', ']')?;

        if remaining_after_content.starts_with(']') {
            let remaining = &remaining_after_content[1..];
            let start_pos = input.len() - remaining.len();
            Ok((remaining, &input[..start_pos]))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Char,
            )))
        }
    } else if input.starts_with('"') {
        parse_quoted_string(input, '"')
    } else if input.starts_with('\'') {
        parse_quoted_string(input, '\'')
    } else {
        alt((take_until(","), nom::combinator::rest)).parse(input)
    }
}

/// Parses a single field-value pair in the format: field_name: field_value.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and a tuple of (field_name, field_value).
fn parse_field_pair(input: &str) -> IResult<&str, (String, String)> {
    map_res(
        (
            identifier,
            delimited(multispace0, char(':'), multispace0),
            parse_field_value,
        ),
        |(field_name, _, value)| -> ParseResult<(String, String)> {
            Ok((field_name.to_string(), value.trim().to_string()))
        },
    )
    .parse(input)
}

/// Parses document field definitions (the part inside the braces of an insert statement).
///
/// ## Arguments
///
/// * `content` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed map of field names to field values.
pub fn parse_document_fields(content: &str) -> ParseResult<HashMap<String, String>> {
    let fields = trim_parse(content, "document fields", |input| {
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, parse_field_pair, multispace0),
        )
        .parse(input)
    })?;

    let mut field_map = HashMap::new();
    for (field_name, field_value) in fields {
        if field_map.contains_key(&field_name) {
            return Err(ParseError::SyntaxError {
                message: format!("Duplicate field name: '{}'", field_name),
            });
        }
        field_map.insert(field_name, field_value);
    }

    Ok(field_map)
}

/// Parses document content into individual items (conditions and field selectors).
///
/// ## Arguments
///
/// * `content` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed list of individual items.
pub fn parse_document_query_items(content: &str) -> ParseResult<Vec<String>> {
    Ok(split_respecting_nesting(content, ',', true))
}

/// Parses a field selector item (field name, *, or **).
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns `Ok(FieldSelector)` for the appropriate selector type
/// or `Err` if it's not a valid field selector.
pub fn parse_field_selector(input: &str) -> ParseResult<FieldSelector> {
    let trimmed = input.trim();

    match trimmed {
        "*" => Ok(FieldSelector::AllFields),
        "**" => Ok(FieldSelector::AllFieldsRecursive),
        _ => match identifier(trimmed) {
            Ok((remaining, field_name)) => {
                if remaining.trim().is_empty() {
                    Ok(FieldSelector::Field(field_name.to_string()))
                } else {
                    Err(ParseError::SyntaxError {
                        message: format!("Invalid field selector: '{}'", trimmed),
                    })
                }
            }
            Err(_) => Err(ParseError::SyntaxError {
                message: format!("Invalid field selector: '{}'", trimmed),
            }),
        },
    }
}

/// Parses a field condition item (field_name = value).
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns `Ok(FieldCondition)` if successfully parsed, or `Err` if not a valid condition.
pub fn parse_field_condition(input: &str) -> ParseResult<FieldCondition> {
    trim_parse(input, "field condition", |input| {
        map_res(
            (
                identifier,
                delimited(multispace0, tag("="), multispace0),
                rest,
            ),
            |(field_name, _, value)| -> ParseResult<FieldCondition> {
                let trimmed_value = value.trim();
                if trimmed_value.is_empty() {
                    return Err(ParseError::SyntaxError {
                        message: "Missing value in field condition".to_string(),
                    });
                }

                Ok(FieldCondition {
                    field_name: field_name.to_string(),
                    operator: QueryOperator::Equal,
                    value: trimmed_value.to_string(),
                })
            },
        )
        .parse(input)
    })
}

/// Parses GET document content into conditions and field selector.
///
/// ## Arguments
///
/// * `content` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed conditions and field selectors.
pub fn parse_get_content(content: &str) -> ParseResult<(Vec<FieldCondition>, Vec<FieldSelector>)> {
    let items = parse_document_query_items(content)?;

    let mut conditions = Vec::new();
    let mut selectors = Vec::new();

    for item in items {
        if let Ok(condition) = parse_field_condition(&item) {
            conditions.push(condition);
        } else if let Ok(selector) = parse_field_selector(&item) {
            selectors.push(selector);
        } else {
            return Err(ParseError::SyntaxError {
                message: format!("Invalid item in GET query: '{}'", item),
            });
        }
    }

    Ok((conditions, selectors))
}
