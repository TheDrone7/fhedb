use bson::doc;
use fhedb_core::prelude::*;
use std::fs;
use tempfile::tempdir;

use super::super::common::{make_complex_schema, make_int_schema};

#[test]
fn append_to_log_creates_file() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let document = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };

    assert!(
        collection
            .append_to_log(&Operation::Insert, &document)
            .is_ok()
    );

    let logfile_path = collection.logfile_path();
    assert!(logfile_path.exists());
    assert!(logfile_path.is_file());

    let file_size = fs::metadata(&logfile_path).unwrap().len();
    assert!(file_size > 0);

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0.document, document);
}

#[test]
fn read_log_entries_empty_file() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let entries = collection.read_log_entries().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn append_and_read_entries() {
    let schema = make_int_schema();
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

    let offset1 = collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    let entries1 = collection.read_log_entries().unwrap();
    assert_eq!(entries1.len(), 1);
    assert_eq!(entries1[0].0.document, doc1);
    assert_eq!(offset1, 0);

    let offset2 = collection.append_to_log(&Operation::Update, &doc2).unwrap();
    let entries2 = collection.read_log_entries().unwrap();
    assert_eq!(entries2.len(), 2);
    assert_eq!(entries2[0].0.document, doc1);
    assert_eq!(entries2[1].0.document, doc2);

    assert!(offset2 > offset1);

    let offset3 = collection.append_to_log(&Operation::Delete, &doc3).unwrap();
    assert!(offset3 > offset2);

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].0.operation, Operation::Insert);
    assert_eq!(entries[0].0.document, doc1);
    assert_eq!(entries[1].0.operation, Operation::Update);
    assert_eq!(entries[1].0.document, doc2);
    assert_eq!(entries[2].0.operation, Operation::Delete);
    assert_eq!(entries[2].0.document, doc3);

    assert_ne!(entries[0].0.timestamp, entries[1].0.timestamp);
    assert_ne!(entries[1].0.timestamp, entries[2].0.timestamp);
}

#[test]
fn different_operation_types() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let document = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };

    let operations = vec![Operation::Insert, Operation::Update, Operation::Delete];

    for operation in &operations {
        assert!(collection.append_to_log(operation, &document).is_ok());
    }

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 3);

    for (i, expected_op) in operations.iter().enumerate() {
        assert_eq!(entries[i].0.operation, *expected_op);
    }
}

#[test]
fn logfile_path() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let expected_path = collection.base_path().join("logfile.log");
    assert_eq!(collection.logfile_path(), expected_path);
}

#[test]
fn log_entries_with_all_field_types() {
    let schema = make_complex_schema();

    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("employees", schema, temp_dir.path()).unwrap();

    let doc_with_all_types = doc! {
        "id": "emp_001".to_string(),
        "name": "John Doe",
        "age": 30i64,
        "salary": 85000.0,
        "active": true,
        "scores": [95.5, 88.0, 92.5, 97.0],
        "tags": ["senior", "mentor"],
        "nested_numbers": [[1, 2, 3], [4, 5, 6]],
        "department": "engineering",
        "nickname": "JD"
    };

    assert!(
        collection
            .append_to_log(&Operation::Insert, &doc_with_all_types)
            .is_ok()
    );

    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0.document, doc_with_all_types);
}

#[test]
fn multiple_collections_log_isolation() {
    let schema1 = make_int_schema();
    let schema2 = make_int_schema();
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

    assert!(collection1.append_to_log(&Operation::Insert, &doc1).is_ok());
    assert!(collection2.append_to_log(&Operation::Insert, &doc2).is_ok());

    let entries1 = collection1.read_log_entries().unwrap();
    let entries2 = collection2.read_log_entries().unwrap();

    assert_eq!(entries1.len(), 1);
    assert_eq!(entries2.len(), 1);
    assert_eq!(entries1[0].0.document, doc1);
    assert_eq!(entries2[0].0.document, doc2);

    assert_ne!(collection1.logfile_path(), collection2.logfile_path());
}

#[test]
fn read_log_entry_at_offset() {
    let schema = make_int_schema();
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

    let offset1 = collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    let offset2 = collection.append_to_log(&Operation::Update, &doc2).unwrap();
    let offset3 = collection.append_to_log(&Operation::Delete, &doc3).unwrap();

    let entry1 = collection.read_log_entry_at_offset(offset1).unwrap();
    let entry2 = collection.read_log_entry_at_offset(offset2).unwrap();
    let entry3 = collection.read_log_entry_at_offset(offset3).unwrap();

    assert_eq!(entry1.operation, Operation::Insert);
    assert_eq!(entry1.document, doc1);

    assert_eq!(entry2.operation, Operation::Update);
    assert_eq!(entry2.document, doc2);

    assert_eq!(entry3.operation, Operation::Delete);
    assert_eq!(entry3.document, doc3);

    let empty_collection = Collection::new("empty", make_int_schema(), temp_dir.path()).unwrap();
    let result = empty_collection.read_log_entry_at_offset(0);
    assert!(result.is_err());

    let invalid_offset_result = collection.read_log_entry_at_offset(99999);
    assert!(invalid_offset_result.is_err());
}

#[test]
fn update_document_logs_correctly() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let original_doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };
    let doc_id = collection.add_document(original_doc.clone()).unwrap();

    let update_doc = doc! {
        "name": "Alice Updated",
        "salary": 80000.0
    };
    let result = collection.update_document(doc_id.clone(), update_doc);
    assert!(result.is_ok());

    let log_entries = collection.read_log_entries().unwrap();
    assert_eq!(log_entries.len(), 2);

    let (insert_entry, _) = &log_entries[0];
    assert_eq!(insert_entry.operation, Operation::Insert);
    assert_eq!(insert_entry.document.get_str("name").unwrap(), "Alice");

    let (update_entry, _) = &log_entries[1];
    assert_eq!(update_entry.operation, Operation::Update);
    assert_eq!(
        update_entry.document.get_str("name").unwrap(),
        "Alice Updated"
    );
    assert_eq!(update_entry.document.get_f64("salary").unwrap(), 80000.0);
    assert_eq!(update_entry.document.get_i64("age").unwrap(), 30);
    assert!(update_entry.document.get_bool("active").unwrap());

    let retrieved_doc = collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice Updated");
    assert_eq!(retrieved_doc.data.get_f64("salary").unwrap(), 80000.0);
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 30);
    assert!(retrieved_doc.data.get_bool("active").unwrap());
}

#[test]
fn collection_from_files_handles_update_operations() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();

    let mut collection = Collection::new("test_updates", schema.clone(), temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 1i64,
        "name": "Bob",
        "age": 25i64,
        "salary": 60000.0,
        "active": true
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update1 = doc! {
        "name": "Bob Smith"
    };
    collection.update_document(doc_id.clone(), update1).unwrap();

    let update2 = doc! {
        "salary": 65000.0,
        "active": false
    };
    collection.update_document(doc_id, update2).unwrap();

    collection.write_metadata().unwrap();

    let loaded_collection = Collection::from_files(temp_dir.path(), "test_updates").unwrap();

    let doc_id = DocId::from_u64(1);
    let retrieved_doc = loaded_collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Bob Smith");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 25);
    assert_eq!(retrieved_doc.data.get_f64("salary").unwrap(), 65000.0);
    assert!(!retrieved_doc.data.get_bool("active").unwrap());

    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 1);
}
