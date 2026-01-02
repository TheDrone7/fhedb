//! # Field Selection
//!
//! Provides field selection utilities for query responses.

use bson::Document as BsonDocument;
use fhedb_types::{FieldSelector, Schema};

/// Trait for selecting specific fields from a document.
pub trait FieldSelectable {
    /// Selects fields from this document based on selectors.
    ///
    /// ## Arguments
    ///
    /// * `selectors` - The field selectors to apply.
    /// * `schema` - The collection schema for field validation.
    ///
    /// ## Returns
    ///
    /// Returns a new [`BsonDocument`] containing only selected fields,
    /// or [`Err`]\([`String`]) if a selector references an unknown field.
    fn select_fields(
        &self,
        selectors: &[FieldSelector],
        schema: &Schema,
    ) -> Result<BsonDocument, String>;
}

impl FieldSelectable for BsonDocument {
    fn select_fields(
        &self,
        selectors: &[FieldSelector],
        schema: &Schema,
    ) -> Result<BsonDocument, String> {
        if selectors.is_empty() {
            return Ok(BsonDocument::new());
        }

        let mut result = BsonDocument::new();
        for selector in selectors {
            match selector {
                FieldSelector::Field(name) => {
                    if !schema.fields.contains_key(name) {
                        return Err(format!("Unknown field '{}'.", name));
                    }
                    if let Some(value) = self.get(name) {
                        result.insert(name.clone(), value.clone());
                    }
                }
                FieldSelector::AllFields | FieldSelector::AllFieldsRecursive => {
                    for (key, value) in self {
                        result.insert(key.clone(), value.clone());
                    }
                }
                FieldSelector::SubDocument { field_name, .. } => {
                    if !schema.fields.contains_key(field_name) {
                        return Err(format!("Unknown field '{}'.", field_name));
                    }
                    let value = self.get(field_name).cloned().unwrap_or(bson::Bson::Null);
                    result.insert(field_name.clone(), value);
                }
            }
        }
        Ok(result)
    }
}
