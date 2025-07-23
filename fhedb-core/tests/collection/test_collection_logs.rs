use bson::doc;
use fhedb_core::prelude::*;
use std::fs;
use tempfile::tempdir;

use super::super::common::{make_complex_schema, make_int_schema};

#[test]
fn test_append_to_log_creates_file() {
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
    assert_eq!(entries[0].0.document, document);
}

#[test]
fn test_read_log_entries_empty_file() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Read from non-existent logfile
    let entries = collection.read_log_entries().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_append_and_read_entries() {
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

    // Append first entry and verify append-only behavior
    let offset1 = collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    let entries1 = collection.read_log_entries().unwrap();
    assert_eq!(entries1.len(), 1);
    assert_eq!(entries1[0].0.document, doc1);

    // First entry should start at offset 0
    assert_eq!(offset1, 0);

    // Append second entry and verify first is still there
    let offset2 = collection.append_to_log(&Operation::Update, &doc2).unwrap();
    let entries2 = collection.read_log_entries().unwrap();
    assert_eq!(entries2.len(), 2);
    assert_eq!(entries2[0].0.document, doc1);
    assert_eq!(entries2[1].0.document, doc2);

    // Second entry should start at a different offset
    assert!(offset2 > offset1);

    // Append third entry
    let offset3 = collection.append_to_log(&Operation::Delete, &doc3).unwrap();

    // Third entry should start at an even higher offset
    assert!(offset3 > offset2);

    // Read back all entries
    let entries = collection.read_log_entries().unwrap();
    assert_eq!(entries.len(), 3);

    // Verify operations
    assert_eq!(entries[0].0.operation, Operation::Insert);
    assert_eq!(entries[0].0.document, doc1);
    assert_eq!(entries[1].0.operation, Operation::Update);
    assert_eq!(entries[1].0.document, doc2);
    assert_eq!(entries[2].0.operation, Operation::Delete);
    assert_eq!(entries[2].0.document, doc3);

    // Verify timestamps are different (operations happened at different times)
    assert_ne!(entries[0].0.timestamp, entries[1].0.timestamp);
    assert_ne!(entries[1].0.timestamp, entries[2].0.timestamp);
}

#[test]
fn test_different_operation_types() {
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

    // Test all operation types
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
fn test_logfile_path() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let expected_path = collection.base_path().join("logfile.log");
    assert_eq!(collection.logfile_path(), expected_path);
}

#[test]
fn test_log_entries_with_all_field_types() {
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
fn test_multiple_collections_log_isolation() {
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

    // Append to both collections
    assert!(collection1.append_to_log(&Operation::Insert, &doc1).is_ok());
    assert!(collection2.append_to_log(&Operation::Insert, &doc2).is_ok());

    // Verify isolation
    let entries1 = collection1.read_log_entries().unwrap();
    let entries2 = collection2.read_log_entries().unwrap();

    assert_eq!(entries1.len(), 1);
    assert_eq!(entries2.len(), 1);
    assert_eq!(entries1[0].0.document, doc1);
    assert_eq!(entries2[0].0.document, doc2);

    // Verify different logfile paths
    assert_ne!(collection1.logfile_path(), collection2.logfile_path());
}

#[test]
fn test_read_log_entry_at_offset() {
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

    // Append entries and collect their offsets
    let offset1 = collection.append_to_log(&Operation::Insert, &doc1).unwrap();
    let offset2 = collection.append_to_log(&Operation::Update, &doc2).unwrap();
    let offset3 = collection.append_to_log(&Operation::Delete, &doc3).unwrap();

    // Read each entry by its specific offset
    let entry1 = collection.read_log_entry_at_offset(offset1).unwrap();
    let entry2 = collection.read_log_entry_at_offset(offset2).unwrap();
    let entry3 = collection.read_log_entry_at_offset(offset3).unwrap();

    // Verify that we get the correct entries
    assert_eq!(entry1.operation, Operation::Insert);
    assert_eq!(entry1.document, doc1);

    assert_eq!(entry2.operation, Operation::Update);
    assert_eq!(entry2.document, doc2);

    assert_eq!(entry3.operation, Operation::Delete);
    assert_eq!(entry3.document, doc3);

    // Test reading from non-existent file
    let empty_collection = Collection::new("empty", make_int_schema(), temp_dir.path()).unwrap();
    let result = empty_collection.read_log_entry_at_offset(0);
    assert!(result.is_err());

    // Test reading with invalid offset
    let invalid_offset_result = collection.read_log_entry_at_offset(99999);
    assert!(invalid_offset_result.is_err());
}

#[test]
fn test_update_document_logs_correctly() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    // Add a document first
    let original_doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64,
        "salary": 75000.0,
        "active": true
    };
    let doc_id = collection.add_document(original_doc.clone()).unwrap();

    // Update the document
    let update_doc = doc! {
        "name": "Alice Updated",
        "salary": 80000.0
    };
    let result = collection.update_document(doc_id.clone(), update_doc);
    assert!(result.is_ok());

    // Read all log entries to verify the update was logged correctly
    let log_entries = collection.read_log_entries().unwrap();
    assert_eq!(log_entries.len(), 2); // Insert + Update

    // Check the insert entry
    let (insert_entry, _) = &log_entries[0];
    assert_eq!(insert_entry.operation, Operation::Insert);
    assert_eq!(insert_entry.document.get_str("name").unwrap(), "Alice");

    // Check the update entry
    let (update_entry, _) = &log_entries[1];
    assert_eq!(update_entry.operation, Operation::Update);
    assert_eq!(
        update_entry.document.get_str("name").unwrap(),
        "Alice Updated"
    );
    assert_eq!(update_entry.document.get_f64("salary").unwrap(), 80000.0);
    // Verify unchanged fields are preserved
    assert_eq!(update_entry.document.get_i64("age").unwrap(), 30);
    assert_eq!(update_entry.document.get_bool("active").unwrap(), true);

    // Verify that retrieving the document returns the updated version
    let retrieved_doc = collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice Updated");
    assert_eq!(retrieved_doc.data.get_f64("salary").unwrap(), 80000.0);
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 30);
    assert_eq!(retrieved_doc.data.get_bool("active").unwrap(), true);
}

#[test]
fn test_collection_from_files_handles_update_operations() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();

    // Create a collection and add some documents with updates
    {
        let mut collection =
            Collection::new("test_updates", schema.clone(), temp_dir.path()).unwrap();

        // Add a document
        let doc = doc! {
            "id": 1i64,
            "name": "Bob",
            "age": 25i64,
            "salary": 60000.0,
            "active": true
        };
        let doc_id = collection.add_document(doc).unwrap();

        // Update it multiple times
        let update1 = doc! {
            "name": "Bob Smith"
        };
        collection.update_document(doc_id.clone(), update1).unwrap();

        let update2 = doc! {
            "salary": 65000.0,
            "active": false
        };
        collection.update_document(doc_id, update2).unwrap();

        // Force metadata write
        collection.write_metadata().unwrap();
    }

    // Load the collection from files
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_updates").unwrap();

    // Verify the final state is correct
    let doc_id = DocId::from_u64(1);
    let retrieved_doc = loaded_collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Bob Smith");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 25);
    assert_eq!(retrieved_doc.data.get_f64("salary").unwrap(), 65000.0);
    assert_eq!(retrieved_doc.data.get_bool("active").unwrap(), false);

    // Verify the document count is correct (should be 1, not 3)
    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 1);
}
