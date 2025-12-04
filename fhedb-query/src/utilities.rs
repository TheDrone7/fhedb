use bson::Bson;
use chumsky::{extra, input::ValueInput, prelude::*};
use fhedb_core::db::schema::{FieldType, validate_bson_type};

use crate::lexer::{Span, Token, lexer};

pub fn unescape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => {
                    chars.next();
                    result.push('\n');
                }
                Some('t') => {
                    chars.next();
                    result.push('\t');
                }
                Some('r') => {
                    chars.next();
                    result.push('\r');
                }
                Some('0') => {
                    chars.next();
                    result.push('\0');
                }
                Some('\\') => {
                    chars.next();
                    result.push('\\');
                }
                Some('"') => {
                    chars.next();
                    result.push('"');
                }
                Some('\'') => {
                    chars.next();
                    result.push('\'');
                }
                _ => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

pub(crate) fn bson_value_parser_internal<'tokens, I>()
-> impl Parser<'tokens, I, Bson, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    recursive(|bson_value| {
        let atom = select! {
            Token::StringLit(s) => Bson::String(s),
            Token::IntLit(n) => Bson::Int64(n),
            Token::FloatLit(s) => Bson::Double(s.parse().unwrap()),
            Token::Ident(s) if s.eq_ignore_ascii_case("true") => Bson::Boolean(true),
            Token::Ident(s) if s.eq_ignore_ascii_case("false") => Bson::Boolean(false),
            Token::Ident(s) if s.eq_ignore_ascii_case("null") => Bson::Null,
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

pub fn parse_bson_value(input: &str, expected_type: &FieldType) -> Result<Bson, String> {
    let tokens = lexer()
        .parse(input)
        .into_result()
        .map_err(|e| format!("{:?}", e))?;

    let eoi = Span::new((), input.len()..input.len());
    let token_stream = tokens.as_slice().map(eoi, |(tok, span)| (tok, span));

    let value = bson_value_parser_internal()
        .then_ignore(end())
        .parse(token_stream)
        .into_result()
        .map_err(|e| format!("{:?}", e))?;

    validate_bson_type(&value, expected_type)?;
    Ok(value)
}
