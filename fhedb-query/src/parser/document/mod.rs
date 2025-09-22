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
use helpers::parse_doc_content;

/// Parses a GET DOCUMENT query.
///
/// ## Arguments
/// * `input` - The input string to parse.
///
/// ## Returns
///
/// Returns an [`IResult`] containing the remaining input and the parsed [`DocumentQuery::Get`].
fn get_document(input: &str) -> IResult<&str, DocumentQuery> {
    map_res(
        (
            tag_no_case("get"),
            multispace1,
            alt((
                (tag_no_case("doc"), multispace1),
                (tag_no_case("docs"), multispace1),
                (tag_no_case("document"), multispace1),
                (tag_no_case("documents"), multispace1),
            )),
            tag_no_case("from"),
            multispace1,
            identifier,
            multispace0,
            delimited(char('{'), balanced_braces_content, char('}')),
        ),
        |(_, _, _, _, _, collection_name, _, doc_content)| -> Result<DocumentQuery, ParseError> {
            let ParsedDocContent {
                assignments,
                conditions,
                selectors: field_selector,
            } = parse_doc_content(doc_content)?;
            if !assignments.is_empty() {
                return Err(ParseError::SyntaxError {
                    message: "Assignments are not allowed in GET DOCUMENT queries".to_string(),
                });
            }
            if conditions.is_empty() && field_selector.is_empty() {
                return Err(ParseError::SyntaxError {
                    message: "GET queries must specify at least one condition or field selector"
                        .to_string(),
                });
            }
            Ok(DocumentQuery::Get {
                collection_name: collection_name.to_string(),
                conditions,
                field_selector,
            })
        },
    )
    .parse(input)
}

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
                (tag_no_case("docs"), multispace1),
                (tag_no_case("document"), multispace1),
                (tag_no_case("documents"), multispace1),
            )),
            tag_no_case("into"),
            multispace1,
            identifier,
            multispace0,
            delimited(char('{'), balanced_braces_content, char('}')),
        ),
        |(_, _, _, _, _, collection_name, _, doc_content)| -> Result<DocumentQuery, ParseError> {
            let ParsedDocContent {
                assignments,
                conditions,
                selectors,
            } = parse_doc_content(doc_content)?;
            if !conditions.is_empty() || !selectors.is_empty() {
                return Err(ParseError::SyntaxError {
                    message: "Only assignment operations allowed in INSERT DOCUMENT queries"
                        .to_string(),
                });
            }
            Ok(DocumentQuery::Insert {
                collection_name: collection_name.to_string(),
                fields: assignments,
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
        preceded(multispace0, alt((get_document, insert_document))).parse(input)
    })
}
