//! # Reference Utilities
//!
//! Provides utilities for inspecting and validating schema references.

use fhedb_types::{FieldType, Schema};

use super::database::Database;

/// Extension trait for [`FieldType`] reference utilities.
pub trait ReferenceChecker {
    /// Checks if this field type is or contains a reference.
    fn contains_reference(&self) -> bool;

    /// Checks if this field type references the specified collection.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to check for.
    fn references_collection(&self, collection_name: &str) -> bool;

    /// Finds the first invalid reference in this field type.
    ///
    /// Self-references are allowed when `self_collection` matches.
    ///
    /// ## Arguments
    ///
    /// * `db` - The [`Database`] to check collection existence against.
    /// * `self_collection` - Optional name of the collection being created or modified.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] with the invalid collection name, or [`None`] if all references are valid.
    fn find_invalid_reference(
        &self,
        db: &Database,
        self_collection: Option<&str>,
    ) -> Option<String>;
}

impl ReferenceChecker for FieldType {
    fn contains_reference(&self) -> bool {
        match self {
            FieldType::Reference(_) => true,
            FieldType::Array(inner) | FieldType::Nullable(inner) => inner.contains_reference(),
            _ => false,
        }
    }

    fn references_collection(&self, collection_name: &str) -> bool {
        match self {
            FieldType::Reference(name) => name == collection_name,
            FieldType::Array(inner) | FieldType::Nullable(inner) => {
                inner.references_collection(collection_name)
            }
            _ => false,
        }
    }

    fn find_invalid_reference(
        &self,
        db: &Database,
        self_collection: Option<&str>,
    ) -> Option<String> {
        match self {
            FieldType::Reference(name) => {
                let is_self = self_collection.is_some_and(|s| s == name);
                if is_self || db.has_collection(name) {
                    None
                } else {
                    Some(name.clone())
                }
            }
            FieldType::Array(inner) | FieldType::Nullable(inner) => {
                inner.find_invalid_reference(db, self_collection)
            }
            _ => None,
        }
    }
}

/// Extension trait for [`Schema`] reference utilities.
pub trait SchemaReferenceValidator {
    /// Validates that all reference fields point to existing collections.
    ///
    /// ## Arguments
    ///
    /// * `db` - The [`Database`] to check collection existence against.
    /// * `self_collection` - Optional name of the collection being created or modified.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if all references are valid, or [`Err`]\([`String`]) with the first invalid reference.
    fn validate_references(
        &self,
        db: &Database,
        self_collection: Option<&str>,
    ) -> Result<(), String>;
}

impl SchemaReferenceValidator for Schema {
    fn validate_references(
        &self,
        db: &Database,
        self_collection: Option<&str>,
    ) -> Result<(), String> {
        for field_def in self.fields.values() {
            if let Some(invalid) = field_def
                .field_type
                .find_invalid_reference(db, self_collection)
            {
                return Err(format!("Collection '{}' does not exist.", invalid));
            }
        }
        Ok(())
    }
}

impl Database {
    /// Finds all collections that reference the specified collection.
    ///
    /// Self-references are excluded from the results.
    ///
    /// ## Arguments
    ///
    /// * `target_collection` - The name of the collection to find references to.
    ///
    /// ## Returns
    ///
    /// A vector of collection names that reference the target collection.
    pub fn find_referencing_collections(&self, target_collection: &str) -> Vec<String> {
        let mut referencing = Vec::new();

        for collection_name in self.collection_names() {
            if collection_name == target_collection {
                continue;
            }

            if let Some(col) = self.get_collection(&collection_name) {
                for field_def in col.schema().fields.values() {
                    if field_def
                        .field_type
                        .references_collection(target_collection)
                    {
                        referencing.push(collection_name.clone());
                        break;
                    }
                }
            }
        }

        referencing
    }
}
