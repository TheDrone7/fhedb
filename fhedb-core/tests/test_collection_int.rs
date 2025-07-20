use bson::doc;
use fhedb_core::prelude::*;
use std::collections::HashMap;

fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

#[test]
fn test_get_documents_with_data() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    // Add some documents
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

    let id1 = collection.add_document(doc1).unwrap();
    let id2 = collection.add_document(doc2).unwrap();

    assert_eq!(id1.to_string(), "1");
    assert_eq!(id2.to_string(), "2");

    let documents = collection.get_documents();
    assert_eq!(documents.len(), 2);

    // Check that both documents are present
    let doc_ids: Vec<_> = documents.iter().map(|doc| doc.id.clone()).collect();
    assert!(doc_ids.contains(&id1));
    assert!(doc_ids.contains(&id2));
}

#[test]
fn test_add_document_without_id_field() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    // Add a document without an id field
    let doc = doc! {
        "name": "Alice",
        "age": 30i64
    };

    // Should succeed and generate an id automatically
    let doc_id = collection.add_document(doc).unwrap();

    // Verify the document was added
    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 30);

    // Verify the id field was added to the document
    assert!(retrieved_doc.data.contains_key("id"));

    // Check the ID value - should be an integer (0 for first document)
    match retrieved_doc.data.get("id").unwrap() {
        bson::Bson::Int64(i) => {
            assert_eq!(*i, 0);
            assert_eq!(i.to_string(), doc_id.to_string());
        }
        _ => panic!("Expected integer ID"),
    }
}

#[test]
fn test_add_document_with_custom_integer_id() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    // Add a document with a custom integer ID
    let doc = doc! {
        "id": 999i64,
        "name": "Alice",
        "age": 30i64
    };

    let doc_id = collection.add_document(doc).unwrap();
    assert_eq!(doc_id.to_string(), "999");

    // Verify the document was added
    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_i64("id").unwrap(), 999);
}

#[test]
fn test_add_document_with_string_id_should_fail() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    // Try to add a document with a string ID to an integer ID collection
    let doc = doc! {
        "id": "user-123",
        "name": "Alice",
        "age": 30i64
    };

    let result = collection.add_document(doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Field 'id': Expected ID as integer"))
    );
}

#[test]
fn test_sequential_id_generation() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    // Add documents without IDs to test sequential generation
    let doc1 = doc! { "name": "Alice", "age": 30i64 };
    let doc2 = doc! { "name": "Bob", "age": 25i64 };
    let doc3 = doc! { "name": "Charlie", "age": 35i64 };

    let id1 = collection.add_document(doc1).unwrap();
    let id2 = collection.add_document(doc2).unwrap();
    let id3 = collection.add_document(doc3).unwrap();

    // Should be sequential starting from 0
    assert_eq!(id1.to_string(), "0");
    assert_eq!(id2.to_string(), "1");
    assert_eq!(id3.to_string(), "2");

    // Verify the documents were stored with correct IDs
    let retrieved1 = collection.get_document(id1).unwrap();
    let retrieved2 = collection.get_document(id2).unwrap();
    let retrieved3 = collection.get_document(id3).unwrap();

    assert_eq!(retrieved1.data.get_i64("id").unwrap(), 0);
    assert_eq!(retrieved2.data.get_i64("id").unwrap(), 1);
    assert_eq!(retrieved3.data.get_i64("id").unwrap(), 2);
}
