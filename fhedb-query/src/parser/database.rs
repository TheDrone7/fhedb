//! # Database Query Parser
//!
//! This module provides parsing functionality for database-level FHEDB queries.

use chumsky::{extra, input::ValueInput, prelude::*};

use crate::ast::DatabaseQuery;
use crate::error::ParserError;
use crate::lexer::{Span, Token, lexer};

/// Creates a parser for database-level queries.
///
/// Parses CREATE DATABASE, DROP DATABASE, and LIST DATABASES queries.
fn database_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DatabaseQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let create_db = just(Token::Create)
        .ignore_then(just(Token::Database))
        .ignore_then(select! { Token::Ident(name) => name }.labelled("database name"))
        .then(
            just(Token::Drop)
                .ignore_then(just(Token::If))
                .ignore_then(just(Token::Exists))
                .or_not(),
        )
        .map(|(name, drop_if_exists)| DatabaseQuery::Create {
            name,
            drop_if_exists: drop_if_exists.is_some(),
        })
        .labelled("create database")
        .as_context();

    let drop_db = just(Token::Drop)
        .ignore_then(just(Token::Database))
        .ignore_then(select! { Token::Ident(name) => name }.labelled("database name"))
        .map(|name| DatabaseQuery::Drop { name })
        .labelled("drop database")
        .as_context();

    let list_dbs = just(Token::List)
        .ignore_then(just(Token::Databases))
        .to(DatabaseQuery::List)
        .labelled("list databases")
        .as_context();

    choice((create_db, drop_db, list_dbs))
        .labelled("database query")
        .as_context()
        .then_ignore(end())
}

/// Parses a database query string into a [`DatabaseQuery`] AST node.
///
/// ## Arguments
///
/// * `input` - The query string to parse.
///
/// ## Returns
///
/// Returns [`Ok`]([`DatabaseQuery`]) if parsing succeeds,
/// or [`Err`]([`Vec<ParserError>`]) containing all parsing errors if it fails.
pub fn parse_database_query(input: &str) -> Result<DatabaseQuery, Vec<ParserError>> {
    let (tokens, lex_errs) = lexer().parse(input).into_output_errors();

    if !lex_errs.is_empty() {
        return Err(lex_errs
            .iter()
            .map(|e| ParserError::from_lexer_rich(e, input))
            .collect());
    }

    let tokens = tokens.unwrap();
    let len = input.len();
    let eoi = Span::new((), len..len);

    let (result, parse_errs) = database_query_parser()
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
