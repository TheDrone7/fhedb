//! Common utilities for FHEDB query parsers.

use chumsky::prelude::*;

use crate::error::ParserError;
use crate::lexer::{Span, Token, lexer};

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
