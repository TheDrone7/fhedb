//! Collection query parsing functionality.
//!
//! This module provides parsers for collection operation queries such as CREATE or DROP COLLECTION.

use crate::{
    ast::*,
    error::ParseError,
    parser::{
        schema::parse_schema,
        utilities::{ParseResult, identifier},
    },
};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    sequence::{delimited, preceded},
};

/// Parses a CREATE COLLECTION query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`CollectionQuery::Create`].
fn create_collection(input: &str) -> IResult<&str, CollectionQuery> {
    map(
        (
            tag("collection"),
            multispace1,
            identifier,
            multispace1,
            tag("create"),
            opt(preceded(
                multispace1,
                (
                    tag("drop"),
                    multispace1,
                    tag("if"),
                    multispace1,
                    tag("exists"),
                ),
            )),
            multispace0,
            delimited(char('{'), parse_schema, char('}')),
        ),
        |(_, _, name, _, _, drop_if_exists, _, schema)| CollectionQuery::Create {
            name: name.to_string(),
            drop_if_exists: drop_if_exists.is_some(),
            schema,
        },
    )
    .parse(input)
}

/// Parses a complete collection query from the input string.
///
/// ## Arguments
///
/// * `input` - The input string containing the collection query to parse.
///
/// ## Returns
///
/// Returns a [`ParseResult<CollectionQuery>`] containing the parsed query on success,
/// or a [`ParseError`] if the parsing fails.
pub fn parse_collection_query(input: &str) -> ParseResult<CollectionQuery> {
    let input = input.trim();

    let (remaining, query) = preceded(multispace0, create_collection)
        .parse(input)
        .map_err(|_| ParseError::SyntaxError {
            message: "Unknown collection query".to_string(),
        })?;

    if !remaining.trim().is_empty() {
        return Err(ParseError::SyntaxError {
            message: "Unexpected input after query".to_string(),
        });
    }

    Ok(query)
}
