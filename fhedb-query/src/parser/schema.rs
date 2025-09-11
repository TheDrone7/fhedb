//! Schema parsing functionality.
//!
//! This module provides parsers for schema definitions used in collection operations.

use crate::{
    error::ParseError,
    parser::utilities::{ParseResult, identifier},
};
use bson::Bson;
use fhedb_core::db::schema::{FieldDefinition, FieldType, Schema};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::{map, map_res},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded},
};

/// Represents a field constraint
#[derive(Debug, Clone)]
enum FieldConstraint {
    Nullable,
    Default(String),
}

/// Parses a schema's field's constraint.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`FieldConstraint`].
fn parse_field_constraint(input: &str) -> IResult<&str, FieldConstraint> {
    delimited(
        char('('),
        alt((
            map(delimited(multispace0, tag("nullable"), multispace0), |_| {
                FieldConstraint::Nullable
            }),
            map(
                preceded(
                    delimited(multispace0, tag("default"), multispace0),
                    delimited(
                        char('='),
                        delimited(multispace0, take_until(")"), multispace0),
                        multispace0,
                    ),
                ),
                |value: &str| FieldConstraint::Default(value.to_string()),
            ),
        )),
        char(')'),
    )
    .parse(input)
}

/// Parses a field type from the input string.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`FieldType`].
fn parse_field_type(input: &str) -> IResult<&str, FieldType> {
    alt((
        map(
            (
                tag("array"),
                multispace0,
                delimited(
                    delimited(multispace0, char('<'), multispace0),
                    parse_field_type,
                    delimited(multispace0, char('>'), multispace0),
                ),
            ),
            |(_, _, inner_type)| FieldType::Array(Box::new(inner_type)),
        ),
        map(
            (
                tag("ref"),
                multispace0,
                delimited(
                    delimited(multispace0, char('<'), multispace0),
                    identifier,
                    delimited(multispace0, char('>'), multispace0),
                ),
            ),
            |(_, _, collection_name)| FieldType::Reference(collection_name.to_string()),
        ),
        map(tag("id_string"), |_| FieldType::IdString),
        map(tag("id_int"), |_| FieldType::IdInt),
        map(tag("int"), |_| FieldType::Int),
        map(tag("float"), |_| FieldType::Float),
        map(tag("boolean"), |_| FieldType::Boolean),
        map(tag("string"), |_| FieldType::String),
    ))
    .parse(input)
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
fn parse_default_value(value_str: String, field_type: &FieldType) -> ParseResult<Bson> {
    let trimmed = value_str.trim();

    match field_type {
        FieldType::Int => {
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
        FieldType::String => {
            let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            };
            Ok(Bson::String(unquoted.to_string()))
        }
        FieldType::IdString => Err(ParseError::SyntaxError {
            message: "ID fields cannot have default values as they must be unique".to_string(),
        }),
        FieldType::IdInt => Err(ParseError::SyntaxError {
            message: "ID fields cannot have default values as they must be unique".to_string(),
        }),
        FieldType::Nullable(inner_type) => {
            if trimmed.to_lowercase() == "null" {
                Ok(Bson::Null)
            } else {
                parse_default_value(value_str, inner_type)
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

/// Parses a single field definition in the format: field_name: field_type [constraints...]
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and a tuple of (field_name, FieldDefinition).
fn parse_field_definition(input: &str) -> IResult<&str, (String, FieldDefinition)> {
    map_res(
        (
            identifier,
            delimited(multispace0, char(':'), multispace0),
            parse_field_type,
            many0(preceded(multispace0, parse_field_constraint)),
        ),
        |(name, _, field_type, constraints)| -> ParseResult<(String, FieldDefinition)> {
            let mut is_nullable = false;
            let mut default_value_str: Option<String> = None;

            for constraint in constraints {
                match constraint {
                    FieldConstraint::Nullable => is_nullable = true,
                    FieldConstraint::Default(value) => default_value_str = Some(value),
                }
            }

            let final_type = if is_nullable {
                FieldType::Nullable(Box::new(field_type))
            } else {
                field_type
            };

            let field_def = if let Some(default_str) = default_value_str {
                match parse_default_value(default_str, &final_type) {
                    Ok(default_bson) => FieldDefinition::with_default(final_type, default_bson),
                    Err(e) => return Err(e),
                }
            } else {
                FieldDefinition::new(final_type)
            };

            Ok((name.to_string(), field_def))
        },
    )
    .parse(input)
}

/// Parses a schema definition (the part inside the braces).
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`Schema`].
pub fn parse_schema(input: &str) -> IResult<&str, Schema> {
    map(
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, parse_field_definition, multispace0),
        ),
        |field_definitions| {
            let mut schema = Schema::new();
            for (field_name, field_def) in field_definitions {
                schema.fields.insert(field_name, field_def);
            }
            schema
        },
    )
    .parse(input)
}
