use std::fmt;
use std::str::FromStr;

use bson::Document as BsonDocument;

/// Represents a database operation type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Insert a new document.
    Insert,
    /// Delete an existing document.
    Delete,
    /// Update an existing document.
    Update,
}

impl Operation {
    /// Converts the operation to its string representation.
    ///
    /// ## Returns
    ///
    /// A string representation of the operation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::Insert => "INSERT",
            Operation::Delete => "DELETE",
            Operation::Update => "UPDATE",
        }
    }
}

/// Error returned when parsing an invalid operation string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOperationError(String);

impl fmt::Display for ParseOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized operation: {:?}", self.0)
    }
}

impl FromStr for Operation {
    type Err = ParseOperationError;

    /// Converts a string to an operation.
    ///
    /// ## Arguments
    ///
    /// * `s` - The string to convert.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Operation`]) if the string is valid, or [`Err`]\([`ParseOperationError`]) if not recognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INSERT" => Ok(Operation::Insert),
            "DELETE" => Ok(Operation::Delete),
            "UPDATE" => Ok(Operation::Update),
            _ => Err(ParseOperationError(s.to_string())),
        }
    }
}

/// A log entry representing a database operation.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The timestamp when the operation occurred.
    pub timestamp: String,
    /// The type of operation.
    pub operation: Operation,
    /// The BSON document associated with the operation.
    pub document: BsonDocument,
}

impl LogEntry {
    /// Creates a new log entry.
    ///
    /// ## Arguments
    ///
    /// * `operation` - The type of operation.
    /// * `document` - The BSON document associated with the operation.
    ///
    /// ## Returns
    ///
    /// A new [`LogEntry`] with the current timestamp.
    pub fn new(operation: Operation, document: BsonDocument) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation,
            document,
        }
    }
}
