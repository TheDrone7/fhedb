//! Helper functions for parsing document content.

use crate::{
    ast::{FieldCondition, FieldSelector, QueryOperator},
    error::ParseError,
    parser::utilities::{ParseResult, identifier, split_respecting_nesting, trim_parse},
};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{map_res, rest},
    sequence::delimited,
};
use std::collections::HashMap;

/// Parses document content into individual items (separated by commas).
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

/// A helper enum to represent either a field condition or an assignment.
pub enum FieldOperation {
    /// A field condition (e.g., field = value).
    Condition(FieldCondition),
    /// A field assignment (e.g., field: value).
    Assignment((String, String)),
}

/// Parses a field operation (condition and assignment) item.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns `Ok(FieldOperation)` if successfully parsed, or `Err` if not a valid operation.
pub fn parse_document_field_operations(input: &str) -> ParseResult<FieldOperation> {
    trim_parse(input, "field operation", |input| {
        map_res(
            (
                identifier,
                delimited(
                    multispace0,
                    alt((
                        tag("=="),
                        tag("!="),
                        tag(">="),
                        tag("<="),
                        tag("="),
                        tag(">"),
                        tag("<"),
                        tag(":"),
                    )),
                    multispace0,
                ),
                rest,
            ),
            |(field_name, operator, value)| -> ParseResult<FieldOperation> {
                let trimmed_value = value.trim();
                if trimmed_value.is_empty() {
                    return Err(ParseError::SyntaxError {
                        message: "Missing value in field operation".to_string(),
                    });
                }

                match operator {
                    ":" => Ok(FieldOperation::Assignment((
                        field_name.to_string(),
                        value.to_string(),
                    ))),
                    "=" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::Equal,
                        value: trimmed_value.to_string(),
                    })),
                    "!=" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::NotEqual,
                        value: trimmed_value.to_string(),
                    })),
                    ">" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::GreaterThan,
                        value: trimmed_value.to_string(),
                    })),
                    ">=" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::GreaterThanOrEqual,
                        value: trimmed_value.to_string(),
                    })),
                    "<" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::LessThan,
                        value: trimmed_value.to_string(),
                    })),
                    "<=" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::LessThanOrEqual,
                        value: trimmed_value.to_string(),
                    })),
                    "==" => Ok(FieldOperation::Condition(FieldCondition {
                        field_name: field_name.to_string(),
                        operator: QueryOperator::Similar,
                        value: trimmed_value.to_string(),
                    })),
                    _ => Err(ParseError::SyntaxError {
                        message: format!("Unsupported operator in field condition: '{}'", operator),
                    }),
                }
            },
        )
        .parse(input)
    })
}

/// Parses document query content into assignments, conditions and field selector.
///
/// ## Arguments
///
/// * `content` - The input string to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult`] containing the parsed assignments, conditions and field selectors.
pub fn parse_doc_content(
    content: &str,
) -> ParseResult<(
    HashMap<String, String>,
    Vec<FieldCondition>,
    Vec<FieldSelector>,
)> {
    let items = parse_document_query_items(content)?;

    let mut conditions = Vec::new();
    let mut selectors = Vec::new();
    let mut assignments = HashMap::new();

    for item in items {
        if let Ok(operation) = parse_document_field_operations(&item) {
            if let FieldOperation::Condition(condition) = operation {
                conditions.push(condition);
            } else if let FieldOperation::Assignment((field, value)) = operation {
                if assignments.contains_key(&field) {
                    return Err(ParseError::SyntaxError {
                        message: format!("Duplicate assignment for field '{}'", field),
                    });
                }
                assignments.insert(field, value);
            }
        } else if let Ok(selector) = parse_field_selector(&item) {
            selectors.push(selector);
        } else {
            return Err(ParseError::SyntaxError {
                message: format!("Invalid item in document query: '{}'", item),
            });
        }
    }

    Ok((assignments, conditions, selectors))
}
