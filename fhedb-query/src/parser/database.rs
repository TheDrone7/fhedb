//! Database query parsing functionality.
//!
//! This module provides parsers for database operation queries such as CREATE DATABASE and DROP DATABASE.

use crate::{
    ast::*,
    error::{ParserError, Span, convert_error, create_span},
    parser::utilities::{ParserResult, identifier},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{multispace0, multispace1},
    combinator::{cut, map, opt},
    sequence::preceded,
};

/// Parses a CREATE DATABASE query.
///
/// ## Arguments
///
/// * `input` - The input [`Span`] to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::Create`].
fn create_database(input: Span) -> IResult<Span, DatabaseQuery> {
    map(
        (
            tag_no_case("create"),
            multispace1,
            tag_no_case("database"),
            cut((
                multispace1,
                identifier,
                opt(preceded(
                    multispace1,
                    (
                        tag_no_case("drop"),
                        multispace1,
                        tag_no_case("if"),
                        multispace1,
                        tag_no_case("exists"),
                    ),
                )),
            )),
        ),
        |(_, _, _, (_, name, drop_if_exists))| DatabaseQuery::Create {
            name: name.fragment().to_string(),
            drop_if_exists: drop_if_exists.is_some(),
        },
    )
    .parse(input)
}

/// Parses a DROP DATABASE query.
///
/// ## Arguments
///
/// * `input` - The input [`Span`] to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::Drop`].
fn drop_database(input: Span) -> IResult<Span, DatabaseQuery> {
    map(
        (
            tag_no_case("drop"),
            multispace1,
            tag_no_case("database"),
            cut((multispace1, identifier)),
        ),
        |(_, _, _, (_, name))| DatabaseQuery::Drop {
            name: name.fragment().to_string(),
        },
    )
    .parse(input)
}

/// Parses a LIST DATABASES query.
///
/// ## Arguments
///
/// * `input` - The input [`Span`] to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::List`].
fn list_databases(input: Span) -> IResult<Span, DatabaseQuery> {
    map(
        (tag_no_case("list"), multispace1, tag_no_case("databases")),
        |(_, _, _)| DatabaseQuery::List,
    )
    .parse(input)
}

/// Parses a complete database query from the input string.
///
/// ## Arguments
///
/// * `input` - The input string containing the database query to parse.
///
/// ## Returns
///
/// Returns a [`ParserResult<DatabaseQuery>`] containing the parsed query on success.
///
/// ## Errors
///
/// This function returns a [`ParserError::SyntaxError`] in the following cases:
/// - The input doesn't match any known database query pattern
/// - There is unexpected input remaining after a valid query
pub fn parse_database_query(input: &str) -> ParserResult<DatabaseQuery> {
    let trimmed_input = input.trim();
    let span = create_span(trimmed_input);

    match preceded(
        multispace0,
        alt((create_database, drop_database, list_databases)),
    )
    .parse(span)
    {
        Ok((remaining, result)) => {
            let trimmed_remaining = remaining.fragment().trim();
            if !trimmed_remaining.is_empty() {
                let remaining_after_whitespace =
                    match multispace0::<Span, nom::error::Error<Span>>(remaining) {
                        Ok((after_ws, _)) => after_ws,
                        Err(_) => remaining,
                    };

                let line = remaining_after_whitespace.location_line();
                let column = remaining_after_whitespace.get_utf8_column();
                let source_line = trimmed_input
                    .lines()
                    .nth((line - 1) as usize)
                    .unwrap_or("")
                    .to_string();

                return Err(ParserError::SyntaxError {
                    message: "Unexpected input after database query".to_string(),
                    line,
                    column,
                    context_path: vec!["query".to_string(), "database".to_string()],
                    source_line,
                    pointer: format!("{}^", " ".repeat(column.saturating_sub(1))),
                    suggestion: None,
                });
            }
            Ok(result)
        }
        Err(e) => Err(convert_error(
            trimmed_input,
            vec!["query".to_string(), "database".to_string()],
            e,
        )),
    }
}
