//! Collection query parsing functionality.
//!
//! This module provides parsers for collection operation queries such as CREATE or DROP COLLECTION.

use crate::{
    ast::*,
    error::ParseError,
    parser::{
        schema::{parse_field_modifications, parse_schema},
        utilities::{ParseResult, balanced_braces_content, identifier, trim_parse},
    },
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{char, multispace0, multispace1},
    combinator::{map, map_res, opt},
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
    map_res(
        (
            tag_no_case("create"),
            multispace1,
            tag_no_case("collection"),
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
            multispace0,
            delimited(char('{'), balanced_braces_content, char('}')),
        ),
        |(_, _, _, _, name, drop_if_exists, _, schema_text)| -> Result<CollectionQuery, ParseError> {
            let schema = parse_schema(schema_text)?;
            Ok(CollectionQuery::Create {
                name: name.to_string(),
                drop_if_exists: drop_if_exists.is_some(),
                schema,
            })
        },
    )
    .parse(input)
}

/// Parses a DROP COLLECTION query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`CollectionQuery::Drop`].
fn drop_collection(input: &str) -> IResult<&str, CollectionQuery> {
    map(
        (
            tag_no_case("drop"),
            multispace1,
            tag_no_case("collection"),
            multispace1,
            identifier,
        ),
        |(_, _, _, _, name)| CollectionQuery::Drop {
            name: name.to_string(),
        },
    )
    .parse(input)
}

/// Parses a MODIFY COLLECTION query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`CollectionQuery::Modify`].
fn modify_collection(input: &str) -> IResult<&str, CollectionQuery> {
    map_res(
        (
            alt((tag_no_case("modify"), tag_no_case("alter"))),
            multispace1,
            tag_no_case("collection"),
            multispace1,
            identifier,
            multispace0,
            delimited(char('{'), balanced_braces_content, char('}')),
        ),
        |(_, _, _, _, name, _, modifications_text)| -> Result<CollectionQuery, ParseError> {
            let modifications = parse_field_modifications(modifications_text)?;
            Ok(CollectionQuery::Modify {
                name: name.to_string(),
                modifications,
            })
        },
    )
    .parse(input)
}

/// Parses a LIST COLLECTIONS query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`CollectionQuery::List`].
fn list_collections(input: &str) -> IResult<&str, CollectionQuery> {
    map(
        (tag_no_case("list"), multispace1, tag_no_case("collections")),
        |(_, _, _)| CollectionQuery::List,
    )
    .parse(input)
}

/// Parses a GET SCHEMA FROM query.
///
/// ## Arguments
///
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`CollectionQuery::GetSchema`].
fn get_collection_schema(input: &str) -> IResult<&str, CollectionQuery> {
    map(
        (
            tag_no_case("get"),
            multispace1,
            tag_no_case("schema"),
            multispace1,
            tag_no_case("from"),
            multispace1,
            identifier,
        ),
        |(_, _, _, _, _, _, name)| CollectionQuery::GetSchema {
            name: name.to_string(),
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
    trim_parse(input, "collection query", |input| {
        preceded(
            multispace0,
            alt((
                create_collection,
                drop_collection,
                modify_collection,
                list_collections,
                get_collection_schema,
            )),
        )
        .parse(input)
    })
}
