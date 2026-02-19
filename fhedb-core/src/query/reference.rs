//! # Reference Resolution
//!
//! Provides reference resolution utilities for cross-collection lookups.

use crate::{
    database::Database,
    document::{DocId, Document},
};
use fhedb_types::FieldType;

/// Trait for resolving document references across collections.
pub trait ReferenceResolvable {
    /// Resolves a reference value to a document in the specified collection.
    ///
    /// ## Arguments
    ///
    /// * `ref_value` - The reference value (document ID as string).
    /// * `collection_name` - The name of the collection to look up.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Document`]) if found, or [`None`] if the collection
    /// doesn't exist, the ID format is invalid, or no document matches.
    fn resolve_reference(&self, ref_value: &str, collection_name: &str) -> Option<Document>;
}

impl ReferenceResolvable for Database {
    fn resolve_reference(&self, ref_value: &str, collection_name: &str) -> Option<Document> {
        let collection = self.get_collection(collection_name)?;
        let id_field_def = collection.schema().fields.get(collection.id_field_name())?;

        let doc_id = match &id_field_def.field_type {
            FieldType::IdString => DocId::from(ref_value.to_string()),
            FieldType::IdInt => DocId::from(ref_value.parse::<u64>().ok()?),
            _ => return None,
        };

        collection.get_document(doc_id)
    }
}
