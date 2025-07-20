use crate::db::document::{DocId, Document};
use crate::db::schema::Schema;
use std::collections::HashMap;

/// Describes a collection of documents in the database.
///
/// Each collection has a unique name and an associated [`Schema`].
#[derive(Debug, Clone)]
pub struct Collection {
    /// The name of the collection.
    pub name: String,
    /// The schema describing the structure of documents in this collection.
    schema: Schema,
    /// The in-memory storage of documents in this collection, keyed by document ID.
    documents: HashMap<DocId, Document>,
    /// The name of the field in the schema with type Id, or "id" if not present in the schema.
    id_field: String,
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
    pub fn new(name: impl Into<String>, mut schema: Schema) -> Result<Self, String> {
        let id_field = schema.ensure_id()?;
        Ok(Self {
            name: name.into(),
            schema,
            documents: HashMap::new(),
            id_field,
        })
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
    /// Returns [`Ok(DocId)`] if the document is valid and added, or [`Err(Vec<String>)`] with validation errors. Returns an error if the schema does not have an ID field.
    pub fn add_document(&mut self, mut doc: bson::Document) -> Result<DocId, Vec<String>> {
        if let Err(errors) = self.validate_document(&doc) {
            return Err(errors);
        }
        // Use the id_field (from schema or default)
        let id_field = &self.id_field;
        let doc_id = match doc.get_str(id_field) {
            Ok(id_str) => match uuid::Uuid::parse_str(id_str) {
                Ok(uuid) => DocId::from(uuid),
                Err(_) => {
                    let new_id = DocId::new();
                    doc.insert(id_field, new_id.to_string());
                    new_id
                }
            },
            Err(_) => {
                let new_id = DocId::new();
                doc.insert(id_field, new_id.to_string());
                new_id
            }
        };
        let db_doc = Document::new(doc_id, doc);
        self.documents.insert(doc_id, db_doc);
        Ok(doc_id)
    }

    /// Removes a document from the collection by its ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to remove.
    ///
    /// ## Returns
    ///
    /// Returns [`Some(Document)`] if the document was present and removed, or [`None`] if not found.
    pub fn remove_document(&mut self, id: DocId) -> Option<Document> {
        self.documents.remove(&id)
    }

    /// Retrieves a reference to a document by its ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The [`DocId`] of the document to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns [`Some(&Document)`] if found, or [`None`] if not present.
    pub fn get_document(&self, id: DocId) -> Option<&Document> {
        self.documents.get(&id)
    }

    /// Retrieves all documents in the collection.
    ///
    /// ## Returns
    ///
    /// Returns a [`Vec`] containing references to all [`Document`]s in the collection.
    pub fn get_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }
}
