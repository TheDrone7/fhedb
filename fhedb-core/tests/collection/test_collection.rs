use bson::doc;
use fhedb_core::prelude::*;
use tempfile::tempdir;

use super::super::common::{make_int_schema, make_schema_with_defaults, make_string_schema};

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

#[test]
fn test_collection_with_default_values() {
    let schema = make_schema_with_defaults();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "name": "Alice",
        "email": "alice@example.com"
    };

    let doc_id = collection.add_document(doc).unwrap();
    let retrieved_doc = collection.get_document(doc_id).unwrap();

    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice");
    assert_eq!(
        retrieved_doc.data.get_str("email").unwrap(),
        "alice@example.com"
    );
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 18);
    assert_eq!(retrieved_doc.data.get_bool("active").unwrap(), true);
    assert_eq!(retrieved_doc.data.get_str("role").unwrap(), "user");
    assert_eq!(retrieved_doc.data.get_f64("score").unwrap(), 0.0);

    let doc2 = doc! {
        "name": "Bob",
        "email": "bob@example.com",
        "age": 25i64,
        "role": "admin"
    };

    let doc_id2 = collection.add_document(doc2).unwrap();
    let retrieved_doc2 = collection.get_document(doc_id2).unwrap();

    assert_eq!(retrieved_doc2.data.get_str("name").unwrap(), "Bob");
    assert_eq!(
        retrieved_doc2.data.get_str("email").unwrap(),
        "bob@example.com"
    );
    assert_eq!(retrieved_doc2.data.get_i64("age").unwrap(), 25);
    assert_eq!(retrieved_doc2.data.get_bool("active").unwrap(), true);
    assert_eq!(retrieved_doc2.data.get_str("role").unwrap(), "admin");
    assert_eq!(retrieved_doc2.data.get_f64("score").unwrap(), 0.0);
}
