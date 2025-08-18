use bson::doc;
use fhedb_core::prelude::*;
use tempfile::tempdir;

use super::super::common::make_int_schema;

#[test]
fn test_compact_logfile_empty() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    assert!(collection.compact_logfile().is_ok());

    let logfile_path = collection.logfile_path();
    assert!(!logfile_path.exists());
}

#[test]
fn test_compact_logfile_inserts_only() {
    let schema = make_int_schema();
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

    collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();

    assert!(collection.compact_logfile().is_ok());
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 2);

    let doc_ids: Vec<_> = entries
        .iter()
        .map(|e| e.0.document.get_i64("id").unwrap())
        .collect();
    assert!(doc_ids.contains(&1));
    assert!(doc_ids.contains(&2));

    for (entry, _) in entries {
        assert_eq!(entry.operation, Operation::Insert);
    }
}

#[test]
fn test_compact_logfile_with_updates() {
    let schema = make_int_schema();
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

    collection
        .append_to_log(&Operation::Insert, &doc1_original)
        .unwrap();
    collection
        .append_to_log(&Operation::Update, &doc1_updated)
        .unwrap();

    assert!(collection.compact_logfile().is_ok());
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.0.operation, Operation::Insert);
    assert_eq!(entry.0.document.get_str("name").unwrap(), "Alice Smith");
    assert_eq!(entry.0.document.get_i64("age").unwrap(), 31);
}

#[test]
fn test_compact_logfile_with_deletes() {
    let schema = make_int_schema();
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

    collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();
    collection.append_to_log(&Operation::Delete, &doc1).unwrap();

    assert!(collection.compact_logfile().is_ok());
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.0.operation, Operation::Insert);
    assert_eq!(entry.0.document.get_i64("id").unwrap(), 2);
    assert_eq!(entry.0.document.get_str("name").unwrap(), "Bob");
}

#[test]
fn test_compact_logfile_complex_sequence() {
    let schema = make_int_schema();
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

    collection
        .append_to_log(&Operation::Insert, &doc1_v1)
        .unwrap();
    collection
        .append_to_log(&Operation::Update, &doc1_v2)
        .unwrap();
    collection.append_to_log(&Operation::Insert, &doc2).unwrap();
    collection.append_to_log(&Operation::Delete, &doc2).unwrap();
    collection.append_to_log(&Operation::Insert, &doc3).unwrap();

    assert!(collection.compact_logfile().is_ok());
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 2);

    let doc_ids: Vec<_> = entries
        .iter()
        .map(|e| e.0.document.get_i64("id").unwrap())
        .collect();
    assert!(doc_ids.contains(&1));
    assert!(doc_ids.contains(&3));
    assert!(!doc_ids.contains(&2));

    let doc1_entry = entries
        .iter()
        .find(|e| e.0.document.get_i64("id").unwrap() == 1)
        .unwrap();
    assert_eq!(
        doc1_entry.0.document.get_str("name").unwrap(),
        "Alice Smith"
    );
    assert_eq!(doc1_entry.0.document.get_i64("age").unwrap(), 31);
}
