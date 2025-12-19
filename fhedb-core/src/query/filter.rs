//! # Document Filtering
//!
//! Provides document filtering utilities for query operations.

use fhedb_types::FieldCondition;

use crate::db::collection::Collection;
use crate::db::document::Document;
use crate::query::comparison::evaluate_condition;

/// Filters documents from a collection based on conditions.
///
/// ## Arguments
///
/// * `collection` - The collection to filter.
/// * `conditions` - The conditions to apply (AND logic).
///
/// ## Returns
///
/// Returns matching documents. Empty conditions returns all documents.
pub fn filter_documents(
    collection: &Collection,
    conditions: &[FieldCondition],
) -> Result<Vec<Document>, String> {
    let all_docs = collection.get_documents();
    if conditions.is_empty() {
        return Ok(all_docs);
    }

    let mut filtered = Vec::new();
    for doc in all_docs {
        let matches = conditions.iter().try_fold(true, |acc, c| {
            evaluate_condition(&doc.data, c, collection.schema()).map(|m| acc && m)
        })?;
        if matches {
            filtered.push(doc);
        }
    }
    Ok(filtered)
}
