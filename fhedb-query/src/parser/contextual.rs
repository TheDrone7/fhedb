//! # Contextual Query Parser
//!
//! This module provides parsing functionality for contextual FHEDB queries
//! (collections and documents).

use chumsky::{extra, input::ValueInput, prelude::*};

use crate::ast::ContextualQuery;
use crate::error::ParserError;
use crate::lexer::{Span, Token};

use super::collection::collection_query_parser;
use super::common::lex_input;

fn contextual_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ContextualQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((collection_query_parser().map(ContextualQuery::Collection),)).then_ignore(end())
}

/// Parses a contextual query string into a [`ContextualQuery`] AST node.
///
/// ## Arguments
///
/// * `input` - The query string to parse.
///
/// ## Returns
///
/// Returns [`Ok`]([`ContextualQuery`]) if parsing succeeds,
/// or [`Err`]([`Vec<ParserError>`]) containing all parsing errors if it fails.
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
