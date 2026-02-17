use std::fmt;

use bson::Document as BsonDocument;
use uuid::Uuid;

/// A unique identifier for a document in the database.
///
/// This type can represent either a string identifier (defaulting to UUIDs) or a u64 integer identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocId {
    /// A string-based identifier (UUIDs or arbitrary strings).
    String(String),
    /// A u64-based identifier.
    U64(u64),
}

impl DocId {
    /// Creates a new document ID with a randomly generated UUID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with a random UUID.
    pub fn new() -> Self {
        Self::String(Uuid::new_v4().to_string())
    }

    /// Creates a new document ID with a u64 value.
    ///
    /// ## Arguments
    ///
    /// * `value` - The u64 value to use as the ID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with the specified u64 value.
    pub fn from_u64(value: u64) -> Self {
        Self::U64(value)
    }

    /// Creates a new document ID with a UUID.
    ///
    /// ## Arguments
    ///
    /// * `uuid` - The UUID to use as the ID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with the specified UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self::String(uuid.to_string())
    }

    /// Creates a new document ID with an arbitrary string.
    ///
    /// ## Arguments
    ///
    /// * `value` - The string value to use as the ID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with the specified string value.
    pub fn from_string(value: String) -> Self {
        Self::String(value)
    }

    /// Converts the document ID to a BSON value.
    ///
    /// ## Returns
    ///
    /// A BSON value representation of the document ID.
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

/// A document in the database containing data and metadata.
///
/// Documents are the primary storage unit in the database. Each document
/// has a unique identifier and contains the actual data as a BSON document.
#[derive(Debug, Clone)]
pub struct Document {
    /// The unique identifier for this document.
    pub id: DocId,
    /// The BSON document containing the actual data.
    pub data: BsonDocument,
}

impl Document {
    /// Creates a new document with the specified ID and data.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [document ID](DocId) to use.
    /// * `data` - The [BSON document](BsonDocument) containing the document data.
    ///
    /// ## Returns
    ///
    /// A new [`Document`] with the specified ID and data.
    pub fn new(id: DocId, data: BsonDocument) -> Self {
        Self { id, data }
    }

    /// Creates a new document with a randomly generated ID and the provided data.
    ///
    /// ## Arguments
    ///
    /// * `data` - The [BSON document](BsonDocument) containing the document data.
    ///
    /// ## Returns
    ///
    /// A new [`Document`] with a random ID and the provided data.
    pub fn with_random_id(data: BsonDocument) -> Self {
        Self {
            id: DocId::new(),
            data,
        }
    }

    /// Consumes the document and returns its components.
    ///
    /// ## Returns
    ///
    /// A tuple containing the [document ID](DocId) and the [BSON document](BsonDocument) data.
    pub fn into_parts(self) -> (DocId, BsonDocument) {
        (self.id, self.data)
    }
}

impl From<BsonDocument> for Document {
    /// Creates a new document with a randomly generated ID and the provided data.
    ///
    /// ## Arguments
    ///
    /// * `data` - The [BSON document](BsonDocument) containing the document data.
    ///
    /// ## Returns
    ///
    /// A new [`Document`] with a random ID and the provided data.
    fn from(data: BsonDocument) -> Self {
        Self::with_random_id(data)
    }
}

impl From<(DocId, BsonDocument)> for Document {
    /// Creates a new document with the specified ID and data.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [document ID](DocId) to use.
    /// * `data` - The [BSON document](BsonDocument) containing the document data.
    ///
    /// ## Returns
    ///
    /// A new [`Document`] with the specified ID and data.
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
