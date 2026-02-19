//! # Collection Query Parser
//!
//! This module provides parsing functionality for collection-level FHEDB queries.

use chumsky::{extra, input::ValueInput, prelude::*};
use fhedb_core::schema::{FieldDefinition, FieldType, Schema, validate_bson_type};

use crate::lexer::{Span, Token};
use fhedb_types::{CollectionQuery, FieldModification};

use super::common::{
    drop_if_exists_parser, field_modifier_parser, field_type_parser, identifier_parser,
};

/// Parses a field definition in a collection schema.
///
/// ## Returns
///
/// Returns a parser that matches field definitions and returns a tuple of
/// (field_name, [`FieldDefinition`]).
fn field_definition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (String, FieldDefinition), extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let id_type = select! {
        Token::TypeIdString => FieldType::IdString,
        Token::TypeIdInt => FieldType::IdInt,
    }
    .labelled("id type")
    .map(|ft| (ft, None));

    let regular_type = field_type_parser()
        .then(field_modifier_parser().or_not())
        .map(|(ft, modifier)| (ft, modifier));

    let type_and_modifier = choice((id_type, regular_type)).labelled("field type");

    identifier_parser("field name")
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

/// Parses a collection schema enclosed in braces.
///
/// ## Returns
///
/// Returns a parser that matches schemas and returns a [`Schema`].
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
        .try_map(|fields, span| {
            let mut schema = Schema::new();
            for (name, def) in fields {
                if schema.fields.contains_key(&name) {
                    return Err(Rich::custom(
                        span,
                        format!("duplicate field name: {}", name),
                    ));
                }
                schema.fields.insert(name, def);
            }
            Ok(schema)
        })
        .labelled("schema")
        .as_context()
}

/// Parses a CREATE COLLECTION query.
///
/// ## Returns
///
/// Returns a parser that matches CREATE COLLECTION queries and returns a [`CollectionQuery::Create`].
fn create_collection_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Create)
        .ignore_then(just(Token::Collection))
        .ignore_then(identifier_parser("collection name"))
        .then(drop_if_exists_parser())
        .then(schema_parser())
        .map(|((name, drop_if_exists), schema)| CollectionQuery::Create {
            name,
            drop_if_exists: drop_if_exists.is_some(),
            schema,
        })
        .labelled("create collection")
        .as_context()
}

/// Parses a DROP COLLECTION query.
///
/// ## Returns
///
/// Returns a parser that matches DROP COLLECTION queries and returns a [`CollectionQuery::Drop`].
fn drop_collection_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Drop)
        .ignore_then(just(Token::Collection))
        .ignore_then(identifier_parser("collection name"))
        .map(|name| CollectionQuery::Drop { name })
        .labelled("drop collection")
        .as_context()
}

/// Parses a LIST COLLECTIONS query.
///
/// ## Returns
///
/// Returns a parser that matches LIST COLLECTIONS queries and returns [`CollectionQuery::List`].
fn list_collections_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::List)
        .ignore_then(just(Token::Collections))
        .to(CollectionQuery::List)
        .labelled("list collections")
        .as_context()
}

/// Parses a GET SCHEMA query.
///
/// ## Returns
///
/// Returns a parser that matches GET SCHEMA queries and returns a [`CollectionQuery::GetSchema`].
fn get_schema_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Get)
        .ignore_then(just(Token::Schema))
        .ignore_then(just(Token::From))
        .ignore_then(identifier_parser("collection name"))
        .map(|name| CollectionQuery::GetSchema { name })
        .labelled("get collection schema")
        .as_context()
}

/// Parses a field modification in a MODIFY COLLECTION query.
///
/// ## Returns
///
/// Returns a parser that matches field modifications and returns a tuple of
/// (field_name, [`FieldModification`]).
fn field_modification_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (String, FieldModification), extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let drop_modification = just(Token::Drop)
        .to(FieldModification::Drop)
        .labelled("drop");

    let id_type = select! {
        Token::TypeIdString => FieldType::IdString,
        Token::TypeIdInt => FieldType::IdInt,
    }
    .labelled("id type")
    .map(|ft| (ft, None));

    let regular_type = field_type_parser()
        .then(field_modifier_parser().or_not())
        .map(|(ft, modifier)| (ft, modifier));

    let type_and_modifier = choice((id_type, regular_type)).labelled("field type");

    let set_modification = type_and_modifier.try_map(|(field_type, modifier), span| {
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
        Ok(FieldModification::Set(field_def))
    });

    identifier_parser("field name")
        .then_ignore(just(Token::Colon))
        .then(choice((drop_modification, set_modification)))
        .labelled("field modification")
        .as_context()
}

/// Parses a modification schema enclosed in braces.
///
/// ## Returns
///
/// Returns a parser that matches modification schemas and returns a [`HashMap`] of field modifications.
fn modification_schema_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<
    'tokens,
    I,
    std::collections::HashMap<String, FieldModification>,
    extra::Err<Rich<'tokens, Token, Span>>,
> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    field_modification_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .try_map(|fields, span| {
            let mut modifications = std::collections::HashMap::new();
            for (name, modification) in fields {
                if modifications.contains_key(&name) {
                    return Err(Rich::custom(
                        span,
                        format!("duplicate field name: {}", name),
                    ));
                }
                modifications.insert(name, modification);
            }
            Ok(modifications)
        })
        .labelled("modification schema")
        .as_context()
}

/// Parses a MODIFY COLLECTION query.
///
/// ## Returns
///
/// Returns a parser that matches MODIFY COLLECTION queries and returns a [`CollectionQuery::Modify`].
fn modify_collection_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((just(Token::Modify), just(Token::Alter)))
        .ignore_then(just(Token::Collection))
        .ignore_then(identifier_parser("collection name"))
        .then(modification_schema_parser())
        .map(|(name, modifications)| CollectionQuery::Modify {
            name,
            modifications,
        })
        .labelled("modify collection")
        .as_context()
}

/// Creates a parser for collection-level queries.
///
/// ## Returns
///
/// Returns a parser that matches collection queries and returns a [`CollectionQuery`].
pub(crate) fn collection_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, CollectionQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((
        create_collection_parser(),
        drop_collection_parser(),
        list_collections_parser(),
        get_schema_parser(),
        modify_collection_parser(),
    ))
    .labelled("collection query")
    .as_context()
}
