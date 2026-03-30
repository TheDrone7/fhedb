//! # Document Query Handlers
//!
//! Handles document operations (INSERT, GET, UPDATE, DELETE) within a database context.

use std::collections::HashMap;

use bson::{Bson, Document as BsonDocument};
use fhedb_core::prelude::{
    Database, FieldType, ReferenceChecker, Schema, SchemaOps, ValueParseable,
};
use fhedb_types::{DocumentQuery, FieldCondition, FieldSelector};
use serde_json::{Value as JsonValue, json};

use crate::{errors::AppError, state::ServerState};

/// Executes a document-level query within a specific database.
///
/// ## Arguments
///
/// * `db_name` - The name of the database to operate on.
/// * `query` - The [`DocumentQuery`] to execute.
/// * `state` - The [`ServerState`] containing database references.
///
/// ## Returns
///
/// Returns [`Ok`]\([`serde_json::Value`]) on success, or [`Err`]\([`String`]) on failure.
pub fn execute_document_query(
    db_name: String,
    query: DocumentQuery,
    state: &ServerState,
) -> Result<JsonValue, AppError> {
    match query {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => execute_insert(db_name, collection_name, fields, state),
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => execute_get(db_name, collection_name, conditions, selectors, state),
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            selectors,
        } => execute_update(
            db_name,
            collection_name,
            conditions,
            updates,
            selectors,
            state,
        ),
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => execute_delete(db_name, collection_name, conditions, selectors, state),
    }
}

/// Executes an INSERT document query.
///
/// ## Arguments
///
/// * `db_name` - The name of the database.
/// * `collection_name` - The name of the collection to insert into.
/// * `fields` - The field-value pairs to insert.
/// * `state` - The server state.
///
/// ## Returns
///
/// Returns the inserted document as a JSON array with one element.
fn execute_insert(
    db_name: String,
    collection_name: String,
    fields: HashMap<String, String>,
    state: &ServerState,
) -> Result<JsonValue, AppError> {
    let mut dbs = state
        .databases
        .write()
        .map_err(|e| AppError::Server(e.to_string()))?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| AppError::Server(format!("Database '{}' not found.", db_name)))?;
    let collection = db.get_collection_mut(&collection_name).ok_or_else(|| {
        AppError::Core(fhedb_core::errors::Error::CollectionNotFound(
            collection_name,
        ))
    })?;

    let doc = collection.schema().prepare_document(&fields)?;
    let doc_id = collection.add_document(doc)?;
    let inserted = collection.get_document(doc_id).ok_or(AppError::Server(
        "Failed to retrieve inserted document.".to_string(),
    ))?;

    Ok(json!([
        serde_json::to_value(&inserted.data).map_err(|e| AppError::Server(e.to_string()))?
    ]))
}

/// Executes a GET document query.
///
/// ## Arguments
///
/// * `db_name` - The name of the database.
/// * `collection_name` - The name of the collection to query.
/// * `conditions` - The filter conditions.
/// * `selectors` - The fields to return.
/// * `state` - The server state.
///
/// ## Returns
///
/// Returns matching documents as a JSON array.
fn execute_get(
    db_name: String,
    collection_name: String,
    conditions: Vec<FieldCondition>,
    selectors: Vec<FieldSelector>,
    state: &ServerState,
) -> Result<JsonValue, AppError> {
    let mut dbs = state
        .databases
        .write()
        .map_err(|e| AppError::Server(e.to_string()))?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| AppError::Server(format!("Database '{}' not found.", db_name)))?;

    let collection = db.get_collection(&collection_name).unwrap();
    let schema = collection.schema().clone();

    let filtered = collection
        .filter(&conditions)
        .collect::<Result<Vec<_>, _>>()?;
    let doc_data: Vec<BsonDocument> = filtered.iter().map(|doc| doc.data.clone()).collect();

    let results: Result<Vec<_>, _> = doc_data
        .iter()
        .map(|data| select_fields(data, &selectors, &schema, db, 1))
        .collect();

    Ok(JsonValue::Array(results?))
}

/// Executes an UPDATE document query with rollback on failure.
///
/// ## Arguments
///
/// * `db_name` - The name of the database.
/// * `collection_name` - The name of the collection to update.
/// * `conditions` - The filter conditions.
/// * `updates` - The field updates to apply.
/// * `selectors` - The fields to return.
/// * `state` - The server state.
///
/// ## Returns
///
/// Returns updated documents as a JSON array.
fn execute_update(
    db_name: String,
    collection_name: String,
    conditions: Vec<FieldCondition>,
    updates: HashMap<String, String>,
    selectors: Vec<FieldSelector>,
    state: &ServerState,
) -> Result<JsonValue, AppError> {
    let mut dbs = state
        .databases
        .write()
        .map_err(|e| AppError::Server(e.to_string()))?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| AppError::Server(format!("Database '{}' not found.", db_name)))?;

    let collection = db.get_collection_mut(&collection_name).unwrap();
    let schema = collection.schema().clone();
    let matching: Vec<_> = collection
        .filter(&conditions)
        .collect::<Result<Vec<_>, _>>()?;
    if matching.is_empty() {
        return Ok(json!([]));
    }

    let originals: Vec<_> = matching
        .iter()
        .map(|d| (d.id.clone(), d.data.clone()))
        .collect();
    let matching_ids: Vec<_> = matching.into_iter().map(|d| d.id).collect();

    let update_doc = convert_fields_to_bson(&updates, &schema)?;

    let mut updated_docs = Vec::new();
    for (idx, id) in matching_ids.iter().enumerate() {
        match collection.update_document(id.clone(), update_doc.clone()) {
            Ok(doc) => updated_docs.push(doc.data),
            Err(errors) => {
                for (orig_id, orig_data) in originals.iter().take(idx) {
                    let _ = collection.update_document(orig_id.clone(), orig_data.clone());
                }
                return Err(AppError::Server(format!(
                    "Update failed and rolled back: {:?}",
                    errors
                )));
            }
        }
    }

    let results: Result<Vec<_>, _> = updated_docs
        .iter()
        .map(|data| select_fields(data, &selectors, &schema, db, 1))
        .collect();

    Ok(JsonValue::Array(results?))
}

/// Executes a DELETE document query.
///
/// ## Arguments
///
/// * `db_name` - The name of the database.
/// * `collection_name` - The name of the collection to delete from.
/// * `conditions` - The filter conditions.
/// * `selectors` - The fields to return from deleted documents.
/// * `state` - The server state.
///
/// ## Returns
///
/// Returns deleted documents as a JSON array.
fn execute_delete(
    db_name: String,
    collection_name: String,
    conditions: Vec<FieldCondition>,
    selectors: Vec<FieldSelector>,
    state: &ServerState,
) -> Result<JsonValue, AppError> {
    let mut dbs = state
        .databases
        .write()
        .map_err(|e| AppError::Server(e.to_string()))?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| AppError::Server(format!("Database '{}' not found.", db_name)))?;

    let collection = db.get_collection_mut(&collection_name).unwrap();
    let schema = collection.schema().clone();
    let matching = collection
        .filter(&conditions)
        .collect::<Result<Vec<_>, _>>()?;

    if matching.is_empty() {
        return Ok(json!([]));
    }

    let doc_data: Vec<BsonDocument> = matching.iter().map(|doc| doc.data.clone()).collect();
    let matching_ids: Vec<_> = matching.into_iter().map(|d| d.id).collect();

    let results: Result<Vec<_>, _> = doc_data
        .iter()
        .map(|data| select_fields(data, &selectors, &schema, db, 1))
        .collect();
    let results = results?;

    let collection = db.get_collection_mut(&collection_name).unwrap();
    for id in matching_ids {
        collection.remove_document(id);
    }

    Ok(JsonValue::Array(results))
}

/// Converts string field values to typed BSON based on schema.
///
/// ## Arguments
///
/// * `fields` - The field name to string value mapping.
/// * `schema` - The collection schema for type information.
///
/// ## Returns
///
/// Returns a [`BsonDocument`] with typed values.
fn convert_fields_to_bson(
    fields: &HashMap<String, String>,
    schema: &Schema,
) -> Result<BsonDocument, AppError> {
    let mut doc = BsonDocument::new();
    for (field_name, value_str) in fields {
        let field_def = schema.fields.get(field_name).ok_or_else(|| {
            AppError::Core(fhedb_core::errors::Error::Schema(format!(
                "Unknown field '{}'",
                field_name
            )))
        })?;
        doc.insert(
            field_name.clone(),
            value_str.parse_as_bson(&field_def.field_type)?,
        );
    }
    Ok(doc)
}

/// Selects fields from a document based on selectors.
///
/// ## Arguments
///
/// * `doc` - The BSON document.
/// * `selectors` - The field selectors.
/// * `schema` - The collection schema for field type lookups.
/// * `database` - The database for reference resolution.
/// * `depth` - Current recursion depth (max 3).
///
/// ## Returns
///
/// Returns a JSON object with selected fields. Empty selectors returns `{}`.
fn select_fields(
    doc: &BsonDocument,
    selectors: &[FieldSelector],
    schema: &Schema,
    database: &mut Database,
    depth: u8,
) -> Result<JsonValue, AppError> {
    if selectors.is_empty() {
        return Ok(json!({}));
    }

    let selected = schema.select_fields(doc, selectors)?;
    let mut result: serde_json::Map<String, JsonValue> = serde_json::from_value(
        serde_json::to_value(&selected).map_err(|e| AppError::Server(e.to_string()))?,
    )
    .map_err(|e| AppError::Server(e.to_string()))?;

    for selector in selectors {
        match selector {
            FieldSelector::AllFieldsRecursive => {
                for (key, value) in doc {
                    let field_def = schema.fields.get(key).ok_or_else(|| {
                        AppError::Core(fhedb_core::errors::Error::Execution(format!(
                            "Unknown field '{}'.",
                            key
                        )))
                    })?;

                    let resolved = if field_def.field_type.contains_reference() {
                        resolve_reference(
                            value,
                            &field_def.field_type,
                            key,
                            &[],
                            &[FieldSelector::AllFieldsRecursive],
                            database,
                            depth,
                        )?
                    } else {
                        serde_json::to_value(value).map_err(|e| AppError::Server(e.to_string()))?
                    };
                    result.insert(key.to_string(), resolved);
                }
            }
            FieldSelector::SubDocument {
                field_name,
                content,
            } => {
                let field_def = schema.fields.get(field_name).ok_or_else(|| {
                    AppError::Core(fhedb_core::errors::Error::Execution(format!(
                        "Unknown field '{}'.",
                        field_name
                    )))
                })?;
                let field_value = doc.get(field_name).cloned().unwrap_or(Bson::Null);
                result.insert(
                    field_name.clone(),
                    resolve_reference(
                        &field_value,
                        &field_def.field_type,
                        field_name,
                        &content.conditions,
                        &content.selectors,
                        database,
                        depth,
                    )?,
                );
            }
            _ => {}
        }
    }
    Ok(JsonValue::Object(result))
}

/// Resolves a reference field value by fetching referenced documents.
///
/// ## Arguments
///
/// * `value` - The reference field value (string ID or array of IDs).
/// * `field_type` - The field's type from schema.
/// * `field_name` - The field name for error messages.
/// * `conditions` - Conditions to filter referenced documents.
/// * `selectors` - Selectors to apply on referenced documents.
/// * `database` - Database for reference lookups.
/// * `depth` - Current recursion depth (max 3).
///
/// ## Returns
///
/// Returns the resolved JSON value, or null if not found or conditions don't match.
fn resolve_reference(
    value: &Bson,
    field_type: &FieldType,
    field_name: &str,
    conditions: &[FieldCondition],
    selectors: &[FieldSelector],
    database: &mut Database,
    depth: u8,
) -> Result<JsonValue, AppError> {
    if depth >= 3 {
        return serde_json::to_value(value).map_err(|e| AppError::Server(e.to_string()));
    }

    match field_type {
        FieldType::Reference(ref_col) => match value {
            Bson::String(ref_id) => {
                let ref_doc = match database.resolve_reference(ref_id, ref_col) {
                    Some(d) => d,
                    None => return Ok(JsonValue::Null),
                };

                let ref_schema = match database.get_collection(ref_col) {
                    Some(c) => c.schema().clone(),
                    None => return Ok(JsonValue::Null),
                };

                for condition in conditions {
                    if !ref_schema.evaluate_condition(&ref_doc.data, condition)? {
                        return Ok(JsonValue::Null);
                    }
                }

                select_fields(&ref_doc.data, selectors, &ref_schema, database, depth + 1)
            }
            Bson::Null => Ok(JsonValue::Null),
            _ => serde_json::to_value(value).map_err(|e| AppError::Server(e.to_string())),
        },
        FieldType::Nullable(inner) => match value {
            Bson::Null => Ok(JsonValue::Null),
            _ => resolve_reference(
                value, inner, field_name, conditions, selectors, database, depth,
            ),
        },
        FieldType::Array(inner) => match value {
            Bson::Array(arr) => {
                let results: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|item| {
                        resolve_reference(
                            item, inner, field_name, conditions, selectors, database, depth,
                        )
                    })
                    .collect();
                let filtered: Vec<_> = results?.into_iter().filter(|v| !v.is_null()).collect();
                Ok(JsonValue::Array(filtered))
            }
            Bson::Null => Ok(JsonValue::Array(vec![])),
            _ => Ok(JsonValue::Array(vec![])),
        },
        _ => Err(AppError::Core(fhedb_core::errors::Error::Execution(
            format!("Field '{}' is not a reference type.", field_name),
        ))),
    }
}
