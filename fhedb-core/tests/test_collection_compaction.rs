use bson::doc;
use fhedb_core::file::collection::CollectionFileOps;
use fhedb_core::file::types::Operation;
use fhedb_core::prelude::*;
use std::collections::HashMap;
use tempfile::tempdir;

/// Creates a test schema with integer ID type
fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

#[test]
fn test_compact_logfile_empty() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Compact empty logfile
    assert!(collection.compact_logfile().is_ok());

    // Verify logfile still doesn't exist (was empty)
    let logfile_path = collection.logfile_path();
    assert!(!logfile_path.exists());
}

#[test]
fn test_compact_logfile_inserts_only() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "age": 25i64
    };

    // Add some documents to the log
    collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();

    // Compact the logfile
    assert!(collection.compact_logfile().is_ok());

    // Read back the compacted entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 2);

    // Verify both documents are present as INSERT operations
    let doc_ids: Vec<_> = entries
        .iter()
        .map(|e| e.document.get_i64("id").unwrap())
        .collect();
    assert!(doc_ids.contains(&1));
    assert!(doc_ids.contains(&2));

    for entry in entries {
        assert_eq!(entry.operation, Operation::Insert);
    }
}

#[test]
fn test_compact_logfile_with_updates() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1_original = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc1_updated = doc! {
        "id": 1i64,
        "name": "Alice Smith",
        "age": 31i64
    };

    // Add document, then update it
    collection
        .append_to_log(&Operation::Insert, &doc1_original)
        .unwrap();
    collection
        .append_to_log(&Operation::Update, &doc1_updated)
        .unwrap();

    // Compact the logfile
    assert!(collection.compact_logfile().is_ok());

    // Read back the compacted entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.operation, Operation::Insert);
    assert_eq!(entry.document.get_str("name").unwrap(), "Alice Smith");
    assert_eq!(entry.document.get_i64("age").unwrap(), 31);
}

#[test]
fn test_compact_logfile_with_deletes() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "age": 25i64
    };

    // Add two documents, then delete one
    collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();
    collection.append_to_log(&Operation::Delete, &doc1).unwrap();

    // Compact the logfile
    assert!(collection.compact_logfile().is_ok());

    // Read back the compacted entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.operation, Operation::Insert);
    assert_eq!(entry.document.get_i64("id").unwrap(), 2);
    assert_eq!(entry.document.get_str("name").unwrap(), "Bob");
}

#[test]
fn test_compact_logfile_complex_sequence() {
    let schema = make_test_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1_v1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc1_v2 = doc! {
        "id": 1i64,
        "name": "Alice Smith",
        "age": 31i64
    };
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "age": 25i64
    };
    let doc3 = doc! {
        "id": 3i64,
        "name": "Charlie",
        "age": 35i64
    };

    // Complex sequence: insert, update, insert, delete, insert
    collection
        .append_to_log(&Operation::Insert, &doc1_v1)
        .unwrap();
    collection
        .append_to_log(&Operation::Update, &doc1_v2)
        .unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();
    collection.append_to_log(&Operation::Delete, &doc2).unwrap();
    collection.append_to_log(&Operation::Insert, &doc3).unwrap();

    // Compact the logfile
    assert!(collection.compact_logfile().is_ok());

    // Read back the compacted entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 2);

    // Should have doc1 (updated version) and doc3
    let doc_ids: Vec<_> = entries
        .iter()
        .map(|e| e.document.get_i64("id").unwrap())
        .collect();
    assert!(doc_ids.contains(&1));
    assert!(doc_ids.contains(&3));
    assert!(!doc_ids.contains(&2)); // Should be deleted

    // Verify doc1 has the updated values
    let doc1_entry = entries
        .iter()
        .find(|e| e.document.get_i64("id").unwrap() == 1)
        .unwrap();
    assert_eq!(doc1_entry.document.get_str("name").unwrap(), "Alice Smith");
    assert_eq!(doc1_entry.document.get_i64("age").unwrap(), 31);
}
