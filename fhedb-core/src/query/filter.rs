//! # Document Filtering
//!
//! Provides document filtering utilities for query operations.

use crate::{collection::Collection, document::Document, errors::Result, schema::SchemaOps};
use fhedb_types::FieldCondition;

/// Document filtering operations for query execution.
impl Collection {
    /// Filters documents based on conditions.
    ///
    /// ## Arguments
    ///
    /// * `conditions` - The conditions to apply (AND logic).
    ///
    /// ## Returns
    ///
    /// Returns matching documents. Empty conditions returns all documents.
    pub fn filter<'a>(
        &'a self,
        conditions: &'a [FieldCondition],
    ) -> impl Iterator<Item = Result<Document>> + 'a {
        self.get_documents().filter_map(move |doc| {
            if conditions.is_empty() {
                return Some(Ok(doc));
            }

            match conditions.iter().try_fold(true, |acc, c| {
                self.schema()
                    .evaluate_condition(&doc.data, c)
                    .map(|m| acc && m)
            }) {
                Ok(true) => Some(Ok(doc)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        })
    }
}
