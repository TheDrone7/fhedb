use bson::doc;
use fhedb_core::prelude::*;
use tempfile::tempdir;

use super::super::common::{make_int_schema, make_string_schema};

#[test]
fn test_collection_construction() {
    let schema = make_string_schema();
    let schema2 = make_int_schema();
    let temp_dir1 = tempdir().unwrap();
    let temp_dir2 = tempdir().unwrap();
    let collection = Collection::new("users", schema.clone(), temp_dir1.path()).unwrap();
    let collection2 = Collection::new("users2", schema2.clone(), temp_dir2.path()).unwrap();
    assert_eq!(collection.name, "users");
    assert_eq!(collection2.name, "users2");
}

#[test]
fn test_has_field() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    assert!(collection.has_field("id"));
    assert!(collection.has_field("name"));
    assert!(!collection.has_field("email"));
}

#[test]
fn test_validate_document_valid() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64
    };
    assert!(collection.validate_document(&doc).is_ok());
}

#[test]
fn test_get_documents_empty() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    let documents = collection.get_documents();
    assert_eq!(documents.len(), 0);
}

#[test]
fn test_id_type_enforcement() {
    // Test string collection rejects integer IDs
    let string_schema = make_string_schema();
    let temp_dir1 = tempdir().unwrap();
    let mut string_collection =
        Collection::new("string_users", string_schema, temp_dir1.path()).unwrap();

    let doc_with_int_id = doc! { "id": 42i64, "name": "Alice", "age": 30i64 };
    let result = string_collection.add_document(doc_with_int_id);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Field 'id': Expected ID as string"))
    );

    // Test integer collection rejects string IDs
    let int_schema = make_int_schema();
    let temp_dir2 = tempdir().unwrap();
    let mut int_collection = Collection::new("int_users", int_schema, temp_dir2.path()).unwrap();

    let doc_with_string_id = doc! { "id": "user-123", "name": "Bob", "age": 25i64 };
    let result = int_collection.add_document(doc_with_string_id);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Field 'id': Expected ID as integer"))
    );
}
