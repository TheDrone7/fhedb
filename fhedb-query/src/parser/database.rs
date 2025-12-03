//! Database query parsing functionality.
//!
//! This module provides parsers for database operation queries such as CREATE DATABASE and DROP DATABASE.

use crate::{
    ast::*,
    error::Span,
    parser::utilities::{ParserResult, identifier, parse_subcommand},
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
            cut((
                multispace1,
                tag_no_case("database"),
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
        |(_, (_, _, _, name, drop_if_exists))| DatabaseQuery::Create {
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
            cut((
                multispace1,
                tag_no_case("database"),
                multispace1,
                identifier,
            )),
        ),
        |(_, (_, _, _, name))| DatabaseQuery::Drop {
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
        (
            tag_no_case("list"),
            cut((multispace1, tag_no_case("databases"))),
        ),
        |(_, _)| DatabaseQuery::List,
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
    parse_subcommand(
        input,
        "database query",
        |span| {
            preceded(
                multispace0,
                alt((create_database, drop_database, list_databases)),
            )
            .parse(span)
        },
        |query| match query {
            DatabaseQuery::Create { .. } => "create database",
            DatabaseQuery::Drop { .. } => "drop database",
            DatabaseQuery::List => "list databases",
        },
        determine_context_from_error,
    )
}

/// Determines the parsing context based on the input and error information.
///
/// ## Arguments
///
/// * `input` - The input string being parsed.
/// * `error` - The nom error that occurred during parsing.
///
/// ## Returns
///
/// Returns a vector of context strings representing the parsing hierarchy.
fn determine_context_from_error(
    input: &str,
    error: &nom::Err<nom::error::Error<Span>>,
) -> Vec<String> {
    let input_upper = input.trim().to_uppercase();
    let mut context = vec!["database query".to_string()];

    let error_column = match error {
        nom::Err::Error(e) | nom::Err::Failure(e) => e.input.get_utf8_column(),
        nom::Err::Incomplete(_) => 1,
    };

    let is_failure = matches!(error, nom::Err::Failure(_));

    if input_upper.starts_with("CREATE") && (error_column > 6 || is_failure) {
        context.push("create database".to_string());
    } else if input_upper.starts_with("DROP") && (error_column > 4 || is_failure) {
        context.push("drop database".to_string());
    } else if input_upper.starts_with("LIST") && (error_column > 4 || is_failure) {
        context.push("list databases".to_string());
    }

    context
}
