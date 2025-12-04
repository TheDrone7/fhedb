//! # Collection Query Parser
//!
//! This module provides parsing functionality for collection-level FHEDB queries.

use chumsky::{extra, input::ValueInput, prelude::*};
use fhedb_core::db::schema::{validate_bson_type, FieldDefinition, FieldType, Schema};

use crate::ast::CollectionQuery;
use crate::lexer::{Span, Token};
use crate::utilities::bson_value_parser_internal;

fn field_modifier_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (bool, Option<bson::Bson>), extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let nullable = select! { Token::Ident(s) if s.eq_ignore_ascii_case("nullable") => () }
        .to(true)
        .labelled("nullable");

    let default = select! { Token::Ident(s) if s.eq_ignore_ascii_case("default") => () }
        .ignore_then(just(Token::Equals))
        .ignore_then(bson_value_parser_internal().labelled("default value"))
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

fn field_type_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FieldType, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|field_type| {
        let simple_type = select! {
            Token::Ident(s) if s.eq_ignore_ascii_case("int") => FieldType::Int,
            Token::Ident(s) if s.eq_ignore_ascii_case("float") => FieldType::Float,
            Token::Ident(s) if s.eq_ignore_ascii_case("boolean") => FieldType::Boolean,
            Token::Ident(s) if s.eq_ignore_ascii_case("string") => FieldType::String,
        }
        .labelled("field type");

        let ref_type = select! { Token::Ident(s) if s.eq_ignore_ascii_case("ref") => () }
            .ignore_then(just(Token::OpenAngle))
            .ignore_then(select! { Token::Ident(name) => name }.labelled("collection name"))
            .then_ignore(just(Token::CloseAngle))
            .map(FieldType::Reference)
            .labelled("reference type");

        let array_type = select! { Token::Ident(s) if s.eq_ignore_ascii_case("array") => () }
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

fn field_definition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (String, FieldDefinition), extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let id_type = select! {
        Token::Ident(s) if s.eq_ignore_ascii_case("id_string") => FieldType::IdString,
        Token::Ident(s) if s.eq_ignore_ascii_case("id_int") => FieldType::IdInt,
    }
    .labelled("id type")
    .map(|ft| (ft, None));

    let regular_type = field_type_parser()
        .then(field_modifier_parser().or_not())
        .map(|(ft, modifier)| (ft, modifier));

    let type_and_modifier = choice((id_type, regular_type)).labelled("field type");

    select! { Token::Ident(name) => name }
        .labelled("field name")
        .then_ignore(just(Token::Colon))
        .then(type_and_modifier)
        .labelled("field type")
        .as_context()
        .try_map(|(name, (field_type, modifier)), span| {
            let (nullable, default) = modifier.unwrap_or((false, None));
            let base_type = if nullable {
                FieldType::Nullable(Box::new(field_type))
            } else {
                field_type
            };
            if let Some(ref default_value) = default {
                validate_bson_type(default_value, &base_type)
                    .map_err(|e| Rich::custom(span, format!("invalid default value: {}", e)))?;
            }
            let field_def = FieldDefinition::with_optional_default(base_type, default);
            Ok((name, field_def))
        })
}

fn schema_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Schema, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    field_definition_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .map(|fields| {
            let mut schema = Schema::new();
            for (name, def) in fields {
                schema.fields.insert(name, def);
            }
            schema
        })
        .labelled("schema")
        .as_context()
}

fn create_collection_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Create)
        .ignore_then(just(Token::Collection))
        .ignore_then(select! { Token::Ident(name) => name }.labelled("collection name"))
        .then(
            just(Token::Drop)
                .ignore_then(just(Token::If))
                .ignore_then(just(Token::Exists))
                .or_not(),
        )
        .then(schema_parser())
        .map(|((name, drop_if_exists), schema)| CollectionQuery::Create {
            name,
            drop_if_exists: drop_if_exists.is_some(),
            schema,
        })
        .labelled("create collection")
        .as_context()
}

pub(crate) fn collection_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((create_collection_parser(),))
        .labelled("collection query")
        .as_context()
}
