//! Database query parsing functionality.
//!
//! This module provides parsers for database operation queries such as CREATE DATABASE and DROP DATABASE.

use crate::{
    ast::*,
    error::ParseError,
    parser::utilities::{ParseResult, identifier},
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
/// This function parses a CREATE DATABASE statement, which creates a new database
/// with the specified name. It also supports an optional "DROP IF EXISTS" clause.
///
/// ## Syntax
///
/// ```text
/// CREATE DATABASE <database_name> [DROP IF EXISTS]
/// ```
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
/// This function parses a DROP DATABASE statement, which removes an existing database
/// with the specified name.
///
/// ## Syntax
///
/// ```text
/// DROP DATABASE <database_name>
/// ```
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
    let input = input.trim();

    let (remaining, query) = preceded(multispace0, alt((create_database, drop_database)))
        .parse(input)
        .map_err(|_| ParseError::SyntaxError {
            message: "Unknown database query".to_string(),
        })?;

    if !remaining.trim().is_empty() {
        return Err(ParseError::SyntaxError {
            message: "Unexpected input after query".to_string(),
        });
    }

    Ok(query)
}
