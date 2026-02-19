//! # Document Query Handlers
//!
//! This module handles document operations within a database context,
//! including INSERT, GET, UPDATE, and DELETE operations.

use std::collections::HashMap;

use bson::{Bson, Document as BsonDocument};
use fhedb_core::prelude::{
    Collection, ConditionEvaluable, Database, DocumentPreparable, FieldSelectable, FieldType,
    Schema, ValueParseable,
};
use fhedb_types::{DocumentQuery, FieldCondition, FieldSelector};
use serde_json::{Value as JsonValue, json};

use crate::state::ServerState;

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
) -> Result<JsonValue, String> {
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
) -> Result<JsonValue, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| format!("Database '{}' not found.", db_name))?;
    let collection = db
        .get_collection_mut(&collection_name)
        .ok_or_else(|| format!("Collection '{}' not found.", collection_name))?;

    let doc = fields.prepare_document(collection.schema())?;
    let doc_id = collection.add_document(doc).map_err(|e| e.join("; "))?;
    let inserted = collection
        .get_document(doc_id)
        .ok_or("Failed to retrieve inserted document.")?;

    Ok(json!([
        serde_json::to_value(&inserted.data).map_err(|e| e.to_string())?
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
) -> Result<JsonValue, String> {
    let dbs = state.databases.read().map_err(|e| e.to_string())?;
    let db = dbs
        .get(&db_name)
        .ok_or_else(|| format!("Database '{}' not found.", db_name))?;
    let collection = db
        .get_collection(&collection_name)
        .ok_or_else(|| format!("Collection '{}' not found.", collection_name))?;

    let filtered = collection.filter(&conditions)?;
    let results: Result<Vec<_>, _> = filtered
        .iter()
        .map(|doc| select_fields(&doc.data, &selectors, collection, db, 1))
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
) -> Result<JsonValue, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| format!("Database '{}' not found.", db_name))?;

    let collection = db
        .get_collection(&collection_name)
        .ok_or_else(|| format!("Collection '{}' not found.", collection_name))?;

    let matching: Vec<_> = collection.filter(&conditions)?;
    if matching.is_empty() {
        return Ok(json!([]));
    }

    let originals: Vec<_> = matching
        .iter()
        .map(|d| (d.id.clone(), d.data.clone()))
        .collect();
    let matching_ids: Vec<_> = matching.into_iter().map(|d| d.id).collect();

    let collection = db.get_collection_mut(&collection_name).unwrap();
    let update_doc = convert_fields_to_bson(&updates, collection.schema())?;

    let mut updated_docs = Vec::new();
    for (idx, id) in matching_ids.iter().enumerate() {
        match collection.update_document(id.clone(), update_doc.clone()) {
            Ok(doc) => updated_docs.push(doc),
            Err(errors) => {
                for (orig_id, orig_data) in originals.iter().take(idx) {
                    let _ = collection.update_document(orig_id.clone(), orig_data.clone());
                }
                return Err(format!(
                    "Update failed and rolled back: {}",
                    errors.join("; ")
                ));
            }
        }
    }

    let db = dbs.get(&db_name).unwrap();
    let collection = db.get_collection(&collection_name).unwrap();
    let results: Result<Vec<_>, _> = updated_docs
        .iter()
        .map(|doc| select_fields(&doc.data, &selectors, collection, db, 1))
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
) -> Result<JsonValue, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| format!("Database '{}' not found.", db_name))?;

    let collection = db
        .get_collection(&collection_name)
        .ok_or_else(|| format!("Collection '{}' not found.", collection_name))?;
    let matching: Vec<_> = collection.filter(&conditions)?;

    if matching.is_empty() {
        return Ok(json!([]));
    }

    let results: Result<Vec<_>, _> = matching
        .iter()
        .map(|doc| select_fields(&doc.data, &selectors, collection, db, 1))
        .collect();
    let results = results?;

    let collection = db.get_collection_mut(&collection_name).unwrap();
    for doc in matching {
        collection.remove_document(doc.id);
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
) -> Result<BsonDocument, String> {
    let mut doc = BsonDocument::new();
    for (field_name, value_str) in fields {
        let field_def = schema
            .fields
            .get(field_name)
            .ok_or_else(|| format!("Unknown field '{}' not in schema.", field_name))?;
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
/// * `collection` - The source collection.
/// * `database` - The database for reference resolution.
/// * `depth` - Current recursion depth (max 3).
///
/// ## Returns
///
/// Returns a JSON object with selected fields. Empty selectors returns `{}`.
fn select_fields(
    doc: &BsonDocument,
    selectors: &[FieldSelector],
    collection: &Collection,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    if selectors.is_empty() {
        return Ok(json!({}));
    }

    let selected = doc.select_fields(selectors, collection.schema())?;
    let mut result: serde_json::Map<String, JsonValue> =
        serde_json::from_value(serde_json::to_value(&selected).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    for selector in selectors {
        match selector {
            FieldSelector::AllFieldsRecursive => {
                for (key, value) in doc {
                    let field_def = collection
                        .schema()
                        .fields
                        .get(key)
                        .ok_or_else(|| format!("Unknown field '{}'.", key))?;

                    let resolved = if contains_reference(&field_def.field_type) {
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
                        serde_json::to_value(value).map_err(|e| e.to_string())?
                    };
                    result.insert(key.to_string(), resolved);
                }
            }
            FieldSelector::SubDocument {
                field_name,
                content,
            } => {
                let field_def = collection
                    .schema()
                    .fields
                    .get(field_name)
                    .ok_or_else(|| format!("Unknown field '{}'.", field_name))?;
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
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    if depth >= 3 {
        return serde_json::to_value(value).map_err(|e| e.to_string());
    }

    match field_type {
        FieldType::Reference(ref_col) => match value {
            Bson::String(ref_id) => {
                let ref_doc = match database.resolve_reference(ref_id, ref_col) {
                    Some(d) => d,
                    None => return Ok(JsonValue::Null),
                };
                let ref_collection = match database.get_collection(ref_col) {
                    Some(c) => c,
                    None => return Ok(JsonValue::Null),
                };

                for condition in conditions {
                    if !ref_doc.data.matches(condition, ref_collection.schema())? {
                        return Ok(JsonValue::Null);
                    }
                }

                select_fields(
                    &ref_doc.data,
                    selectors,
                    ref_collection,
                    database,
                    depth + 1,
                )
            }
            Bson::Null => Ok(JsonValue::Null),
            _ => serde_json::to_value(value).map_err(|e| e.to_string()),
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
        _ => Err(format!("Field '{}' is not a reference type.", field_name)),
    }
}

/// Checks if a field type contains or is a reference type.
///
/// Recursively inspects `Nullable` and `Array` wrappers to find nested references.
///
/// ## Arguments
///
/// * `field_type` - The [`FieldType`] to check.
///
/// ## Returns
///
/// Returns `true` if the field type is or contains a [`FieldType::Reference`].
fn contains_reference(field_type: &FieldType) -> bool {
    match field_type {
        FieldType::Reference(_) => true,
        FieldType::Nullable(inner) | FieldType::Array(inner) => contains_reference(inner),
        _ => false,
    }
}
