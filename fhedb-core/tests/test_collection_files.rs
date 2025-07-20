use bson::doc;
use fhedb_core::file::collection::CollectionFileOps;
use fhedb_core::prelude::*;
use std::collections::HashMap;
use tempfile::tempdir;

/// Creates a test schema with string ID type
fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("active".to_string(), FieldType::Boolean);
    Schema { fields }
}

#[test]
fn test_from_files_empty_collection() {
    let temp_dir = tempdir().unwrap();
    let schema = make_test_schema();

    // Create a new collection and save its metadata
    let original_collection = Collection::new("test_collection", schema, temp_dir.path()).unwrap();
    original_collection.write_metadata().unwrap();

    // Load the collection from files
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    // Verify basic properties
    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 0);
    assert_eq!(loaded_collection.document_indices().len(), 0);
    assert_eq!(loaded_collection.id_field_name(), "id");
}

#[test]
fn test_from_files_with_documents() {
    let temp_dir = tempdir().unwrap();
    let schema = make_test_schema();

    // Create a new collection and add some documents
    let mut original_collection =
        Collection::new("test_collection", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": "user1",
        "name": "Alice",
        "age": 30,
        "active": true
    };
    let doc2 = doc! {
        "id": "user2",
        "name": "Bob",
        "age": 25,
        "active": false
    };
    let doc3 = doc! {
        "id": "user3",
        "name": "Charlie",
        "age": 35,
        "active": true
    };

    let id1 = original_collection.add_document(doc1).unwrap();
    let id2 = original_collection.add_document(doc2).unwrap();
    let id3 = original_collection.add_document(doc3).unwrap();

    // Save metadata
    original_collection.write_metadata().unwrap();

    // Load the collection from files
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    // Verify basic properties
    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 3);

    // Verify all documents can be retrieved
    let retrieved_doc1 = loaded_collection.get_document(id1).unwrap();
    let retrieved_doc2 = loaded_collection.get_document(id2).unwrap();
    let retrieved_doc3 = loaded_collection.get_document(id3).unwrap();

    assert_eq!(retrieved_doc1.data.get_str("name").unwrap(), "Alice");
    assert_eq!(retrieved_doc2.data.get_str("name").unwrap(), "Bob");
    assert_eq!(retrieved_doc3.data.get_str("name").unwrap(), "Charlie");

    // Verify all documents are returned by get_documents
    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 3);
}

#[test]
fn test_from_files_with_deleted_documents() {
    let temp_dir = tempdir().unwrap();
    let schema = make_test_schema();

    // Create a new collection and add some documents
    let mut original_collection =
        Collection::new("test_collection", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": "user1",
        "name": "Alice",
        "age": 30,
        "active": true
    };
    let doc2 = doc! {
        "id": "user2",
        "name": "Bob",
        "age": 25,
        "active": false
    };
    let doc3 = doc! {
        "id": "user3",
        "name": "Charlie",
        "age": 35,
        "active": true
    };

    let id1 = original_collection.add_document(doc1).unwrap();
    let id2 = original_collection.add_document(doc2).unwrap();
    let id3 = original_collection.add_document(doc3).unwrap();

    // Remove one document
    original_collection.remove_document(id2.clone());

    // Save metadata
    original_collection.write_metadata().unwrap();

    // Load the collection from files
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    // Verify basic properties
    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 2); // Only 2 documents should remain

    // Verify correct documents exist
    assert!(loaded_collection.get_document(id1).is_some());
    assert!(loaded_collection.get_document(id2).is_none()); // Should be deleted
    assert!(loaded_collection.get_document(id3).is_some());

    // Verify get_documents returns only existing documents
    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 2);
}

#[test]
fn test_from_files_nonexistent_collection() {
    let temp_dir = tempdir().unwrap();

    // Try to load a collection that doesn't exist
    let result = Collection::from_files(temp_dir.path(), "nonexistent_collection");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn test_from_files_missing_document_id() {
    let temp_dir = tempdir().unwrap();
    let schema = make_test_schema();

    // Create a collection and manually corrupt the log file
    let mut original_collection =
        Collection::new("test_collection", schema, temp_dir.path()).unwrap();

    // Add a normal document first
    let doc1 = doc! {
        "id": "user1",
        "name": "Alice",
        "age": 30,
        "active": true
    };
    original_collection.add_document(doc1).unwrap();

    // Save metadata
    original_collection.write_metadata().unwrap();

    // Manually add a corrupted log entry without an ID
    let logfile_path = original_collection.logfile_path();
    let corrupt_entry = doc! {
        "timestamp": "2024-01-01T00:00:00Z",
        "operation": "INSERT",
        "document": {
            "name": "Bob",
            "age": 25,
            "active": false
            // Missing "id" field
        }
    };

    let bson_bytes = bson::to_vec(&corrupt_entry).unwrap();
    std::fs::write(&logfile_path, bson_bytes).unwrap();

    // Try to load the collection - should fail with InvalidData error
    let result = Collection::from_files(temp_dir.path(), "test_collection");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("Could not extract document ID"));
}

#[test]
fn test_from_files_complex_operations() {
    let temp_dir = tempdir().unwrap();
    let schema = make_test_schema();

    // Create a collection with complex operations: insert, update, delete
    let mut original_collection =
        Collection::new("test_collection", schema, temp_dir.path()).unwrap();

    // Insert documents
    let doc1 = doc! {
        "id": "user1",
        "name": "Alice",
        "age": 30,
        "active": true
    };
    let doc2 = doc! {
        "id": "user2",
        "name": "Bob",
        "age": 25,
        "active": false
    };

    let id1 = original_collection.add_document(doc1).unwrap();
    let id2 = original_collection.add_document(doc2).unwrap();

    // Delete one document
    original_collection.remove_document(id1.clone());

    // Add another document
    let doc3 = doc! {
        "id": "user3",
        "name": "Charlie",
        "age": 35,
        "active": true
    };
    let id3 = original_collection.add_document(doc3).unwrap();

    // Save metadata
    original_collection.write_metadata().unwrap();

    // Load the collection from files
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    // Verify the final state
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 2); // user1 deleted, user2 and user3 remain

    assert!(loaded_collection.get_document(id1).is_none()); // Deleted
    assert!(loaded_collection.get_document(id2).is_some()); // Exists
    assert!(loaded_collection.get_document(id3).is_some()); // Exists

    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 2);
}
