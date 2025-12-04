use bson::Bson;
use chumsky::prelude::*;
use fhedb_core::db::schema::{FieldType, validate_bson_type};

use crate::lexer::{Span, lexer};
use crate::parser::common::bson_value_parser;

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

pub fn parse_bson_value(input: &str, expected_type: &FieldType) -> Result<Bson, String> {
    let tokens = lexer()
        .parse(input)
        .into_result()
        .map_err(|e| format!("{:?}", e))?;

    let eoi = Span::new((), input.len()..input.len());
    let token_stream = tokens.as_slice().map(eoi, |(tok, span)| (tok, span));

    let value = bson_value_parser()
        .then_ignore(end())
        .parse(token_stream)
        .into_result()
        .map_err(|e| format!("{:?}", e))?;

    validate_bson_type(&value, expected_type)?;
    Ok(value)
}
