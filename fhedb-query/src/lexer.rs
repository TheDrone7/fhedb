//! # Lexer
//!
//! This module provides the lexical analysis (tokenization) functionality for FHEDB queries.

use chumsky::{extra, prelude::*};

/// Represents a token in the FHEDB query language.
///
/// Tokens are the smallest meaningful units produced by the lexer.
/// Keywords are case-insensitive during lexing but stored as distinct token variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    /// The CREATE keyword.
    Create,
    /// The DROP keyword.
    Drop,
    /// The LIST keyword.
    List,
    /// The DATABASE keyword.
    Database,
    /// The DATABASES keyword.
    Databases,
    /// The IF keyword.
    If,
    /// The EXISTS keyword.
    Exists,
    /// An identifier (database name, collection name, etc.).
    Ident(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Create => write!(f, "CREATE"),
            Token::Drop => write!(f, "DROP"),
            Token::List => write!(f, "LIST"),
            Token::Database => write!(f, "DATABASE"),
            Token::Databases => write!(f, "DATABASES"),
            Token::If => write!(f, "IF"),
            Token::Exists => write!(f, "EXISTS"),
            Token::Ident(s) => write!(f, "{}", s),
        }
    }
}

/// A span representing a range of positions in the source input.
pub type Span = SimpleSpan<usize>;

/// A value paired with its span in the source input.
pub type Spanned<T> = (T, Span);

/// Creates a parser that matches a keyword case-insensitively.
///
/// ## Arguments
///
/// * `kw` - The keyword to match (case-insensitive).
///
/// ## Returns
///
/// Returns a parser that matches the keyword regardless of case.
fn keyword_ci<'src>(
    kw: &'static str,
) -> impl Parser<'src, &'src str, &'src str, extra::Err<Rich<'src, char, Span>>> + Clone {
    text::ident().filter(move |s: &&str| s.eq_ignore_ascii_case(kw))
}

/// Creates the main lexer for tokenizing FHEDB query strings.
///
/// The lexer recognizes keywords (case-insensitive) and identifiers,
/// producing a list of tokens with their source spans.
///
/// ## Returns
///
/// Returns a parser that transforms a string input into a vector of spanned tokens.
pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token>>, extra::Err<Rich<'src, char, Span>>> {
    let kw = choice((
        keyword_ci("create").to(Token::Create),
        keyword_ci("drop").to(Token::Drop),
        keyword_ci("list").to(Token::List),
        keyword_ci("databases").to(Token::Databases),
        keyword_ci("database").to(Token::Database),
        keyword_ci("if").to(Token::If),
        keyword_ci("exists").to(Token::Exists),
    ));

    let ident = text::ident()
        .map(|s: &str| Token::Ident(s.to_string()))
        .labelled("identifier");

    let token = kw.or(ident);

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect()
        .then_ignore(end())
}
