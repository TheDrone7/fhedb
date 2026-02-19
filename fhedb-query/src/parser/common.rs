//! # Common Parser Utilities
//!
//! This module provides common utilities for FHEDB query parsers.

use bson::Bson;
use chumsky::{extra, input::ValueInput, prelude::*};
use fhedb_core::schema::FieldType;

use crate::{
    error::ParserError,
    lexer::{Span, Token, lexer},
};

/// Lexes the input string into a vector of tokens.
///
/// ## Arguments
///
/// * `input` - The input string to lex.
pub(crate) fn lex_input(input: &str) -> Result<Vec<(Token, Span)>, Vec<ParserError>> {
    let (tokens, lex_errs) = lexer().parse(input).into_output_errors();

    if !lex_errs.is_empty() {
        return Err(lex_errs
            .iter()
            .map(|e| ParserError::from_lexer_rich(e, input))
            .collect());
    }

    Ok(tokens.unwrap())
}

/// Creates a parser that matches an identifier token.
///
/// ## Arguments
///
/// * `label` - The label to use for error messages.
pub(crate) fn identifier_parser<'tokens, 'src: 'tokens, I>(
    label: &'static str,
) -> impl Parser<'tokens, I, String, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    select! { Token::Ident(name) => name }.labelled(label)
}

/// Creates a parser for the `DROP IF EXISTS` clause.
pub(crate) fn drop_if_exists_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Option<()>, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Drop)
        .ignore_then(just(Token::If))
        .ignore_then(just(Token::Exists))
        .to(())
        .or_not()
}

/// Creates a parser for [`Bson`] values.
pub(crate) fn bson_value_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Bson, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|bson_value| {
        let atom = select! {
            Token::StringLit(s) => Bson::String(s),
            Token::IntLit(n) => Bson::Int64(n),
            Token::FloatLit(s) => Bson::Double(s.parse().unwrap()),
            Token::True => Bson::Boolean(true),
            Token::False => Bson::Boolean(false),
            Token::Null => Bson::Null,
        };

        let array = just(Token::OpenBracket)
            .ignore_then(
                bson_value
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::CloseBracket))
            .map(Bson::Array);

        choice((array, atom))
    })
}

/// Creates a parser for field type modifiers (nullable, default value).
pub(crate) fn field_modifier_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (bool, Option<Bson>), extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let nullable = just(Token::Nullable).to(true).labelled("nullable");

    let default = just(Token::Default)
        .ignore_then(just(Token::Equals))
        .ignore_then(bson_value_parser().labelled("default value"))
        .labelled("default");

    just(Token::OpenParen)
        .ignore_then(
            nullable
                .then(just(Token::Comma).ignore_then(default.clone()).or_not())
                .map(|(_, default)| (true, default))
                .or(default.map(|d| (false, Some(d)))),
        )
        .then_ignore(just(Token::CloseParen))
        .labelled("field constraint")
        .as_context()
}

/// Creates a parser for field types.
pub(crate) fn field_type_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FieldType, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|field_type| {
        let simple_type = select! {
            Token::TypeInt => FieldType::Int,
            Token::TypeFloat => FieldType::Float,
            Token::TypeBoolean => FieldType::Boolean,
            Token::TypeString => FieldType::String,
        }
        .labelled("field type");

        let ref_type = just(Token::TypeRef)
            .ignore_then(just(Token::OpenAngle))
            .ignore_then(identifier_parser("collection name"))
            .then_ignore(just(Token::CloseAngle))
            .map(FieldType::Reference)
            .labelled("reference type");

        let array_type = just(Token::TypeArray)
            .ignore_then(just(Token::OpenAngle))
            .ignore_then(field_type.clone())
            .then_ignore(just(Token::CloseAngle))
            .map(|inner| FieldType::Array(Box::new(inner)))
            .labelled("array type");

        choice((array_type, ref_type, simple_type))
    })
    .labelled("field type")
    .as_context()
}
