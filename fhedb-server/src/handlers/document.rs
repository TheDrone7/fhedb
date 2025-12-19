//! # Document Query Handlers
//!
//! This module handles document operations within a database context,
//! including INSERT, GET, UPDATE, and DELETE operations.

use std::collections::HashMap;

use bson::{Bson, Document as BsonDocument};
use fhedb_core::prelude::{
    Collection, Database, DocId, Document, FieldType, Schema, parse_bson_value,
};
use fhedb_types::{DocumentQuery, FieldCondition, FieldSelector, ParsedDocContent, QueryOperator};
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

    let doc = prepare_insert_doc(&fields, collection.schema())?;
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

    let filtered = filter_documents(collection, &conditions)?;
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

    let matching: Vec<_> = filter_documents(collection, &conditions)?;
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
    let matching: Vec<_> = filter_documents(collection, &conditions)?;

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

/// Prepares a document for insertion by converting fields and ensuring schema completeness.
///
/// ## Arguments
///
/// * `fields` - The field name to string value mapping.
/// * `schema` - The collection schema.
///
/// ## Returns
///
/// Returns the completed [`BsonDocument`] ready for insertion.
fn prepare_insert_doc(
    fields: &HashMap<String, String>,
    schema: &Schema,
) -> Result<BsonDocument, String> {
    let mut doc = convert_fields_to_bson(fields, schema)?;

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
            parse_bson_value(value_str, &field_def.field_type)?,
        );
    }
    Ok(doc)
}

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
fn filter_documents(
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

/// Evaluates a single condition against a document.
///
/// ## Arguments
///
/// * `doc` - The BSON document to evaluate.
/// * `condition` - The condition to check.
/// * `schema` - The collection schema.
///
/// ## Returns
///
/// Returns `true` if the condition matches.
fn evaluate_condition(
    doc: &BsonDocument,
    condition: &FieldCondition,
    schema: &Schema,
) -> Result<bool, String> {
    let field_def = schema
        .fields
        .get(&condition.field_name)
        .ok_or_else(|| format!("Unknown field '{}'.", condition.field_name))?;
    let condition_value = parse_bson_value(&condition.value, &field_def.field_type)?;

    match doc.get(&condition.field_name) {
        None => Ok(false),
        Some(Bson::Null) => Ok(match condition.operator {
            QueryOperator::Equal => condition_value == Bson::Null,
            QueryOperator::NotEqual => condition_value != Bson::Null,
            _ => false,
        }),
        Some(doc_val) => match &condition.operator {
            QueryOperator::Equal => Ok(doc_val == &condition_value),
            QueryOperator::NotEqual => Ok(doc_val != &condition_value),
            QueryOperator::GreaterThan => {
                compare_bson(doc_val, &condition_value, &condition.operator)
            }
            QueryOperator::GreaterThanOrEqual => {
                compare_bson(doc_val, &condition_value, &condition.operator)
            }
            QueryOperator::LessThan => compare_bson(doc_val, &condition_value, &condition.operator),
            QueryOperator::LessThanOrEqual => {
                compare_bson(doc_val, &condition_value, &condition.operator)
            }
            QueryOperator::Similar => Ok(match (doc_val, &condition_value) {
                (Bson::String(s), Bson::String(p)) => s.contains(p.as_str()),
                (Bson::Array(arr), _) => arr.contains(&condition_value),
                _ => false,
            }),
        },
    }
}

/// Compares two BSON values using the given operator.
///
/// ## Arguments
///
/// * `a` - First value.
/// * `b` - Second value.
/// * `op` - The comparison operator.
///
/// ## Returns
///
/// Returns the comparison result or error for incompatible types.
fn compare_bson(a: &Bson, b: &Bson, op: &QueryOperator) -> Result<bool, String> {
    let result = match (a, b) {
        (Bson::Int32(x), Bson::Int32(y)) => match op {
            QueryOperator::GreaterThan => x > y,
            QueryOperator::GreaterThanOrEqual => x >= y,
            QueryOperator::LessThan => x < y,
            QueryOperator::LessThanOrEqual => x <= y,
            _ => false,
        },
        (Bson::Int64(x), Bson::Int64(y)) => match op {
            QueryOperator::GreaterThan => x > y,
            QueryOperator::GreaterThanOrEqual => x >= y,
            QueryOperator::LessThan => x < y,
            QueryOperator::LessThanOrEqual => x <= y,
            _ => false,
        },
        (Bson::Int32(x), Bson::Int64(y)) => {
            let x = *x as i64;
            match op {
                QueryOperator::GreaterThan => x > *y,
                QueryOperator::GreaterThanOrEqual => x >= *y,
                QueryOperator::LessThan => x < *y,
                QueryOperator::LessThanOrEqual => x <= *y,
                _ => false,
            }
        }
        (Bson::Int64(x), Bson::Int32(y)) => {
            let y = *y as i64;
            match op {
                QueryOperator::GreaterThan => *x > y,
                QueryOperator::GreaterThanOrEqual => *x >= y,
                QueryOperator::LessThan => *x < y,
                QueryOperator::LessThanOrEqual => *x <= y,
                _ => false,
            }
        }
        (Bson::Double(x), Bson::Double(y)) => match op {
            QueryOperator::GreaterThan => x > y,
            QueryOperator::GreaterThanOrEqual => x >= y,
            QueryOperator::LessThan => x < y,
            QueryOperator::LessThanOrEqual => x <= y,
            _ => false,
        },
        (Bson::String(x), Bson::String(y)) => match op {
            QueryOperator::GreaterThan => x > y,
            QueryOperator::GreaterThanOrEqual => x >= y,
            QueryOperator::LessThan => x < y,
            QueryOperator::LessThanOrEqual => x <= y,
            _ => false,
        },
        (Bson::Array(_), _) | (_, Bson::Array(_)) => {
            return Err("Comparison operators not supported for arrays.".to_string());
        }
        (Bson::Null, _) | (_, Bson::Null) => false,
        _ => return Err("Incompatible types for comparison.".to_string()),
    };
    Ok(result)
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

    let mut result = serde_json::Map::new();
    for selector in selectors {
        match selector {
            FieldSelector::Field(name) => {
                if !collection.schema().fields.contains_key(name) {
                    return Err(format!("Unknown field '{}' in selector.", name));
                }
                if let Some(value) = doc.get(name) {
                    result.insert(
                        name.clone(),
                        serde_json::to_value(value).map_err(|e| e.to_string())?,
                    );
                }
            }
            FieldSelector::AllFields => {
                for (key, value) in doc {
                    result.insert(
                        key.to_string(),
                        serde_json::to_value(value).map_err(|e| e.to_string())?,
                    );
                }
            }
            FieldSelector::AllFieldsRecursive => {
                for (key, value) in doc {
                    result.insert(
                        key.to_string(),
                        resolve_field(key, value, collection, database, depth)?,
                    );
                }
            }
            FieldSelector::SubDocument {
                field_name,
                content,
            } => {
                result.insert(
                    field_name.clone(),
                    resolve_subdocument(field_name, doc, content, collection, database, depth)?,
                );
            }
        }
    }
    Ok(JsonValue::Object(result))
}

/// Resolves a field value, recursively resolving references up to depth 3.
///
/// ## Arguments
///
/// * `field_name` - The field name.
/// * `value` - The BSON value.
/// * `collection` - Source collection.
/// * `database` - Database for references.
/// * `depth` - Current depth.
///
/// ## Returns
///
/// Returns the resolved JSON value.
fn resolve_field(
    field_name: &str,
    value: &Bson,
    collection: &Collection,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    if depth >= 3 {
        return serde_json::to_value(value).map_err(|e| e.to_string());
    }

    let field_def = match collection.schema().fields.get(field_name) {
        Some(def) => def,
        None => return serde_json::to_value(value).map_err(|e| e.to_string()),
    };

    resolve_value_by_type(value, &field_def.field_type, database, depth)
}

/// Recursively resolves a BSON value according to its field type.
///
/// ## Arguments
///
/// * `value` - The BSON value to resolve.
/// * `field_type` - The expected field type.
/// * `database` - Database for reference lookups.
/// * `depth` - Current recursion depth.
///
/// ## Returns
///
/// Returns the resolved JSON value.
fn resolve_value_by_type(
    value: &Bson,
    field_type: &FieldType,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    match field_type {
        FieldType::Reference(ref_col) => match value {
            Bson::String(ref_id) => {
                if let Some(ref_doc) = resolve_reference(ref_id, ref_col, database)
                    && let Some(ref_collection) = database.get_collection(ref_col)
                {
                    return select_fields(
                        &ref_doc.data,
                        &[FieldSelector::AllFieldsRecursive],
                        ref_collection,
                        database,
                        depth + 1,
                    );
                }
                Ok(JsonValue::Null)
            }
            Bson::Null => Ok(JsonValue::Null),
            _ => serde_json::to_value(value).map_err(|e| e.to_string()),
        },
        FieldType::Nullable(inner) => match value {
            Bson::Null => Ok(JsonValue::Null),
            _ => resolve_value_by_type(value, inner, database, depth),
        },
        FieldType::Array(inner) => match value {
            Bson::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    results.push(resolve_value_by_type(item, inner, database, depth)?);
                }
                Ok(JsonValue::Array(results))
            }
            Bson::Null => Ok(JsonValue::Null),
            _ => serde_json::to_value(value).map_err(|e| e.to_string()),
        },
        _ => serde_json::to_value(value).map_err(|e| e.to_string()),
    }
}

/// Resolves a subdocument selector with nested conditions and selectors.
///
/// ## Arguments
///
/// * `field_name` - The reference field name.
/// * `doc` - The parent document.
/// * `content` - The parsed subdocument content.
/// * `collection` - Source collection.
/// * `database` - Database for references.
/// * `depth` - Current depth.
///
/// ## Returns
///
/// Returns the resolved JSON value or null if not found or conditions don't match.
/// For array of references, returns an array of resolved subdocuments.
fn resolve_subdocument(
    field_name: &str,
    doc: &BsonDocument,
    content: &ParsedDocContent,
    collection: &Collection,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    let field_def = collection
        .schema()
        .fields
        .get(field_name)
        .ok_or_else(|| format!("Unknown field '{}'.", field_name))?;

    let field_value = doc.get(field_name).cloned().unwrap_or(Bson::Null);

    resolve_subdoc_by_type(
        &field_value,
        &field_def.field_type,
        field_name,
        content,
        database,
        depth,
    )
}

/// Recursively resolves a BSON value for subdocument selection according to its field type.
///
/// ## Arguments
///
/// * `value` - The BSON value to resolve.
/// * `field_type` - The expected field type.
/// * `field_name` - The field name for error messages.
/// * `content` - The subdocument content with conditions and selectors.
/// * `database` - Database for reference lookups.
/// * `depth` - Current recursion depth.
///
/// ## Returns
///
/// Returns the resolved JSON value.
fn resolve_subdoc_by_type(
    value: &Bson,
    field_type: &FieldType,
    field_name: &str,
    content: &ParsedDocContent,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    match field_type {
        FieldType::Reference(ref_col) => match value {
            Bson::String(ref_id) => {
                resolve_single_reference(ref_id, ref_col, content, database, depth)
            }
            Bson::Null => Ok(JsonValue::Null),
            _ => Ok(JsonValue::Null),
        },
        FieldType::Nullable(inner) => match value {
            Bson::Null => Ok(JsonValue::Null),
            _ => resolve_subdoc_by_type(value, inner, field_name, content, database, depth),
        },
        FieldType::Array(inner) => match value {
            Bson::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    results.push(resolve_subdoc_by_type(
                        item, inner, field_name, content, database, depth,
                    )?);
                }
                Ok(JsonValue::Array(results))
            }
            Bson::Null => Ok(JsonValue::Array(vec![])),
            _ => Ok(JsonValue::Array(vec![])),
        },
        _ => Err(format!("Field '{}' is not a reference type.", field_name)),
    }
}

/// Resolves a single reference ID to a subdocument.
///
/// ## Arguments
///
/// * `ref_id` - The reference ID string.
/// * `ref_col` - The referenced collection name.
/// * `content` - The parsed subdocument content with conditions and selectors.
/// * `database` - The database.
/// * `depth` - Current recursion depth.
///
/// ## Returns
///
/// Returns the resolved JSON value or null if not found or conditions don't match.
fn resolve_single_reference(
    ref_id: &str,
    ref_col: &str,
    content: &ParsedDocContent,
    database: &Database,
    depth: u8,
) -> Result<JsonValue, String> {
    let ref_doc = match resolve_reference(ref_id, ref_col, database) {
        Some(d) => d,
        None => return Ok(JsonValue::Null),
    };

    let ref_collection = database
        .get_collection(ref_col)
        .ok_or_else(|| format!("Referenced collection '{}' not found.", ref_col))?;

    for condition in &content.conditions {
        if !evaluate_condition(&ref_doc.data, condition, ref_collection.schema())? {
            return Ok(JsonValue::Null);
        }
    }

    select_fields(
        &ref_doc.data,
        &content.selectors,
        ref_collection,
        database,
        depth + 1,
    )
}

/// Resolves a reference to fetch the referenced document.
///
/// ## Arguments
///
/// * `ref_value` - The reference ID as a string.
/// * `ref_collection_name` - The target collection name.
/// * `database` - The database.
///
/// ## Returns
///
/// Returns the referenced [`Document`] if found, or [`None`] if not found or conversion fails.
fn resolve_reference(
    ref_value: &str,
    ref_collection_name: &str,
    database: &Database,
) -> Option<Document> {
    let collection = database.get_collection(ref_collection_name)?;
    let id_field_def = collection.schema().fields.get(collection.id_field_name())?;

    let doc_id = match &id_field_def.field_type {
        FieldType::IdString => DocId::from(ref_value.to_string()),
        FieldType::IdInt => DocId::from(ref_value.parse::<u64>().ok()?),
        _ => return None,
    };

    collection.get_document(doc_id)
}
