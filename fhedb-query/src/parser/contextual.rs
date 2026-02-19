//! # Contextual Query Parser
//!
//! This module provides parsing functionality for contextual FHEDB queries.

use chumsky::{extra, input::ValueInput, prelude::*};
use fhedb_types::ContextualQuery;

use crate::{
    error::ParserError,
    lexer::{Span, Token},
    parser::{
        collection::collection_query_parser, common::lex_input, document::document_query_parser,
    },
};

/// Creates a parser for contextual queries.
fn contextual_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ContextualQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((
        collection_query_parser().map(ContextualQuery::Collection),
        document_query_parser().map(ContextualQuery::Document),
    ))
    .then_ignore(end())
}

/// Parses a contextual query string into a [`ContextualQuery`] AST node.
///
/// ## Arguments
///
/// * `input` - The query string to parse.
pub fn parse_contextual_query(input: &str) -> Result<ContextualQuery, Vec<ParserError>> {
    let tokens = lex_input(input)?;
    let len = input.len();
    let eoi = Span::new((), len..len);

    let (result, parse_errs) = contextual_query_parser()
        .parse(tokens.as_slice().map(eoi, |(t, s)| (t, s)))
        .into_output_errors();

    if !parse_errs.is_empty() {
        return Err(parse_errs
            .iter()
            .map(|e| ParserError::from_rich(e, input))
            .collect());
    }

    Ok(result.unwrap())
}
