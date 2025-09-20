//! Document query parsing functionality.
//!
//! This module provides functions to parse and handle document queries.

use crate::{
    ast::*,
    error::ParseError,
    parser::utilities::{ParseResult, balanced_braces_content, identifier, trim_parse},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{char, multispace0, multispace1},
    combinator::map_res,
    sequence::{delimited, preceded},
};

mod helpers;
use helpers::parse_document_fields;

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
            alt((
                (tag_no_case("doc"), multispace1),
                (tag_no_case("document"), multispace1),
            )),
            tag_no_case("into"),
            multispace1,
            identifier,
            multispace0,
            delimited(char('{'), balanced_braces_content, char('}')),
        ),
        |(_, _, _, _, _, collection_name, _, doc_content)| -> Result<DocumentQuery, ParseError> {
            let fields = parse_document_fields(doc_content)?;
            Ok(DocumentQuery::Insert {
                collection_name: collection_name.to_string(),
                fields,
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
