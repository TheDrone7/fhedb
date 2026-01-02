use bson::doc;
use fhedb_core::prelude::{Database, FieldDefinition, FieldType, ReferenceResolvable, Schema};
use std::collections::HashMap;
use tempfile::TempDir;

fn int_id_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    Schema { fields }
}

fn string_id_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    Schema { fields }
}

fn setup_int_id_collection() -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());
    db.create_collection("users", int_id_schema()).unwrap();

    let col = db.get_collection_mut("users").unwrap();
    col.add_document(doc! { "id": 1_i64, "name": "Alice" })
        .unwrap();
    col.add_document(doc! { "id": 2_i64, "name": "Bob" })
        .unwrap();
    col.add_document(doc! { "id": 3_i64, "name": "Charlie" })
        .unwrap();

    (temp_dir, db)
}

fn setup_string_id_collection() -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());
    db.create_collection("users", string_id_schema()).unwrap();

    let col = db.get_collection_mut("users").unwrap();
    col.add_document(doc! { "id": "user-1", "name": "Alice" })
        .unwrap();
    col.add_document(doc! { "id": "user-2", "name": "Bob" })
        .unwrap();

    (temp_dir, db)
}

#[test]
fn resolve_int_id_found() {
    let (_temp, db) = setup_int_id_collection();

    let result = db.resolve_reference("1", "users");
    assert!(result.is_some());
    assert_eq!(result.unwrap().data.get_str("name").unwrap(), "Alice");
}

#[test]
fn resolve_int_id_not_found() {
    let (_temp, db) = setup_int_id_collection();

    let result = db.resolve_reference("999", "users");
    assert!(result.is_none());
}

#[test]
fn resolve_string_id_found() {
    let (_temp, db) = setup_string_id_collection();

    let result = db.resolve_reference("user-2", "users");
    assert!(result.is_some());
    assert_eq!(result.unwrap().data.get_str("name").unwrap(), "Bob");
}

#[test]
fn resolve_string_id_not_found() {
    let (_temp, db) = setup_string_id_collection();

    let result = db.resolve_reference("nonexistent", "users");
    assert!(result.is_none());
}

#[test]
fn resolve_collection_not_found() {
    let (_temp, db) = setup_int_id_collection();

    let result = db.resolve_reference("1", "nonexistent_collection");
    assert!(result.is_none());
}

#[test]
fn resolve_int_id_invalid_format() {
    let (_temp, db) = setup_int_id_collection();

    let result = db.resolve_reference("not_a_number", "users");
    assert!(result.is_none());
}

#[test]
fn resolve_empty_ref_value() {
    let (_temp, db) = setup_int_id_collection();

    let result = db.resolve_reference("", "users");
    assert!(result.is_none());
}

#[test]
fn resolve_empty_database() {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::new("empty_db", temp_dir.path());

    let result = db.resolve_reference("1", "users");
    assert!(result.is_none());
}
