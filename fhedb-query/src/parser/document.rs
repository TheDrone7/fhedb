//! Document query parsing functionality.
//!
//! This module provides functions to parse and handle document queries.

use std::collections::HashMap;

use crate::{
    ast::*,
    error::ParseError,
    parser::utilities::{ParseResult, identifier, trim_parse},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{tag_no_case, take_until},
    character::complete::{char, multispace0, multispace1},
    combinator::map_res,
    sequence::{delimited, preceded},
};

/// Parses an INSERT DOCUMENT query.
///
/// ## Arguments
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DocumentQuery::Insert`].
fn insert_document(input: &str) -> IResult<&str, DocumentQuery> {
    map_res(
        (
            tag_no_case("insert"),
            multispace1,
            alt((tag_no_case("doc"), tag_no_case("document"))),
            multispace1,
            tag_no_case("into"),
            multispace1,
            identifier,
            multispace0,
            delimited(char('{'), take_until("}"), char('}')),
        ),
        |(_, _, _, _, _, _, collection_name, _, _doc)| -> Result<DocumentQuery, ParseError> {
            Ok(DocumentQuery::Insert {
                collection_name: collection_name.to_string(),
                fields: HashMap::new(),
            })
        },
    )
    .parse(input)
}

/// Parses a complete document query from the input string.
///
/// ## Arguments
///
/// * `input` - The input string containing the document query to parse.
///
/// ## Returns
/// Returns a [`ParseResult<DocumentQuery>`] containing the parsed query on success
/// or a [`ParseError`] on failure.
pub fn parse_document_query(input: &str) -> ParseResult<DocumentQuery> {
    trim_parse(input, "document query", |input| {
        preceded(multispace0, insert_document).parse(input)
    })
}
