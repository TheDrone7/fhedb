//! # Document Query Parser
//!
//! This module provides parsing functionality for document-level FHEDB queries.

use std::collections::HashMap;

use chumsky::{extra, input::ValueInput, prelude::*};

use crate::ast::{DocumentQuery, FieldCondition, FieldSelector, ParsedDocContent, QueryOperator};
use crate::lexer::{Span, Token};

use super::common::identifier_parser;

/// Parses a value that can be used in assignments and conditions.
///
/// ## Returns
///
/// Returns a parser that matches values and returns their string representation.
fn value_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, String, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|value| {
        let atom = select! {
            Token::StringLit(s) => format!("\"{}\"", s),
            Token::IntLit(n) => n.to_string(),
            Token::FloatLit(s) => s,
            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::Null => "null".to_string(),
            Token::Ident(s) => s,
        };

        let array = value
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket))
            .map(|items| format!("[{}]", items.join(", ")));

        choice((array, atom))
    })
    .labelled("value")
}

/// Parses a query operator used in conditions.
///
/// ## Returns
///
/// Returns a parser that matches operators and returns the corresponding [`QueryOperator`].
fn operator_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, QueryOperator, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((
        just(Token::DoubleEquals).to(QueryOperator::Similar),
        just(Token::NotEquals).to(QueryOperator::NotEqual),
        just(Token::LessThanOrEqual).to(QueryOperator::LessThanOrEqual),
        just(Token::GreaterThanOrEqual).to(QueryOperator::GreaterThanOrEqual),
        just(Token::Equals).to(QueryOperator::Equal),
        just(Token::OpenAngle).to(QueryOperator::LessThan),
        just(Token::CloseAngle).to(QueryOperator::GreaterThan),
    ))
    .labelled("operator")
}

/// Represents the different types of items that can appear in a document body.
#[derive(Debug, Clone)]
enum DocumentBodyItem {
    /// A field assignment (field: value).
    Assignment(String, String),
    /// A field condition (field operator value).
    Condition(FieldCondition),
    /// A field selector (field name, wildcard, or sub-document).
    Selector(FieldSelector),
}

/// Parses a field selector in a document body.
///
/// ## Arguments
///
/// * `body_parser` - A parser for nested document body content.
///
/// ## Returns
///
/// Returns a parser that matches field selectors and returns a [`DocumentBodyItem::Selector`].
fn document_selector_parser<'tokens, 'src: 'tokens, I>(
    body_parser: impl Parser<'tokens, I, ParsedDocContent, extra::Err<Rich<'tokens, Token, Span>>>
    + Clone
    + 'tokens,
) -> impl Parser<'tokens, I, DocumentBodyItem, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let all_fields_recursive = just(Token::DoubleStar).to(FieldSelector::AllFieldsRecursive);
    let all_fields = just(Token::Star).to(FieldSelector::AllFields);

    let sub_document =
        identifier_parser("field name")
            .then(body_parser)
            .try_map(|(field_name, content), span| {
                if !content.assignments.is_empty() {
                    return Err(Rich::custom(
                        span,
                        "assignments are not allowed in sub-document selectors",
                    ));
                }
                fn check_nested_assignments(selectors: &[FieldSelector]) -> Option<&str> {
                    for selector in selectors {
                        if let FieldSelector::SubDocument { content, .. } = selector {
                            if !content.assignments.is_empty() {
                                return Some(
                                    "assignments are not allowed in sub-document selectors",
                                );
                            }
                            if let Some(err) = check_nested_assignments(&content.selectors) {
                                return Some(err);
                            }
                        }
                    }
                    None
                }
                if let Some(err) = check_nested_assignments(&content.selectors) {
                    return Err(Rich::custom(span, err));
                }
                Ok(FieldSelector::SubDocument {
                    field_name,
                    content,
                })
            });

    let simple_field = identifier_parser("field name").map(FieldSelector::Field);

    choice((all_fields_recursive, all_fields, sub_document, simple_field))
        .map(DocumentBodyItem::Selector)
        .labelled("field selector")
}

/// Parses a field condition in a document body.
///
/// ## Returns
///
/// Returns a parser that matches field conditions and returns a [`DocumentBodyItem::Condition`].
fn document_condition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentBodyItem, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    identifier_parser("field name")
        .then(operator_parser())
        .then(value_parser())
        .map(|((field_name, operator), value)| {
            DocumentBodyItem::Condition(FieldCondition {
                field_name,
                operator,
                value,
            })
        })
        .labelled("field condition")
}

/// Parses a field assignment in a document body.
///
/// ## Returns
///
/// Returns a parser that matches field assignments and returns a [`DocumentBodyItem::Assignment`].
fn document_assignment_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentBodyItem, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    identifier_parser("field name")
        .then_ignore(just(Token::Colon))
        .then(value_parser())
        .map(|(field_name, value)| DocumentBodyItem::Assignment(field_name, value))
        .labelled("field assignment")
}

/// Parses a document body enclosed in braces.
///
/// ## Returns
///
/// Returns a parser that matches document bodies and returns a [`ParsedDocContent`].
fn document_body_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ParsedDocContent, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|body| {
        choice((
            document_assignment_parser(),
            document_condition_parser(),
            document_selector_parser(body),
        ))
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .try_map(|items, span| {
            let mut assignments = HashMap::new();
            let mut conditions = Vec::new();
            let mut selectors = Vec::new();

            for item in items {
                match item {
                    DocumentBodyItem::Assignment(field, value) => {
                        if assignments.contains_key(&field) {
                            return Err(Rich::custom(
                                span,
                                format!("duplicate field name: {}", field),
                            ));
                        }
                        assignments.insert(field, value);
                    }
                    DocumentBodyItem::Condition(condition) => {
                        conditions.push(condition);
                    }
                    DocumentBodyItem::Selector(selector) => {
                        selectors.push(selector);
                    }
                }
            }

            Ok(ParsedDocContent {
                assignments,
                conditions,
                selectors,
            })
        })
        .labelled("document body")
        .as_context()
    })
}

/// Parses an INSERT document query.
///
/// ## Returns
///
/// Returns a parser that matches INSERT queries and returns a [`DocumentQuery::Insert`].
fn insert_document_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Insert)
        .ignore_then(choice((
            just(Token::Doc),
            just(Token::Docs),
            just(Token::Document),
            just(Token::Documents),
        )))
        .ignore_then(just(Token::Into))
        .ignore_then(identifier_parser("collection name"))
        .then(document_body_parser())
        .try_map(|(collection_name, body), span| {
            if !body.conditions.is_empty() {
                return Err(Rich::custom(
                    span,
                    "conditions are not allowed in INSERT queries",
                ));
            }
            if !body.selectors.is_empty() {
                return Err(Rich::custom(
                    span,
                    "selectors are not allowed in INSERT queries",
                ));
            }
            Ok(DocumentQuery::Insert {
                collection_name,
                fields: body.assignments,
            })
        })
        .labelled("insert document")
        .as_context()
}

/// Parses a GET document query.
///
/// ## Returns
///
/// Returns a parser that matches GET queries and returns a [`DocumentQuery::Get`].
fn get_document_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Get)
        .ignore_then(choice((
            just(Token::Doc),
            just(Token::Docs),
            just(Token::Document),
            just(Token::Documents),
        )))
        .ignore_then(just(Token::From))
        .ignore_then(identifier_parser("collection name"))
        .then(document_body_parser())
        .try_map(|(collection_name, body), span| {
            if !body.assignments.is_empty() {
                return Err(Rich::custom(
                    span,
                    "assignments are not allowed in GET queries",
                ));
            }
            if body.conditions.is_empty() && body.selectors.is_empty() {
                return Err(Rich::custom(
                    span,
                    "GET query must have at least one condition or selector",
                ));
            }
            Ok(DocumentQuery::Get {
                collection_name,
                conditions: body.conditions,
                selectors: body.selectors,
            })
        })
        .labelled("get document")
        .as_context()
}

/// Parses an UPDATE document query.
///
/// ## Returns
///
/// Returns a parser that matches UPDATE queries and returns a [`DocumentQuery::Update`].
fn update_document_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    just(Token::Update)
        .ignore_then(choice((
            just(Token::Doc),
            just(Token::Docs),
            just(Token::Document),
            just(Token::Documents),
        )))
        .ignore_then(just(Token::In))
        .ignore_then(identifier_parser("collection name"))
        .then(document_body_parser())
        .try_map(|(collection_name, body), span| {
            if body.assignments.is_empty() {
                return Err(Rich::custom(
                    span,
                    "UPDATE query must have at least one assignment",
                ));
            }
            Ok(DocumentQuery::Update {
                collection_name,
                conditions: body.conditions,
                updates: body.assignments,
                selectors: body.selectors,
            })
        })
        .labelled("update document")
        .as_context()
}

/// Parses a DELETE document query.
///
/// ## Returns
///
/// Returns a parser that matches DELETE queries and returns a [`DocumentQuery::Delete`].
fn delete_document_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((just(Token::Delete), just(Token::Remove)))
        .ignore_then(choice((
            just(Token::Doc),
            just(Token::Docs),
            just(Token::Document),
            just(Token::Documents),
        )))
        .ignore_then(just(Token::From))
        .ignore_then(identifier_parser("collection name"))
        .then(document_body_parser())
        .try_map(|(collection_name, body), span| {
            if !body.assignments.is_empty() {
                return Err(Rich::custom(
                    span,
                    "assignments are not allowed in DELETE queries",
                ));
            }
            if body.conditions.is_empty() && body.selectors.is_empty() {
                return Err(Rich::custom(
                    span,
                    "DELETE query must have at least one condition or selector",
                ));
            }
            Ok(DocumentQuery::Delete {
                collection_name,
                conditions: body.conditions,
                selectors: body.selectors,
            })
        })
        .labelled("delete document")
        .as_context()
}

/// Creates a parser for document-level queries.
///
/// ## Returns
///
/// Returns a parser that matches document queries and returns a [`DocumentQuery`].
pub(crate) fn document_query_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, DocumentQuery, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    choice((
        insert_document_parser(),
        get_document_parser(),
        update_document_parser(),
        delete_document_parser(),
    ))
    .labelled("document query")
    .as_context()
}
