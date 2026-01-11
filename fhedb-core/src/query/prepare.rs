//! # Document Preparation
//!
//! Provides utilities for preparing documents for insertion.

use bson::{Bson, Document as BsonDocument};
use fhedb_types::{FieldType, Schema};
use std::collections::HashMap;

use crate::query::value::ValueParseable;

/// Trait for preparing documents from string assignments.
pub trait DocumentPreparable {
    /// Prepares a document for insertion by parsing field values and applying defaults.
    ///
    /// ## Arguments
    ///
    /// * `schema` - The collection schema for type information and defaults.
    ///
    /// ## Returns
    ///
    /// Returns a [`BsonDocument`] with parsed values and defaults applied,
    /// or [`Err`]\([`String`]) if a field is unknown or cannot be parsed.
    fn prepare_document(&self, schema: &Schema) -> Result<BsonDocument, String>;
}

impl DocumentPreparable for HashMap<String, String> {
    fn prepare_document(&self, schema: &Schema) -> Result<BsonDocument, String> {
        let mut doc = BsonDocument::new();

        for (field_name, value_str) in self {
            let field_def = schema
                .fields
                .get(field_name)
                .ok_or_else(|| format!("Unknown field '{}'.", field_name))?;
            doc.insert(
                field_name.clone(),
                value_str.parse_as_bson(&field_def.field_type)?,
            );
        }

        for (field_name, field_def) in &schema.fields {
            if doc.contains_key(field_name) {
                continue;
            }
            match &field_def.field_type {
                FieldType::IdString | FieldType::IdInt => continue,
                FieldType::Nullable(_) => {
                    doc.insert(
                        field_name.clone(),
                        field_def.default_value.clone().unwrap_or(Bson::Null),
                    );
                }
                FieldType::Array(_) => {
                    doc.insert(
                        field_name.clone(),
                        field_def
                            .default_value
                            .clone()
                            .unwrap_or(Bson::Array(vec![])),
                    );
                }
                FieldType::Reference(_) => {
                    doc.insert(
                        field_name.clone(),
                        field_def.default_value.clone().unwrap_or(Bson::Null),
                    );
                }
                _ => {
                    if let Some(default) = &field_def.default_value {
                        doc.insert(field_name.clone(), default.clone());
                    } else {
                        return Err(format!("Missing required field '{}'.", field_name));
                    }
                }
            }
        }
        Ok(doc)
    }
}
