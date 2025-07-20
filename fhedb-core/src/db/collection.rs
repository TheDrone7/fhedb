use crate::db::document::{DocId, Document};
use crate::db::schema::{IdType, Schema};
use crate::file::{collection::CollectionFileOps, types::Operation};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Describes a collection of documents in the database.
///
/// Each collection has a unique name and an associated [`Schema`].
#[derive(Debug, Clone)]
pub struct Collection {
    /// The name of the collection.
    pub name: String,
    /// The schema describing the structure of documents in this collection.
    pub(crate) schema: Schema,
    /// The in-memory storage of document indices, mapping document IDs to log file offsets.
    document_indices: HashMap<DocId, usize>,
    /// The name of the field in the schema with type Id, or "id" if not present in the schema.
    pub(crate) id_field: String,
    /// The type of ID used in this collection (string or integer).
    pub(crate) id_type: IdType,
    /// Counter for generating sequential u64 IDs. Starts at 0 and increments on each insert.
    pub(crate) inserts: u64,
    /// The base path for the collection.
    pub(crate) base_path: PathBuf,
}

impl Collection {
    /// Creates a new [`Collection`] with the given name and schema.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the collection.
    /// * `schema` - The [`Schema`] describing the structure of documents in this collection.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if collection was created successfully, or [`Err`]\([`String`]) otherwise.
    pub fn new(
        name: impl Into<String>,
        mut schema: Schema,
        base_path: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        let (id_field, id_type) = schema.ensure_id()?;
        let name = name.into();
        let temp_path = base_path.into();
        let base_path = temp_path.join(name.clone());

        Ok(Self {
            name,
            schema,
            document_indices: HashMap::new(),
            id_field,
            id_type,
            inserts: 0,
            base_path,
        })
    }

    /// Creates a new [`Collection`] from existing files on disk.
    ///
    /// This method reads the collection's metadata from the filesystem and reconstructs
    /// the collection instance. It reads all log entries and processes them to rebuild
    /// the collection's state.
    ///
    /// ## Arguments
    ///
    /// * `base_path` - The base directory path where collections are stored.
    /// * `name` - The name of the collection to load.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if the collection was loaded successfully,
    /// or [`Err`]\([`std::io::Error`]) if the collection could not be loaded.
    pub fn from_files(base_path: impl Into<PathBuf>, name: &str) -> Result<Self, std::io::Error> {
        let mut collection = Self::read_metadata(base_path, name)?;

        // Read all log entries from the logfile with their offsets
        let log_entries = collection.read_log_entries()?;

        // Loop over all log entries and rebuild the document indices
        for (log_entry, log_offset) in log_entries {
            // Extract the document ID from the BSON document
            let doc_id = collection
                .get_doc_id_from_bson(&log_entry.document)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Could not extract document ID from log entry at offset {}",
                            log_offset
                        ),
                    )
                })?;

            // Update collection indices based on operation type
            match log_entry.operation {
                Operation::Insert | Operation::Update => {
                    collection.document_indices.insert(doc_id, log_offset);
                }
                Operation::Delete => {
                    collection.document_indices.remove(&doc_id);
                }
            }
        }

        Ok(collection)
    }

    /// Checks if the schema contains a field with the given name.
    ///
    /// ## Arguments
    ///
    /// * `field` - The name of the field to check.
    ///
    /// ## Returns
    ///
    /// `true` if the field exists in the schema, `false` otherwise.
    pub fn has_field(&self, field: &str) -> bool {
        self.schema.fields.contains_key(field)
    }

    /// Validates a BSON document against this collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - A reference to the [`bson::Document`] to validate.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the document matches the schema. Returns [`Err(Vec<String>)`](Result::Err) containing error messages for each field that does not conform to the schema.
    pub fn validate_document(&self, doc: &bson::Document) -> Result<(), Vec<String>> {
        self.schema.validate_document(doc)
    }

    /// Adds a BSON document to the collection after validating it against the schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`bson::Document`] to add.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`DocId`]) if the document is valid and added, or [`Err`]\([`Vec<String>`]) with validation errors. Returns an error if the schema does not have an ID field.
    pub fn add_document(&mut self, mut doc: bson::Document) -> Result<DocId, Vec<String>> {
        if let Err(errors) = self.validate_document(&doc) {
            return Err(errors);
        }
        // Use the id_field (from schema or default)
        let id_field = &self.id_field;
        let doc_id = match self.get_doc_id_from_bson(&doc) {
            Some(value) => value,
            None => {
                // No ID provided, generate new one
                let new_id = self.generate_id();
                doc.insert(id_field, new_id.to_bson());
                new_id
            }
        };
        let db_doc = Document::new(doc_id.clone(), doc);
        // Store the document using the logfile
        match self.append_to_log(&Operation::Insert, &db_doc.data) {
            Ok(offset) => {
                self.document_indices.insert(doc_id.clone(), offset);
            }
            Err(e) => {
                return Err(vec![e.to_string()]);
            }
        }

        self.inserts += 1;
        Ok(doc_id)
    }

    /// Generates a new document ID based on the collection's ID type.
    ///
    /// For u64 IDs, uses the current inserts counter value.
    /// For String IDs, generates a random UUID.
    ///
    /// ## Returns
    ///
    /// A new [`DocId`] with the appropriate type and value.
    fn generate_id(&self) -> DocId {
        match self.id_type {
            IdType::String => DocId::from_uuid(Uuid::new_v4()),
            IdType::Int => DocId::from_u64(self.inserts),
        }
    }

    /// Extracts the document ID from a BSON document.
    ///
    /// This method retrieves the ID field from the document and converts it to a [`DocId`].
    /// If the ID field is not present or is of an unsupported type, it returns `None`.
    ///
    /// ## Arguments
    ///
    /// * `doc` - A reference to the [`bson::Document`] from which to extract the ID.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`DocId`]) if the ID was successfully extracted,
    /// or [`None`] if the ID field is missing or of an unsupported type.
    fn get_doc_id_from_bson(&self, doc: &bson::Document) -> Option<DocId> {
        let id_field = &self.id_field;
        match doc.get(id_field) {
            Some(bson::Bson::String(s)) => Some(DocId::from_string(s.clone())),
            Some(bson::Bson::Int32(i)) => Some(DocId::from_u64(*i as u64)),
            Some(bson::Bson::Int64(i)) => Some(DocId::from_u64(*i as u64)),
            _ => None,
        }
    }

    /// Removes a document from the collection by its ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to remove.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Document`]) if the document was present and removed, or [`None`] if not found.
    pub fn remove_document(&mut self, id: DocId) -> Option<Document> {
        if let Some(offset) = self.document_indices.remove(&id) {
            if let Ok(log_entry) = self.read_log_entry_at_offset(offset) {
                self.append_to_log(&Operation::Delete, &log_entry.document)
                    .ok();
                return Some(Document::new(id.clone(), log_entry.document));
            }
        }
        None
    }

    /// Retrieves a reference to a document by its ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Document`]) if found, or [`None`] if not present.
    pub fn get_document(&self, id: DocId) -> Option<Document> {
        if let Some(&offset) = self.document_indices.get(&id) {
            if let Ok(log_entry) = self.read_log_entry_at_offset(offset) {
                return Some(Document::new(id.clone(), log_entry.document));
            }
        }
        None
    }

    /// Retrieves all documents in the collection.
    ///
    /// ## Returns
    ///
    /// Returns a [`Vec`] containing references to all [`Document`]s in the collection.
    pub fn get_documents(&self) -> Vec<Document> {
        let mut entries = Vec::new();
        for (id, offset) in &self.document_indices {
            if let Ok(log_entry) = self.read_log_entry_at_offset(*offset) {
                entries.push(Document::new(id.clone(), log_entry.document));
            }
        }
        entries
    }

    /// Gets the schema of this collection.
    ///
    /// ## Returns
    ///
    /// A reference to the collection's [`Schema`].
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Gets the number of inserts performed on this collection.
    ///
    /// ## Returns
    ///
    /// The number of inserts as a `u64`.
    pub fn inserts(&self) -> u64 {
        self.inserts
    }

    /// Gets the base path of this collection.
    ///
    /// ## Returns
    ///
    /// The base path as a [`PathBuf`].
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Gets the name of the ID field for this collection.
    ///
    /// ## Returns
    ///
    /// The name of the ID field as a [`String`].
    pub fn id_field_name(&self) -> &str {
        &self.id_field
    }

    /// Gets the document indices map containing DocId to log offset mappings.
    ///
    /// ## Returns
    ///
    /// A reference to the document indices [`HashMap`].
    pub fn document_indices(&self) -> &HashMap<DocId, usize> {
        &self.document_indices
    }
}
