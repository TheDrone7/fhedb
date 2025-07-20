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

    /// Converts a string to an operation.
    ///
    /// ## Arguments
    ///
    /// * `s` - The string to convert.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Operation`]) if the string is valid, or [`None`] if not recognized.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "INSERT" => Some(Operation::Insert),
            "DELETE" => Some(Operation::Delete),
            "UPDATE" => Some(Operation::Update),
            _ => None,
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
