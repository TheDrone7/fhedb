//! Error types for FHEDB query parsing and processing.

use nom_locate::LocatedSpan;
use std::fmt;

/// Type alias for input spans that track position information.
pub type Span<'a> = LocatedSpan<&'a str>;

/// Represents errors that can occur during parsing of FHEDB queries.
///
/// This error type provides detailed information about where parsing failed,
/// including line/column numbers, context path, and helpful suggestions.
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    /// Syntax error occurs when the input doesn't conform to the expected
    /// syntax of the FHEDB query language.
    SyntaxError {
        /// A descriptive message explaining the syntax error.
        message: String,
        /// Line number where the error occurred (1-based).
        line: u32,
        /// Column number where the error occurred (1-based).
        column: usize,
        /// Context path showing the parsing hierarchy (e.g., ["query", "create_collection", "schema"]).
        context_path: Vec<String>,
        /// The source line containing the error.
        source_line: String,
        /// Visual pointer indicating the error position (e.g., "    ^~~").
        pointer: String,
        /// Optional suggestion for fixing the error.
        suggestion: Option<String>,
    },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::SyntaxError {
                message,
                line,
                column,
                context_path,
                source_line,
                pointer,
                suggestion,
            } => {
                writeln!(
                    f,
                    "Syntax error at line {}, column {}: {}",
                    line, column, message
                )?;

                if !context_path.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "Context: {}", context_path.join(" → "))?;
                }

                writeln!(f)?;
                writeln!(f, "{:5} | {}", line, source_line)?;
                write!(f, "        {}", pointer)?;

                if let Some(suggestion) = suggestion {
                    writeln!(f)?;
                    writeln!(f)?;
                    write!(f, "{}", suggestion)?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for ParserError {}

/// Creates a span from a string input.
///
/// ## Arguments
///
/// * `input` - The input string to wrap in a span.
///
/// ## Returns
///
/// Returns a [`Span`] that tracks position information for the input.
pub fn create_span(input: &str) -> Span<'_> {
    LocatedSpan::new(input)
}

/// Extracts position information from a span and original input.
///
/// ## Arguments
///
/// * `span` - The [`Span`] to extract position information from.
/// * `original_input` - The original input string to extract the line from.
///
/// ## Returns
///
/// Returns a tuple of (line, column, line_text) where line and column are 1-based.
pub fn extract_position_info_with_context(
    span: Span,
    original_input: &str,
) -> (u32, usize, String) {
    let line = span.location_line();
    let column = span.get_utf8_column();
    let line_text = original_input
        .lines()
        .nth((line - 1) as usize)
        .unwrap_or("")
        .to_string();
    (line, column, line_text)
}

/// Extracts position information from a span.
///
/// ## Arguments
///
/// * `span` - The [`Span`] to extract position information from.
///
/// ## Returns
///
/// Returns a tuple of (line, column, line_text) where line and column are 1-based.
pub fn extract_position_info(span: Span) -> (u32, usize, String) {
    let line = span.location_line();
    let column = span.get_utf8_column();
    let line_text = span
        .fragment()
        .lines()
        .nth((line - 1) as usize)
        .unwrap_or(span.fragment())
        .to_string();
    (line, column, line_text)
}

/// Converts a nom error to our ParserError.
///
/// ## Arguments
///
/// * `original_input` - The original input string being parsed.
/// * `context_stack` - The parsing context path (e.g., ["query", "create_database"]).
/// * `error` - The nom error to convert.
///
/// ## Returns
///
/// Returns a [`ParserError`] with full position tracking and context information.
pub fn convert_error<'a>(
    original_input: &str,
    context_stack: Vec<String>,
    error: nom::Err<nom::error::Error<Span<'a>>>,
) -> ParserError {
    match error {
        nom::Err::Error(e) | nom::Err::Failure(e) => {
            let span = e.input;
            let line = span.location_line();
            let mut column = span.get_utf8_column();
            let source_line = original_input
                .lines()
                .nth((line - 1) as usize)
                .unwrap_or("")
                .to_string();

            let fragment = span.fragment().trim();
            if fragment.is_empty() {
                column = source_line.len() + 1;
            }

            let pointer = build_pointer(column);
            let message = format_nom_error(span, e.code);
            let suggestion = generate_suggestion(&context_stack, &message);

            ParserError::SyntaxError {
                message,
                line,
                column,
                context_path: context_stack,
                source_line,
                pointer,
                suggestion,
            }
        }
        nom::Err::Incomplete(_) => ParserError::SyntaxError {
            message: "Incomplete input".to_string(),
            line: 1,
            column: 1,
            context_path: context_stack,
            source_line: original_input.lines().next().unwrap_or("").to_string(),
            pointer: "^".to_string(),
            suggestion: None,
        },
    }
}

/// Formats a nom error into a human-readable message.
///
/// ## Arguments
///
/// * `span` - The [`Span`] where the error occurred.
/// * `kind` - The nom error kind.
///
/// ## Returns
///
/// Returns a human-readable error message string.
fn format_nom_error(span: Span, kind: nom::error::ErrorKind) -> String {
    let fragment = span.fragment().trim();

    if fragment.is_empty() {
        return match kind {
            nom::error::ErrorKind::Tag => {
                "Expected keyword or identifier (found end of input)".to_string()
            }
            nom::error::ErrorKind::TakeWhile1 => {
                "Expected identifier (found end of input)".to_string()
            }
            _ => "Unexpected end of input".to_string(),
        };
    }

    let preview = if fragment.len() > 20 {
        format!("{}...", &fragment[..20])
    } else {
        fragment.to_string()
    };

    match kind {
        nom::error::ErrorKind::Tag => {
            format!("Expected keyword or token, found '{}'", preview)
        }
        nom::error::ErrorKind::Eof => "Unexpected end of input".to_string(),
        nom::error::ErrorKind::Char => {
            format!("Expected specific character, found '{}'", preview)
        }
        nom::error::ErrorKind::Alpha => {
            format!("Expected alphabetic character, found '{}'", preview)
        }
        nom::error::ErrorKind::Digit => {
            format!("Expected digit, found '{}'", preview)
        }
        nom::error::ErrorKind::AlphaNumeric => {
            format!("Expected alphanumeric character, found '{}'", preview)
        }
        nom::error::ErrorKind::TakeWhile1 => {
            format!("Expected identifier or value, found '{}'", preview)
        }
        _ => format!("Parse error at '{}'", preview),
    }
}

/// Builds a visual pointer string for the error location.
///
/// ## Arguments
///
/// * `column` - The column number (1-based) where the error occurred.
///
/// ## Returns
///
/// Returns a string with spaces and a caret (e.g., "    ^") pointing to the error position.
fn build_pointer(column: usize) -> String {
    let spaces = " ".repeat(column.saturating_sub(1));
    format!("{}^", spaces)
}

/// Generates helpful suggestions based on context and error message.
///
/// ## Arguments
///
/// * `context_path` - The parsing context path.
/// * `message` - The error message.
///
/// ## Returns
///
/// Returns an optional suggestion string to help the user fix the error.
fn generate_suggestion(context_path: &[String], message: &str) -> Option<String> {
    let last_context = context_path.last()?.as_str();

    match last_context {
        "field_type" => Some(
            "Valid field types: id_int, id_string, int, float, string, boolean, array, nullable, reference"
                .to_string(),
        ),
        "database_name" | "collection_name" | "field_name" => {
            Some("Names must be alphanumeric with underscores".to_string())
        }
        "bson_value" if message.contains("String") || message.contains("quoted") => {
            Some("String values must be quoted with single or double quotes".to_string())
        }
        _ => None,
    }
}
