//! # Collection
//!
//! Provides the core [`Collection`] type and its document management operations.

pub mod data;
pub mod file;

use crate::{
    document::{DocId, Document},
    schema::{IdType, Schema, SchemaOps},
};
use file::Operation;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

/// A collection of documents with a shared [`Schema`].
#[derive(Debug, Clone)]
pub struct Collection {
    /// The name of the collection.
    pub name: String,
    /// The schema describing the structure of documents in this collection.
    pub(crate) schema: Schema,
    /// The in-memory storage of document indices, mapping document IDs to log file offsets.
    pub(crate) document_indices: HashMap<DocId, usize>,
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
    /// * `schema` - The [`Schema`] describing document structure.
    /// * `base_path` - The base directory path for collection storage.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if created successfully,
    /// or [`Err`]\([`String`]) if the schema is invalid.
    pub fn new(
        name: impl Into<String>,
        mut schema: Schema,
        base_path: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        let (id_field, id_type) = schema.ensure_id()?;
        let name = name.into();
        let temp_path = base_path.into();
        let base_path = temp_path.join(&name);

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

    /// Adds a BSON document to the collection after validating it against the schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`bson::Document`] to add.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`DocId`]) of the added document,
    /// or [`Err`]\([`Vec<String>`]) with validation errors.
    pub fn add_document(&mut self, mut doc: bson::Document) -> Result<DocId, Vec<String>> {
        self.schema.apply_defaults(&mut doc);

        self.validate_document(&doc)?;
        let id_field = &self.id_field;
        let doc_id = match self.get_doc_id_from_bson(&doc) {
            Some(value) => value,
            None => {
                let new_id = self.generate_id();
                doc.insert(id_field, new_id.to_bson());
                new_id
            }
        };

        if self.document_indices.contains_key(&doc_id) {
            return Err(vec![format!(
                "Document with ID '{}' already exists.",
                doc_id.to_string()
            )]);
        }

        match self.append_to_log(&Operation::Insert, &doc) {
            Ok(offset) => {
                self.document_indices.insert(doc_id.clone(), offset);
            }
            Err(e) => {
                return Err(vec![e.to_string()]);
            }
        }

        self.inserts += 1;
        self.write_metadata()
            .map_err(|e| vec![format!("Failed to write metadata: {}", e)])?;
        Ok(doc_id)
    }

    /// Generates a new document ID based on the collection's ID type.
    pub(crate) fn generate_id(&self) -> DocId {
        match self.id_type {
            IdType::String => DocId::from_uuid(Uuid::new_v4()),
            IdType::Int => DocId::from_u64(self.inserts),
        }
    }

    /// Extracts the document ID from a BSON document.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`bson::Document`] to extract the ID from.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`DocId`]) if the ID field is present,
    /// or [`None`] if missing or of an unsupported type.
    pub(crate) fn get_doc_id_from_bson(&self, doc: &bson::Document) -> Option<DocId> {
        let id_field = &self.id_field;
        match doc.get(id_field) {
            Some(bson::Bson::String(s)) => Some(DocId::from_string(s.clone())),
            Some(bson::Bson::Int32(i)) => Some(DocId::from_u64(*i as u64)),
            Some(bson::Bson::Int64(i)) => Some(DocId::from_u64(*i as u64)),
            _ => None,
        }
    }

    /// Updates a document in the collection by its ID.
    ///
    /// Only the fields present in the update document will be modified.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to update.
    /// * `update_doc` - A [`bson::Document`] containing the fields to update.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Document`]) with the updated document,
    /// or [`Err`]\([`Vec<String>`]) with validation errors.
    pub fn update_document(
        &mut self,
        id: DocId,
        update_doc: bson::Document,
    ) -> Result<Document, Vec<String>> {
        if update_doc.contains_key(&self.id_field) {
            return Err(vec![format!("Cannot update ID field '{}'", self.id_field)]);
        }

        let offset = match self.document_indices.get(&id) {
            Some(&offset) => offset,
            None => return Err(vec![format!("Document with ID {:?} not found", id)]),
        };

        let current_log_entry = match self.read_log_entry_at_offset(offset) {
            Ok(entry) => entry,
            Err(e) => return Err(vec![format!("Failed to read document: {}", e)]),
        };

        let mut updated_doc = current_log_entry.document.clone();

        for (key, value) in update_doc {
            updated_doc.insert(key, value);
        }

        self.validate_document(&updated_doc)?;

        match self.append_to_log(&Operation::Update, &updated_doc) {
            Ok(new_offset) => {
                self.document_indices.insert(id.clone(), new_offset);
                Ok(Document::new(id, updated_doc))
            }
            Err(e) => Err(vec![format!(
                "Failed to write updated document to log: {}",
                e
            )]),
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
    /// Returns [`Some`]\([`Document`]) if removed, or [`None`] if not found.
    pub fn remove_document(&mut self, id: DocId) -> Option<Document> {
        if let Some(offset) = self.document_indices.remove(&id)
            && let Ok(log_entry) = self.read_log_entry_at_offset(offset)
        {
            self.append_to_log(&Operation::Delete, &log_entry.document)
                .ok();
            return Some(Document::new(id, log_entry.document));
        }
        None
    }

    /// Retrieves a document by its ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Document`]) if found, or [`None`] if not present.
    pub fn get_document(&self, id: DocId) -> Option<Document> {
        if let Some(&offset) = self.document_indices.get(&id)
            && let Ok(log_entry) = self.read_log_entry_at_offset(offset)
        {
            return Some(Document::new(id, log_entry.document));
        }
        None
    }

    /// Returns all documents in the collection.
    pub fn get_documents(&self) -> Vec<Document> {
        let mut entries = Vec::new();
        for (id, offset) in &self.document_indices {
            if let Ok(log_entry) = self.read_log_entry_at_offset(*offset) {
                entries.push(Document::new(id.clone(), log_entry.document));
            }
        }
        entries
    }

    /// Returns the schema of this collection.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Returns the number of inserts performed on this collection.
    pub fn inserts(&self) -> u64 {
        self.inserts
    }

    /// Returns the base path of this collection.
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Returns the name of the ID field for this collection.
    pub fn id_field_name(&self) -> &str {
        &self.id_field
    }

    /// Returns the document indices map containing [`DocId`] to log offset mappings.
    pub fn document_indices(&self) -> &HashMap<DocId, usize> {
        &self.document_indices
    }
}
