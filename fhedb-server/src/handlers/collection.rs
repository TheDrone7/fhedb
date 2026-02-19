//! # Collection Query Handlers
//!
//! This module handles collection operations within a database context,
//! such as creating, dropping, modifying, and listing collections.

use crate::state::ServerState;
use fhedb_core::prelude::{
    FieldDefinition, FieldType, ReferenceChecker, Schema, SchemaReferenceValidator,
};
use fhedb_types::{CollectionQuery, FieldModification};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

/// JSON-serializable representation of a field definition.
#[derive(Serialize)]
struct JsonFieldDefinition {
    /// The type of the field.
    #[serde(rename = "type")]
    field_type: String,
    /// The default value for the field, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<serde_json::Value>,
    /// Whether the field can be null.
    nullable: bool,
}

impl From<&FieldDefinition> for JsonFieldDefinition {
    fn from(def: &FieldDefinition) -> Self {
        let (type_str, nullable) = extract_type_info(&def.field_type);
        JsonFieldDefinition {
            field_type: type_str,
            default: def
                .default_value
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok()),
            nullable,
        }
    }
}

/// Extracts type information from a [`FieldType`], returning the type string and nullability.
///
/// ## Arguments
///
/// * `ft` - The [`FieldType`] to extract information from.
fn extract_type_info(ft: &FieldType) -> (String, bool) {
    match ft {
        FieldType::Nullable(inner) => {
            let (inner_str, _) = extract_type_info(inner);
            (inner_str, true)
        }
        other => (format_field_type(other), false),
    }
}

/// Formats a [`FieldType`] as a human-readable string.
///
/// ## Arguments
///
/// * `ft` - The [`FieldType`] to format.
fn format_field_type(ft: &FieldType) -> String {
    match ft {
        FieldType::Int => "int".to_string(),
        FieldType::Float => "float".to_string(),
        FieldType::Boolean => "boolean".to_string(),
        FieldType::String => "string".to_string(),
        FieldType::IdString => "id_string".to_string(),
        FieldType::IdInt => "id_int".to_string(),
        FieldType::Array(inner) => format!("array({})", format_field_type(inner)),
        FieldType::Reference(r) => format!("reference({})", r),
        FieldType::Nullable(inner) => format!("nullable({})", format_field_type(inner)),
    }
}

/// Serializes a [`Schema`] to a JSON value.
///
/// ## Arguments
///
/// * `schema` - The [`Schema`] to serialize.
///
/// ## Returns
///
/// Returns [`Ok`]\([`serde_json::Value`]) on success, or [`Err`]\([`String`]) on failure.
fn serialize_schema(schema: &Schema) -> Result<serde_json::Value, String> {
    let schema_map: HashMap<String, JsonFieldDefinition> = schema
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), JsonFieldDefinition::from(v)))
        .collect();
    serde_json::to_value(&schema_map).map_err(|e| e.to_string())
}

/// Executes a collection-level query and returns the result.
///
/// ## Arguments
///
/// * `db_name` - The name of the database to operate on.
/// * `query` - The [`CollectionQuery`] to execute.
/// * `state` - The [`ServerState`] containing database references.
///
/// ## Returns
///
/// Returns [`Ok`]\([`serde_json::Value`]) on success, or [`Err`]\([`String`]) on failure.
pub fn execute_collection_query(
    db_name: String,
    query: CollectionQuery,
    state: &ServerState,
) -> Result<serde_json::Value, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| "Database not found".to_string())?;

    match query {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            if drop_if_exists && db.has_collection(&name) {
                db.drop_collection(&name)?;
            }
            schema.validate_references(db, Some(&name))?;
            db.create_collection(&name, schema)?;
            let col = db
                .get_collection(&name)
                .ok_or("Collection not found after creation")?;
            serialize_schema(col.schema())
        }
        CollectionQuery::Drop { name } => {
            let referencing = db.find_referencing_collections(&name);
            if !referencing.is_empty() {
                return Err(format!(
                    "Cannot drop collection '{}'. It is referenced by: {}.",
                    name,
                    referencing.join(", ")
                ));
            }
            db.drop_collection(&name)?;
            Ok(json!({ "dropped": name }))
        }
        CollectionQuery::List => {
            let names = db.collection_names();
            Ok(json!(names))
        }
        CollectionQuery::GetSchema { name } => {
            let col = db
                .get_collection(&name)
                .ok_or_else(|| format!("Collection '{}' not found", name))?;
            serialize_schema(col.schema())
        }
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            for modification in modifications.values() {
                if let FieldModification::Set(def) = modification
                    && let Some(invalid_ref) =
                        def.field_type.find_invalid_reference(db, Some(&name))
                {
                    return Err(format!("Collection '{}' does not exist.", invalid_ref));
                }
            }

            let col = db
                .get_collection_mut(&name)
                .ok_or_else(|| format!("Collection '{}' not found", name))?;
            for (field_name, modification) in modifications {
                match modification {
                    FieldModification::Drop => {
                        col.remove_field(&field_name)?;
                    }
                    FieldModification::Set(def) => {
                        if col.has_field(&field_name) {
                            col.modify_field(&field_name, def)?;
                        } else {
                            col.add_field(field_name, def)?;
                        }
                    }
                }
            }
            serialize_schema(col.schema())
        }
    }
}
