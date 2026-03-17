//! # Document Filtering
//!
//! Provides document filtering utilities for query operations.

use crate::{collection::Collection, document::Document, schema::SchemaOps};
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
    /// TODO: replace with iterated variant.
    pub fn filter(&self, conditions: &[FieldCondition]) -> Result<Vec<Document>, String> {
        let all_docs = self.get_documents();
        if conditions.is_empty() {
            return Ok(all_docs);
        }

        let mut filtered = Vec::new();
        for doc in all_docs {
            let matches = conditions.iter().try_fold(true, |acc, c| {
                self.schema()
                    .evaluate_condition(&doc.data, c)
                    .map(|m| acc && m)
            })?;
            if matches {
                filtered.push(doc);
            }
        }
        Ok(filtered)
    }

    pub fn filter_iter<'a>(
        &'a self,
        conditions: &'a [FieldCondition],
    ) -> impl Iterator<Item = Result<Document, String>> + 'a {
        self.iter_documents().filter_map(move |doc| {
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
