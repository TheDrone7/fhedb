//! Database query parsing functionality.
//!
//! This module provides parsers for database operation queries such as CREATE DATABASE and DROP DATABASE.

use crate::{
    ast::*,
    parser::utilities::{ParseResult, identifier, trim_parse},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    sequence::preceded,
};

/// Parses a CREATE DATABASE query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::Create`].
fn create_database(input: &str) -> IResult<&str, DatabaseQuery> {
    map(
        (
            tag_no_case("create"),
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
        ),
        |(_, _, _, _, name, drop_if_exists)| DatabaseQuery::Create {
            name: name.to_string(),
            drop_if_exists: drop_if_exists.is_some(),
        },
    )
    .parse(input)
}

/// Parses a DROP DATABASE query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::Drop`].
fn drop_database(input: &str) -> IResult<&str, DatabaseQuery> {
    map(
        (
            tag_no_case("drop"),
            multispace1,
            tag_no_case("database"),
            multispace1,
            identifier,
        ),
        |(_, _, _, _, name)| DatabaseQuery::Drop {
            name: name.to_string(),
        },
    )
    .parse(input)
}

/// Parses a LIST DATABASES query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DatabaseQuery::List`].
fn list_databases(input: &str) -> IResult<&str, DatabaseQuery> {
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
/// Returns a [`ParseResult<DatabaseQuery>`] containing the parsed query on success,
/// or a [`ParseError`] if the parsing fails.
///
/// ## Errors
///
/// This function returns a [`ParseError::SyntaxError`] in the following cases:
/// - The input doesn't match any known database query pattern
/// - There is unexpected input remaining after a valid query
pub fn parse_database_query(input: &str) -> ParseResult<DatabaseQuery> {
    trim_parse(input, "database query", |input| {
        preceded(
            multispace0,
            alt((create_database, drop_database, list_databases)),
        )
        .parse(input)
    })
}
