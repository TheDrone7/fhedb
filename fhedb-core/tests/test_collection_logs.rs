use bson::doc;
use fhedb_core::file::collection::CollectionFileOps;
use fhedb_core::file::types::Operation;
use fhedb_core::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

/// Creates a test schema with integer ID type
fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("salary".to_string(), FieldType::Float);
    fields.insert("active".to_string(), FieldType::Boolean);
    Schema { fields }
}

#[test]
fn test_append_to_log_creates_file() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let document = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };

    // Append to log
    assert!(
        collection
            .append_to_log(&Operation::Insert, &document)
            .is_ok()
    );

    // Check that logfile exists and has content
    let logfile_path = collection.logfile_path();
    assert!(logfile_path.exists());
    assert!(logfile_path.is_file());

    let file_size = fs::metadata(&logfile_path).unwrap().len();
    assert!(file_size > 0);

    // Verify persistence by reading back the entry
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].document, document);
}

#[test]
fn test_read_log_entries_empty_file() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Read from non-existent logfile
    let entries = collection.read_log_entries().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_append_and_read_entries() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "age": 25i64,
        "salary": 65000.0,
        "active": false
    };
    let doc3 = doc! {
        "id": 3i64,
        "name": "Charlie",
        "age": 35i64,
        "salary": 85000.0,
        "active": true
    };

    // Append first entry and verify append-only behavior
    assert!(collection.append_to_log(&Operation::Insert, &doc1).is_ok());
    let entries1 = collection.read_log_entries().unwrap();
    assert_eq!(entries1.len(), 1);
    assert_eq!(entries1[0].document, doc1);

    // Append second entry and verify first is still there
    assert!(collection.append_to_log(&Operation::Update, &doc2).is_ok());
    let entries2 = collection.read_log_entries().unwrap();
    assert_eq!(entries2.len(), 2);
    assert_eq!(entries2[0].document, doc1);
    assert_eq!(entries2[1].document, doc2);

    // Append third entry
    assert!(collection.append_to_log(&Operation::Delete, &doc3).is_ok());

    // Read back all entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 3);

    // Verify operations
    assert_eq!(entries[0].operation, Operation::Insert);
    assert_eq!(entries[0].document, doc1);
    assert_eq!(entries[1].operation, Operation::Update);
    assert_eq!(entries[1].document, doc2);
    assert_eq!(entries[2].operation, Operation::Delete);
    assert_eq!(entries[2].document, doc3);

    // Verify timestamps are different (operations happened at different times)
    assert_ne!(entries[0].timestamp, entries[1].timestamp);
    assert_ne!(entries[1].timestamp, entries[2].timestamp);
}

#[test]
fn test_different_operation_types() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let document = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };

    // Test all operation types
    let operations = vec![Operation::Insert, Operation::Update, Operation::Delete];

    for operation in &operations {
        assert!(collection.append_to_log(operation, &document).is_ok());
    }

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 3);

    for (i, expected_op) in operations.iter().enumerate() {
        assert_eq!(entries[i].operation, *expected_op);
    }
}

#[test]
fn test_logfile_path() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let expected_path = collection.base_path().join("logfile.log");
    assert_eq!(collection.logfile_path(), expected_path);
}

#[test]
fn test_log_entries_with_all_field_types() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("salary".to_string(), FieldType::Float);
    fields.insert("active".to_string(), FieldType::Boolean);
    fields.insert(
        "scores".to_string(),
        FieldType::Array(Box::new(FieldType::Float)),
    );
    fields.insert(
        "department".to_string(),
        FieldType::Reference("departments".to_string()),
    );
    let schema = Schema { fields };

    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("employees", schema, temp_dir.path()).unwrap();

    let doc_with_all_types = doc! {
        "id": 1i64,
        "name": "John Doe",
        "age": 30i64,
        "salary": 85000.0,
        "active": true,
        "scores": [95.5, 88.0, 92.5, 97.0],
        "department": "engineering"
    };

    assert!(
        collection
            .append_to_log(&Operation::Insert, &doc_with_all_types)
            .is_ok()
    );

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].document, doc_with_all_types);
}

#[test]
fn test_multiple_collections_log_isolation() {
    let schema1 = make_test_schema();
    let schema2 = make_test_schema();
    let temp_dir = tempdir().unwrap();

    let collection1 = Collection::new("users", schema1, temp_dir.path()).unwrap();
    let collection2 = Collection::new("employees", schema2, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };
    let doc2 = doc! {
        "id": 1i64,
        "name": "Bob",
        "age": 25i64,
        "salary": 50000.0,
        "active": false
    };

    // Append to both collections
    assert!(collection1.append_to_log(&Operation::Insert, &doc1).is_ok());
    assert!(collection2.append_to_log(&Operation::Insert, &doc2).is_ok());

    // Verify isolation
    let entries1 = collection1.read_log_entries().unwrap();
    let entries2 = collection2.read_log_entries().unwrap();

    assert_eq!(entries1.len(), 1);
    assert_eq!(entries2.len(), 1);
    assert_eq!(entries1[0].document, doc1);
    assert_eq!(entries2[0].document, doc2);

    // Verify different logfile paths
    assert_ne!(collection1.logfile_path(), collection2.logfile_path());
}
