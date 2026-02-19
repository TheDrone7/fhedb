//! # Document
//!
//! Provides the [`DocId`] and [`Document`] types for document storage.

use std::fmt;

use bson::Document as BsonDocument;
use uuid::Uuid;

/// A unique document identifier, either a string (UUID) or a [`u64`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocId {
    /// A string-based identifier (UUIDs or arbitrary strings).
    String(String),
    /// A u64-based identifier.
    U64(u64),
}

impl DocId {
    /// Creates a new [`DocId`] with a randomly generated UUID.
    pub fn new() -> Self {
        Self::String(Uuid::new_v4().to_string())
    }

    /// Creates a new [`DocId`] with a [`u64`] value.
    ///
    /// ## Arguments
    ///
    /// * `value` - The [`u64`] value to use as the ID.
    pub fn from_u64(value: u64) -> Self {
        Self::U64(value)
    }

    /// Creates a new [`DocId`] with a [`Uuid`].
    ///
    /// ## Arguments
    ///
    /// * `uuid` - The [`Uuid`] to use as the ID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self::String(uuid.to_string())
    }

    /// Creates a new [`DocId`] with an arbitrary string.
    ///
    /// ## Arguments
    ///
    /// * `value` - The string value to use as the ID.
    pub fn from_string(value: String) -> Self {
        Self::String(value)
    }

    /// Converts the document ID to a [`Bson`](bson::Bson) value.
    pub fn to_bson(&self) -> bson::Bson {
        match self {
            DocId::String(s) => bson::Bson::String(s.clone()),
            DocId::U64(value) => bson::Bson::Int64(*value as i64),
        }
    }
}

impl Default for DocId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for DocId {
    fn from(uuid: Uuid) -> Self {
        Self::String(uuid.to_string())
    }
}

impl From<u64> for DocId {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<String> for DocId {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for DocId {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<DocId> for Uuid {
    fn from(doc_id: DocId) -> Self {
        match doc_id {
            DocId::String(s) => Uuid::parse_str(&s).expect("Invalid UUID string"),
            DocId::U64(_) => panic!("Cannot convert u64 DocId to Uuid"),
        }
    }
}

impl From<DocId> for u64 {
    fn from(doc_id: DocId) -> Self {
        match doc_id {
            DocId::String(_) => panic!("Cannot convert String DocId to u64"),
            DocId::U64(value) => value,
        }
    }
}

/// A document with a unique [`DocId`] and [`BsonDocument`] data.
#[derive(Debug, Clone)]
pub struct Document {
    /// The unique identifier for this document.
    pub id: DocId,
    /// The BSON document containing the actual data.
    pub data: BsonDocument,
}

impl Document {
    /// Creates a new [`Document`] with the specified ID and data.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] to use.
    /// * `data` - The [`BsonDocument`] containing the document data.
    pub fn new(id: DocId, data: BsonDocument) -> Self {
        Self { id, data }
    }

    /// Creates a new [`Document`] with a randomly generated ID.
    ///
    /// ## Arguments
    ///
    /// * `data` - The [`BsonDocument`] containing the document data.
    pub fn with_random_id(data: BsonDocument) -> Self {
        Self {
            id: DocId::new(),
            data,
        }
    }

    /// Consumes the document and returns its ([`DocId`], [`BsonDocument`]) components.
    pub fn into_parts(self) -> (DocId, BsonDocument) {
        (self.id, self.data)
    }
}

impl From<BsonDocument> for Document {
    /// Creates a new [`Document`] with a randomly generated ID.
    fn from(data: BsonDocument) -> Self {
        Self::with_random_id(data)
    }
}

impl From<(DocId, BsonDocument)> for Document {
    /// Creates a new [`Document`] from a ([`DocId`], [`BsonDocument`]) tuple.
    fn from((id, data): (DocId, BsonDocument)) -> Self {
        Self::new(id, data)
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocId::String(s) => write!(f, "{}", s),
            DocId::U64(value) => write!(f, "{}", value),
        }
    }
}
