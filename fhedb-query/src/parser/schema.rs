//! Schema parsing functionality.
//!
//! This module provides parsers for schema definitions used in collection operations.

use crate::{
    ast::FieldModification,
    error::ParseError,
    parser::utilities::{ParseResult, identifier, parse_bson_value, trim_parse},
};
use fhedb_core::db::schema::{FieldDefinition, FieldType, Schema};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_until},
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
            map(
                delimited(multispace0, tag_no_case("nullable"), multispace0),
                |_| FieldConstraint::Nullable,
            ),
            map(
                preceded(
                    delimited(multispace0, tag_no_case("default"), multispace0),
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
                tag_no_case("array"),
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
                tag_no_case("ref"),
                multispace0,
                delimited(
                    delimited(multispace0, char('<'), multispace0),
                    identifier,
                    delimited(multispace0, char('>'), multispace0),
                ),
            ),
            |(_, _, collection_name)| FieldType::Reference(collection_name.to_string()),
        ),
        map(tag_no_case("id_string"), |_| FieldType::IdString),
        map(tag_no_case("id_int"), |_| FieldType::IdInt),
        map(tag_no_case("int"), |_| FieldType::Int),
        map(tag_no_case("float"), |_| FieldType::Float),
        map(tag_no_case("boolean"), |_| FieldType::Boolean),
        map(tag_no_case("string"), |_| FieldType::String),
    ))
    .parse(input)
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
                match &final_type {
                    FieldType::IdString | FieldType::IdInt => {
                        return Err(ParseError::SyntaxError {
                            message: "ID fields cannot have default values as they must be unique"
                                .to_string(),
                        });
                    }
                    FieldType::Nullable(inner_type) => {
                        if matches!(inner_type.as_ref(), FieldType::IdString | FieldType::IdInt) {
                            return Err(ParseError::SyntaxError {
                                message:
                                    "ID fields cannot have default values as they must be unique"
                                        .to_string(),
                            });
                        }
                    }
                    _ => {}
                }

                match parse_bson_value(default_str, &final_type) {
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

/// Parses a single field modification in the format: field_name: drop | field_name: field_type [constraints...]
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and a tuple of (field_name, FieldModification).
fn parse_field_modification(input: &str) -> IResult<&str, (String, FieldModification)> {
    alt((
        map(
            (
                identifier,
                delimited(multispace0, char(':'), multispace0),
                tag_no_case("drop"),
            ),
            |(name, _, _)| (name.to_string(), FieldModification::Drop),
        ),
        map(parse_field_definition, |(name, field_def)| {
            (name, FieldModification::Set(field_def))
        }),
    ))
    .parse(input)
}

/// Parses multiple field modifications (the part inside the braces of a modify collection statement).
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed map of field modifications.
pub fn parse_field_modifications(
    input: &str,
) -> ParseResult<std::collections::HashMap<String, FieldModification>> {
    let modifications = trim_parse(input, "field modifications", |input| {
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, parse_field_modification, multispace0),
        )
        .parse(input)
    })?;

    let mut modification_map = std::collections::HashMap::new();
    for (name, modification) in modifications {
        if modification_map.contains_key(&name) {
            return Err(ParseError::SyntaxError {
                message: format!("Duplicate field modification: {}", name),
            });
        }
        modification_map.insert(name, modification);
    }

    Ok(modification_map)
}

/// Parses a schema definition (the part inside the braces).
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed [`Schema`].
pub fn parse_schema(input: &str) -> ParseResult<Schema> {
    let fields = trim_parse(input, "schema definition", |input| {
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            delimited(multispace0, parse_field_definition, multispace0),
        )
        .parse(input)
    })?;

    let mut schema = Schema::new();
    for (name, field_def) in fields {
        if schema.fields.contains_key(&name) {
            return Err(ParseError::SyntaxError {
                message: format!("Duplicate field name: {}", name),
            });
        }
        schema.fields.insert(name, field_def);
    }

    Ok(schema)
}
