//! # Error Handling
//!
//! This module provides error types and formatting for FHEDB query parsing.

use ariadne::{Config, Label, Report, ReportKind, Source};
use chumsky::error::{RichPattern, RichReason};
use std::fmt;

use crate::lexer::{Span, Token};

/// Converts a string to title case (first letter of each word capitalized).
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Represents a parsing error with detailed context for user-friendly error messages.
///
/// This struct captures all relevant information about a parsing failure,
/// including what was expected, what was found, and the parsing context.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    /// A human-readable error message describing the failure.
    pub message: String,
    /// The span in the source where the error occurred.
    pub span: Span,
    /// The list of expected tokens or patterns at the error location.
    pub expected: Vec<String>,
    /// The actual token found at the error location, if any.
    pub found: Option<String>,
    /// The parsing context stack (innermost first).
    pub context: Vec<String>,
}

impl ParserError {
    /// Creates a new [`ParserError`] with the given details.
    ///
    /// ## Arguments
    ///
    /// * `message` - A human-readable error message.
    /// * `span` - The source span where the error occurred.
    /// * `expected` - The list of expected tokens or patterns.
    /// * `found` - The actual token found, if any.
    /// * `context` - The parsing context stack.
    ///
    /// ## Returns
    ///
    /// Returns a new [`ParserError`] instance.
    pub fn new(
        message: String,
        span: Span,
        expected: Vec<String>,
        found: Option<String>,
        context: Vec<String>,
    ) -> Self {
        Self {
            message,
            span,
            expected,
            found,
            context,
        }
    }

    /// Creates a [`ParserError`] from a chumsky Rich error (token-level).
    ///
    /// ## Arguments
    ///
    /// * `err` - The chumsky Rich error to convert.
    /// * `_source` - The original source string
    ///
    /// ## Returns
    ///
    /// Returns a new [`ParserError`] with information extracted from the Rich error.
    pub fn from_rich(err: &chumsky::error::Rich<'_, Token, Span>, _source: &str) -> Self {
        let span = *err.span();
        let found = err.found().map(|t| t.to_string());

        let expected: Vec<String> = err
            .expected()
            .map(|e| match e {
                RichPattern::Token(t) => t.to_string(),
                RichPattern::Label(l) => l.to_string(),
                RichPattern::EndOfInput => "end of input".to_string(),
                RichPattern::Identifier(s) => s.clone(),
                RichPattern::Any => "any token".to_string(),
                RichPattern::SomethingElse => "something else".to_string(),
            })
            .collect();

        let context: Vec<String> = err
            .contexts()
            .map(|(label, _)| match label {
                RichPattern::Token(t) => t.to_string(),
                RichPattern::Label(l) => l.to_string(),
                RichPattern::EndOfInput => "end of input".to_string(),
                RichPattern::Identifier(s) => s.clone(),
                RichPattern::Any => "any token".to_string(),
                RichPattern::SomethingElse => "something else".to_string(),
            })
            .collect();

        let message = match err.reason() {
            RichReason::ExpectedFound { .. } => {
                if let Some(innermost_context) = context.first() {
                    format!("Invalid {} Query", title_case(innermost_context))
                } else {
                    "Unknown Query".to_string()
                }
            }
            RichReason::Custom(msg) => msg.to_string(),
        };

        Self {
            message,
            span,
            expected,
            found,
            context,
        }
    }

    /// Creates a [`ParserError`] from a chumsky Rich error (character-level/lexer).
    ///
    /// ## Arguments
    ///
    /// * `err` - The chumsky Rich error from the lexer.
    /// * `_source` - The original source string
    ///
    /// ## Returns
    ///
    /// Returns a new [`ParserError`] with information extracted from the lexer error.
    pub fn from_lexer_rich(err: &chumsky::error::Rich<'_, char, Span>, _source: &str) -> Self {
        let span = *err.span();
        let found = err.found().map(|c| c.to_string());

        let expected: Vec<String> = err
            .expected()
            .map(|e| match e {
                RichPattern::Token(t) => format!("'{}'", t.into_inner()),
                RichPattern::Label(l) => l.to_string(),
                RichPattern::EndOfInput => "end of input".to_string(),
                RichPattern::Identifier(s) => s.clone(),
                RichPattern::Any => "any character".to_string(),
                RichPattern::SomethingElse => "something else".to_string(),
            })
            .collect();

        let context: Vec<String> = err
            .contexts()
            .map(|(label, _)| match label {
                RichPattern::Token(t) => t.into_inner().to_string(),
                RichPattern::Label(l) => l.to_string(),
                RichPattern::EndOfInput => "end of input".to_string(),
                RichPattern::Identifier(s) => s.clone(),
                RichPattern::Any => "any character".to_string(),
                RichPattern::SomethingElse => "something else".to_string(),
            })
            .collect();

        let message = match err.reason() {
            RichReason::ExpectedFound { .. } => "invalid token".to_string(),
            RichReason::Custom(msg) => msg.to_string(),
        };

        Self {
            message,
            span,
            expected,
            found,
            context,
        }
    }

    /// Formats the error as a human-readable string without colors.
    ///
    /// ## Arguments
    ///
    /// * `source` - The original source string for context display.
    ///
    /// ## Returns
    ///
    /// Returns a formatted error string with source context.
    pub fn format(&self, source: &str) -> String {
        self.format_impl(source, "<input>", false)
    }

    /// Formats the error as a human-readable string with ANSI colors.
    ///
    /// ## Arguments
    ///
    /// * `source` - The original source string for context display.
    ///
    /// ## Returns
    ///
    /// Returns a formatted error string with source context and ANSI color codes.
    pub fn format_colored(&self, source: &str) -> String {
        self.format_impl(source, "<input>", true)
    }

    /// Formats the error with a custom filename, without colors.
    ///
    /// ## Arguments
    ///
    /// * `source` - The original source string for context display.
    /// * `filename` - The filename to display in the error output.
    ///
    /// ## Returns
    ///
    /// Returns a formatted error string with the specified filename.
    pub fn format_with_filename(&self, source: &str, filename: &str) -> String {
        self.format_impl(source, filename, false)
    }

    /// Formats the error with a custom filename and ANSI colors.
    ///
    /// ## Arguments
    ///
    /// * `source` - The original source string for context display.
    /// * `filename` - The filename to display in the error output.
    ///
    /// ## Returns
    ///
    /// Returns a formatted error string with the specified filename and ANSI color codes.
    pub fn format_with_filename_colored(&self, source: &str, filename: &str) -> String {
        self.format_impl(source, filename, true)
    }

    /// Formats an expected token/pattern for display in error messages.
    ///
    /// Distinguishes between keywords (all uppercase) and identifiers (lowercase start).
    fn format_expected(e: &str) -> String {
        if e == "end of input" {
            "end of input".to_string()
        } else if e.chars().all(|c| c.is_uppercase() || c == '_') {
            format!("keyword '{}'", e)
        } else if e.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
            format!("identifier <{}>", e)
        } else {
            e.to_string()
        }
    }

    /// Internal implementation for formatting errors using ariadne.
    fn format_impl(&self, source: &str, filename: &str, colored: bool) -> String {
        let is_unknown_query = self.message == "Unknown Query";

        let label_msg = if is_unknown_query {
            "No matching query found".to_string()
        } else {
            let expected_str = if self.expected.is_empty() {
                "something".to_string()
            } else if self.expected.len() == 1 {
                Self::format_expected(&self.expected[0])
            } else {
                let formatted: Vec<String> = self
                    .expected
                    .iter()
                    .map(|e| Self::format_expected(e))
                    .collect();
                let last = formatted.last().unwrap();
                let rest = &formatted[..formatted.len() - 1];
                format!("{}, or {}", rest.join(", "), last)
            };
            format!("Expected {}", expected_str)
        };

        let mut report_builder =
            Report::build(ReportKind::Error, (filename, self.span.into_range()))
                .with_config(Config::default().with_color(colored))
                .with_message(&self.message)
                .with_label(Label::new((filename, self.span.into_range())).with_message(label_msg));

        if !self.context.is_empty() {
            let context_path = self
                .context
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join(" -> ");
            report_builder = report_builder.with_help(format!("While parsing: {}", context_path));
        }

        let report = report_builder.finish();

        let mut output = Vec::new();
        report
            .write((filename, Source::from(source)), &mut output)
            .unwrap();

        String::from_utf8(output).unwrap()
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let expected_str = if self.expected.is_empty() {
            "something".to_string()
        } else if self.expected.len() == 1 {
            self.expected[0].clone()
        } else {
            let last = self.expected.last().unwrap();
            let rest = &self.expected[..self.expected.len() - 1];
            format!("{}, or {}", rest.join(", "), last)
        };

        let found_str = self.found.as_deref().unwrap_or("end of input");

        write!(
            f,
            "expected {}, found {} at {}..{}",
            expected_str, found_str, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for ParserError {}
