use bson::doc;
use fhedb_core::file::collection::CollectionFileOps;
use fhedb_core::prelude::*;
use tempfile::tempdir;

use super::super::common::make_string_schema;

#[test]
fn from_files_empty_collection() {
    let temp_dir = tempdir().unwrap();
    let schema = make_string_schema();

    let original_collection = Collection::new("test_collection", schema, temp_dir.path()).unwrap();
    original_collection.write_metadata().unwrap();
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 0);
    assert_eq!(loaded_collection.document_indices().len(), 0);
    assert_eq!(loaded_collection.id_field_name(), "id");
}

#[test]
fn from_files_with_documents() {
    let temp_dir = tempdir().unwrap();
    let schema = make_string_schema();

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

    original_collection.write_metadata().unwrap();
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 3);

    let retrieved_doc1 = loaded_collection.get_document(id1).unwrap();
    let retrieved_doc2 = loaded_collection.get_document(id2).unwrap();
    let retrieved_doc3 = loaded_collection.get_document(id3).unwrap();

    assert_eq!(retrieved_doc1.data.get_str("name").unwrap(), "Alice");
    assert_eq!(retrieved_doc2.data.get_str("name").unwrap(), "Bob");
    assert_eq!(retrieved_doc3.data.get_str("name").unwrap(), "Charlie");

    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 3);
}

#[test]
fn from_files_with_deleted_documents() {
    let temp_dir = tempdir().unwrap();
    let schema = make_string_schema();

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

    original_collection.remove_document(id2.clone());
    original_collection.write_metadata().unwrap();
    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();

    assert_eq!(loaded_collection.name, "test_collection");
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 2);

    assert!(loaded_collection.get_document(id1).is_some());
    assert!(loaded_collection.get_document(id2).is_none());
    assert!(loaded_collection.get_document(id3).is_some());

    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 2);
}

#[test]
fn from_files_nonexistent_collection() {
    let temp_dir = tempdir().unwrap();

    let result = Collection::from_files(temp_dir.path(), "nonexistent_collection");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn from_files_missing_document_id() {
    let temp_dir = tempdir().unwrap();
    let schema = make_string_schema();

    let mut original_collection =
        Collection::new("test_collection", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": "user1",
        "name": "Alice",
        "age": 30,
        "active": true
    };
    original_collection.add_document(doc1).unwrap();

    original_collection.write_metadata().unwrap();

    let logfile_path = original_collection.logfile_path();
    let corrupt_entry = doc! {
        "timestamp": "2024-01-01T00:00:00Z",
        "operation": "INSERT",
        "document": {
            "name": "Bob",
            "age": 25,
            "active": false

        }
    };

    let bson_bytes = bson::to_vec(&corrupt_entry).unwrap();
    std::fs::write(&logfile_path, bson_bytes).unwrap();

    let result = Collection::from_files(temp_dir.path(), "test_collection");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("Could not extract document ID"));
}

#[test]
fn from_files_complex_operations() {
    let temp_dir = tempdir().unwrap();
    let schema = make_string_schema();

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

    let id1 = original_collection.add_document(doc1).unwrap();
    let id2 = original_collection.add_document(doc2).unwrap();
    original_collection.remove_document(id1.clone());

    let doc3 = doc! {
        "id": "user3",
        "name": "Charlie",
        "age": 35,
        "active": true
    };
    let id3 = original_collection.add_document(doc3).unwrap();
    original_collection.write_metadata().unwrap();

    let loaded_collection = Collection::from_files(temp_dir.path(), "test_collection").unwrap();
    assert_eq!(loaded_collection.inserts(), 3);
    assert_eq!(loaded_collection.document_indices().len(), 2);

    assert!(loaded_collection.get_document(id1).is_none());
    assert!(loaded_collection.get_document(id2).is_some());
    assert!(loaded_collection.get_document(id3).is_some());

    let all_docs = loaded_collection.get_documents();
    assert_eq!(all_docs.len(), 2);
}
