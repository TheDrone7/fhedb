use bson::doc;
use fhedb_core::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

/// Creates a test schema with string ID type
fn make_test_schema_string() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("active".to_string(), FieldType::Boolean);
    fields.insert(
        "scores".to_string(),
        FieldType::Array(Box::new(FieldType::Float)),
    );
    fields.insert(
        "department".to_string(),
        FieldType::Reference("departments".to_string()),
    );
    Schema { fields }
}

/// Creates a test schema with integer ID type
fn make_test_schema_int() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("salary".to_string(), FieldType::Float);
    Schema { fields }
}

#[test]
fn test_write_metadata_creates_file() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Write metadata
    assert!(collection.write_metadata().is_ok());

    // Check that metadata file exists
    let metadata_path = collection.metadata_path();
    assert!(metadata_path.exists());
    assert!(metadata_path.is_file());
}

#[test]
fn test_read_metadata_round_trip_string_id() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let original_collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Write metadata
    assert!(original_collection.write_metadata().is_ok());

    // Read metadata back
    let read_collection = Collection::read_metadata(
        original_collection.base_path().clone().parent().unwrap(),
        &original_collection.name,
    )
    .unwrap();

    // Verify all fields match
    assert_eq!(read_collection.name, original_collection.name);
    assert_eq!(
        read_collection.schema().fields,
        original_collection.schema().fields
    );
    assert_eq!(read_collection.inserts(), original_collection.inserts());
    assert_eq!(read_collection.base_path(), original_collection.base_path());
}

#[test]
fn test_read_metadata_round_trip_int_id() {
    let schema = make_test_schema_int();
    let temp_dir = tempdir().unwrap();
    let original_collection = Collection::new("employees", schema, temp_dir.path()).unwrap();

    // Write metadata
    assert!(original_collection.write_metadata().is_ok());

    // Read metadata back
    let read_collection = Collection::read_metadata(
        original_collection.base_path().clone().parent().unwrap(),
        &original_collection.name,
    )
    .unwrap();

    // Verify all fields match
    assert_eq!(read_collection.name, original_collection.name);
    assert_eq!(
        read_collection.schema().fields,
        original_collection.schema().fields
    );
    assert_eq!(read_collection.inserts(), original_collection.inserts());
    assert_eq!(read_collection.base_path(), original_collection.base_path());
}

#[test]
fn test_read_metadata_with_inserts() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Add some documents to increment the inserts counter
    let doc1 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64,
        "active": true,
        "scores": [85.5, 92.0, 78.5],
        "department": "engineering"
    };
    let doc2 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Bob",
        "age": 25i64,
        "active": false,
        "scores": [90.0, 88.5, 95.0],
        "department": "marketing"
    };

    collection.add_document(doc1).unwrap();
    collection.add_document(doc2).unwrap();

    // Write metadata
    assert!(collection.write_metadata().is_ok());

    // Read metadata back
    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();

    // Verify inserts count is preserved
    assert_eq!(read_collection.inserts(), 2);
    assert_eq!(read_collection.inserts(), collection.inserts());
}

#[test]
fn test_read_metadata_file_not_found() {
    let temp_dir = tempdir().unwrap();

    // Try to read metadata from non-existent collection
    let result = Collection::read_metadata(temp_dir.path(), "nonexistent_collection");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(error.to_string().contains("Metadata file not found"));
}

#[test]
fn test_metadata_paths() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Test metadata path
    let expected_metadata_path = collection.base_path().join("metadata.bin");
    assert_eq!(collection.metadata_path(), expected_metadata_path);

    // Test logfile path
    let expected_logfile_path = collection.base_path().join("logfile.log");
    assert_eq!(collection.logfile_path(), expected_logfile_path);
}

#[test]
fn test_write_metadata_overwrites_existing() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Write metadata initially
    assert!(collection.write_metadata().is_ok());

    // Add a document to change the inserts count
    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64,
        "active": true,
        "scores": [85.5, 92.0],
        "department": "engineering"
    };
    collection.add_document(doc).unwrap();

    // Write metadata again
    assert!(collection.write_metadata().is_ok());

    // Read metadata back and verify it was updated
    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();
    assert_eq!(read_collection.inserts(), 1);
    assert_eq!(read_collection.inserts(), collection.inserts());
}

#[test]
fn test_metadata_preserves_complex_schema() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("salary".to_string(), FieldType::Float);
    fields.insert("active".to_string(), FieldType::Boolean);
    fields.insert(
        "tags".to_string(),
        FieldType::Array(Box::new(FieldType::String)),
    );
    fields.insert(
        "scores".to_string(),
        FieldType::Array(Box::new(FieldType::Float)),
    );
    fields.insert(
        "nested_scores".to_string(),
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int)))),
    );
    fields.insert(
        "department".to_string(),
        FieldType::Reference("departments".to_string()),
    );
    fields.insert(
        "manager".to_string(),
        FieldType::Reference("employees".to_string()),
    );

    let schema = Schema { fields };
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("complex_users", schema, temp_dir.path()).unwrap();

    // Write metadata
    assert!(collection.write_metadata().is_ok());

    // Read metadata back
    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();

    // Verify complex schema is preserved exactly
    assert_eq!(
        read_collection.schema().fields.len(),
        collection.schema().fields.len()
    );

    for (key, value) in collection.schema().fields.iter() {
        assert_eq!(read_collection.schema().fields.get(key), Some(value));
    }
}

#[test]
fn test_metadata_file_size() {
    let schema = make_test_schema_string();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Write metadata
    assert!(collection.write_metadata().is_ok());

    // Check that metadata file has content
    let metadata_path = collection.metadata_path();
    let metadata = fs::read(&metadata_path).unwrap();
    assert!(!metadata.is_empty());

    // Verify it's valid BSON by trying to parse it
    let parsed: bson::Document = bson::from_slice(&metadata).unwrap();
    assert!(parsed.contains_key("name"));
    assert!(parsed.contains_key("inserts"));
    assert!(parsed.contains_key("schema"));
}

#[test]
fn test_multiple_collections_metadata_isolation() {
    let schema1 = make_test_schema_string();
    let schema2 = make_test_schema_int();
    let temp_dir = tempdir().unwrap();

    let collection1 = Collection::new("users", schema1, temp_dir.path()).unwrap();
    let collection2 = Collection::new("employees", schema2, temp_dir.path()).unwrap();

    // Write metadata for both collections
    assert!(collection1.write_metadata().is_ok());
    assert!(collection2.write_metadata().is_ok());

    // Verify both metadata files exist
    assert!(collection1.metadata_path().exists());
    assert!(collection2.metadata_path().exists());

    // Read both back and verify they're different
    let read_collection1 = Collection::read_metadata(
        collection1.base_path().clone().parent().unwrap(),
        &collection1.name,
    )
    .unwrap();
    let read_collection2 = Collection::read_metadata(
        collection2.base_path().clone().parent().unwrap(),
        &collection2.name,
    )
    .unwrap();

    assert_eq!(read_collection1.name, "users");
    assert_eq!(read_collection2.name, "employees");
    assert_ne!(
        read_collection1.schema().fields,
        read_collection2.schema().fields
    );
}
