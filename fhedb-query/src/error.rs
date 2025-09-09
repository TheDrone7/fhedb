//! Error types for FHEDB query parsing and processing.

use thiserror::Error;

/// Represents errors that can occur during parsing of FHEDB queries.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// A syntax error in the query string.
    ///
    /// This error occurs when the input doesn't conform to the expected
    /// syntax of the FHEDB query language. The error includes a descriptive
    /// message explaining what went wrong.
    ///
    /// ## Fields
    ///
    /// * `message` - A human-readable description of the syntax error.
    #[error("Syntax error: {message}")]
    SyntaxError {
        /// A descriptive message explaining the syntax error.
        message: String,
    },
}
