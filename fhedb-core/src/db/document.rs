use bson::Document as BsonDocument;
use uuid::Uuid;

/// A unique identifier for a document in the database.
///
/// This type wraps a [`Uuid`] to provide a strongly-typed document identifier
/// that can be used throughout the database system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocId(Uuid);

impl DocId {
    /// Creates a new document ID with a randomly generated UUID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with a random UUID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Converts the document ID to a string representation.
    ///
    /// ## Returns
    ///
    /// A string representation of the document ID.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for DocId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for DocId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<DocId> for Uuid {
    fn from(doc_id: DocId) -> Self {
        doc_id.0
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
