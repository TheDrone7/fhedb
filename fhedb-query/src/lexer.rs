//! # Lexer
//!
//! This module provides the lexical analysis (tokenization) functionality for FHEDB queries.

use chumsky::{extra, prelude::*};

use crate::utilities::unescape;

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
    /// The COLLECTION keyword.
    Collection,
    /// The COLLECTIONS keyword.
    Collections,
    /// The IF keyword.
    If,
    /// The EXISTS keyword.
    Exists,
    /// The SCHEMA keyword.
    Schema,
    /// The FROM keyword.
    From,
    /// The GET keyword.
    Get,
    /// An identifier (database name, collection name, etc.).
    Ident(String),
    /// An open brace.
    OpenBrace,
    /// A close brace.
    CloseBrace,
    /// A colon.
    Colon,
    /// A comma.
    Comma,
    /// An open parenthesis.
    OpenParen,
    /// A close parenthesis.
    CloseParen,
    /// An equals sign.
    Equals,
    /// An open angle bracket.
    OpenAngle,
    /// A close angle bracket.
    CloseAngle,
    /// An open bracket.
    OpenBracket,
    /// A close bracket.
    CloseBracket,
    /// A string literal.
    StringLit(String),
    /// An integer literal.
    IntLit(i64),
    /// A float literal (stored as string for Eq/Hash).
    FloatLit(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Create => write!(f, "CREATE"),
            Token::Drop => write!(f, "DROP"),
            Token::List => write!(f, "LIST"),
            Token::Database => write!(f, "DATABASE"),
            Token::Databases => write!(f, "DATABASES"),
            Token::Collection => write!(f, "COLLECTION"),
            Token::Collections => write!(f, "COLLECTIONS"),
            Token::If => write!(f, "IF"),
            Token::Exists => write!(f, "EXISTS"),
            Token::Schema => write!(f, "SCHEMA"),
            Token::From => write!(f, "FROM"),
            Token::Get => write!(f, "GET"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Equals => write!(f, "="),
            Token::OpenAngle => write!(f, "<"),
            Token::CloseAngle => write!(f, ">"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::IntLit(n) => write!(f, "{}", n),
            Token::FloatLit(n) => write!(f, "{}", n),
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
        keyword_ci("collections").to(Token::Collections),
        keyword_ci("collection").to(Token::Collection),
        keyword_ci("if").to(Token::If),
        keyword_ci("exists").to(Token::Exists),
        keyword_ci("schema").to(Token::Schema),
        keyword_ci("from").to(Token::From),
        keyword_ci("get").to(Token::Get),
    ));

    let ident = text::ident()
        .map(|s: &str| Token::Ident(s.to_string()))
        .labelled("identifier");

    let escape_seq = just('\\').then(any()).map(|(slash, c): (char, char)| {
        let mut s = String::with_capacity(2);
        s.push(slash);
        s.push(c);
        s
    });
    let string_char = none_of("\"\\").map(|c: char| c.to_string()).or(escape_seq);

    let string_lit = just('"')
        .ignore_then(string_char.repeated().collect::<Vec<_>>())
        .then_ignore(just('"'))
        .map(|parts| Token::StringLit(unescape(&parts.join(""))))
        .labelled("string");

    let single_escape_seq = just('\\').then(any()).map(|(slash, c): (char, char)| {
        let mut s = String::with_capacity(2);
        s.push(slash);
        s.push(c);
        s
    });
    let single_string_char = none_of("'\\")
        .map(|c: char| c.to_string())
        .or(single_escape_seq);

    let single_string_lit = just('\'')
        .ignore_then(single_string_char.repeated().collect::<Vec<_>>())
        .then_ignore(just('\''))
        .map(|parts| Token::StringLit(unescape(&parts.join(""))))
        .labelled("string");

    let float_lit = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.'))
        .then(text::digits(10))
        .to_slice()
        .map(|s: &str| Token::FloatLit(s.to_string()))
        .labelled("float");

    let int_lit = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(|s: &str| Token::IntLit(s.parse().unwrap()))
        .labelled("integer");

    let symbol = choice((
        just('{').to(Token::OpenBrace),
        just('}').to(Token::CloseBrace),
        just(':').to(Token::Colon),
        just(',').to(Token::Comma),
        just('(').to(Token::OpenParen),
        just(')').to(Token::CloseParen),
        just('=').to(Token::Equals),
        just('<').to(Token::OpenAngle),
        just('>').to(Token::CloseAngle),
        just('[').to(Token::OpenBracket),
        just(']').to(Token::CloseBracket),
    ));

    let token = choice((
        kw,
        float_lit,
        int_lit,
        string_lit,
        single_string_lit,
        symbol,
        ident,
    ));

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect()
        .then_ignore(end())
}
