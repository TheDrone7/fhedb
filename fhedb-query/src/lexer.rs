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
    /// The DOC keyword.
    Doc,
    /// The DOCUMENT keyword.
    Document,
    /// The DOCS keyword.
    Docs,
    /// The DOCUMENTS keyword.
    Documents,
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
    /// The MODIFY keyword.
    Modify,
    /// The ALTER keyword.
    Alter,
    /// The INSERT keyword.
    Insert,
    /// The UPDATE keyword.
    Update,
    /// The DELETE keyword.
    Delete,
    /// The REMOVE keyword (alias for DELETE).
    Remove,
    /// The INTO keyword.
    Into,
    /// The IN keyword.
    In,
    /// The INT field type keyword.
    TypeInt,
    /// The FLOAT field type keyword.
    TypeFloat,
    /// The STRING field type keyword.
    TypeString,
    /// The BOOLEAN field type keyword.
    TypeBoolean,
    /// The ARRAY field type keyword.
    TypeArray,
    /// The REF field type keyword.
    TypeRef,
    /// The ID_INT field type keyword.
    TypeIdInt,
    /// The ID_STRING field type keyword.
    TypeIdString,
    /// The NULLABLE constraint keyword.
    Nullable,
    /// The DEFAULT constraint keyword.
    Default,
    /// The TRUE boolean literal.
    True,
    /// The FALSE boolean literal.
    False,
    /// The NULL literal.
    Null,
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
    /// A double equals sign (==) for similarity comparison.
    DoubleEquals,
    /// A not equals sign (!=).
    NotEquals,
    /// An open angle bracket.
    OpenAngle,
    /// A close angle bracket.
    CloseAngle,
    /// Less than or equal (<=).
    LessThanOrEqual,
    /// Greater than or equal (>=).
    GreaterThanOrEqual,
    /// A single asterisk (*) for field selection.
    Star,
    /// A double asterisk (**) for recursive field selection.
    DoubleStar,
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
            Token::Doc => write!(f, "DOC"),
            Token::Document => write!(f, "DOCUMENT"),
            Token::Docs => write!(f, "DOCS"),
            Token::Documents => write!(f, "DOCUMENTS"),
            Token::If => write!(f, "IF"),
            Token::Exists => write!(f, "EXISTS"),
            Token::Schema => write!(f, "SCHEMA"),
            Token::From => write!(f, "FROM"),
            Token::Get => write!(f, "GET"),
            Token::Modify => write!(f, "MODIFY"),
            Token::Alter => write!(f, "ALTER"),
            Token::Insert => write!(f, "INSERT"),
            Token::Update => write!(f, "UPDATE"),
            Token::Delete => write!(f, "DELETE"),
            Token::Remove => write!(f, "REMOVE"),
            Token::Into => write!(f, "INTO"),
            Token::In => write!(f, "IN"),
            Token::TypeInt => write!(f, "INT"),
            Token::TypeFloat => write!(f, "FLOAT"),
            Token::TypeString => write!(f, "STRING"),
            Token::TypeBoolean => write!(f, "BOOLEAN"),
            Token::TypeArray => write!(f, "ARRAY"),
            Token::TypeRef => write!(f, "REF"),
            Token::TypeIdInt => write!(f, "ID_INT"),
            Token::TypeIdString => write!(f, "ID_STRING"),
            Token::Nullable => write!(f, "NULLABLE"),
            Token::Default => write!(f, "DEFAULT"),
            Token::True => write!(f, "TRUE"),
            Token::False => write!(f, "FALSE"),
            Token::Null => write!(f, "NULL"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Equals => write!(f, "="),
            Token::DoubleEquals => write!(f, "=="),
            Token::NotEquals => write!(f, "!="),
            Token::OpenAngle => write!(f, "<"),
            Token::CloseAngle => write!(f, ">"),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::Star => write!(f, "*"),
            Token::DoubleStar => write!(f, "**"),
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
    let query_kw = choice((
        keyword_ci("create").to(Token::Create),
        keyword_ci("drop").to(Token::Drop),
        keyword_ci("list").to(Token::List),
        keyword_ci("databases").to(Token::Databases),
        keyword_ci("database").to(Token::Database),
        keyword_ci("collections").to(Token::Collections),
        keyword_ci("collection").to(Token::Collection),
        keyword_ci("doc").to(Token::Doc),
        keyword_ci("document").to(Token::Document),
        keyword_ci("docs").to(Token::Docs),
        keyword_ci("documents").to(Token::Documents),
        keyword_ci("if").to(Token::If),
        keyword_ci("exists").to(Token::Exists),
        keyword_ci("schema").to(Token::Schema),
        keyword_ci("from").to(Token::From),
        keyword_ci("get").to(Token::Get),
        keyword_ci("modify").to(Token::Modify),
        keyword_ci("alter").to(Token::Alter),
        keyword_ci("insert").to(Token::Insert),
        keyword_ci("update").to(Token::Update),
        keyword_ci("delete").to(Token::Delete),
        keyword_ci("remove").to(Token::Remove),
        keyword_ci("into").to(Token::Into),
        keyword_ci("in").to(Token::In),
    ));

    let type_kw = choice((
        keyword_ci("id_string").to(Token::TypeIdString),
        keyword_ci("id_int").to(Token::TypeIdInt),
        keyword_ci("boolean").to(Token::TypeBoolean),
        keyword_ci("string").to(Token::TypeString),
        keyword_ci("float").to(Token::TypeFloat),
        keyword_ci("array").to(Token::TypeArray),
        keyword_ci("int").to(Token::TypeInt),
        keyword_ci("ref").to(Token::TypeRef),
    ));

    let constraint_kw = choice((
        keyword_ci("nullable").to(Token::Nullable),
        keyword_ci("default").to(Token::Default),
    ));

    let literal_kw = choice((
        keyword_ci("true").to(Token::True),
        keyword_ci("false").to(Token::False),
        keyword_ci("null").to(Token::Null),
    ));

    let kw = choice((query_kw, type_kw, constraint_kw, literal_kw));

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
        just("**").to(Token::DoubleStar),
        just('*').to(Token::Star),
        just("==").to(Token::DoubleEquals),
        just("!=").to(Token::NotEquals),
        just("<=").to(Token::LessThanOrEqual),
        just(">=").to(Token::GreaterThanOrEqual),
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
