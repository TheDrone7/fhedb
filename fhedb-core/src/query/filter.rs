//! # Document Filtering
//!
//! Provides document filtering utilities for query operations.

use fhedb_types::FieldCondition;

use crate::{collection::Collection, document::Document, query::comparison::ConditionEvaluable};

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
    pub fn filter(&self, conditions: &[FieldCondition]) -> Result<Vec<Document>, String> {
        let all_docs = self.get_documents();
        if conditions.is_empty() {
            return Ok(all_docs);
        }

        let mut filtered = Vec::new();
        for doc in all_docs {
            let matches = conditions.iter().try_fold(true, |acc, c| {
                doc.data.matches(c, self.schema()).map(|m| acc && m)
            })?;
            if matches {
                filtered.push(doc);
            }
        }
        Ok(filtered)
    }
}
