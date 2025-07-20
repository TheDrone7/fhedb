use bson::doc;
use fhedb_core::prelude::*;
use std::collections::HashMap;

fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::Id);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

#[test]
fn test_collection_construction() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema.clone()).unwrap();
    assert_eq!(collection.name, "users");
}

#[test]
fn test_has_field() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema).unwrap();
    assert!(collection.has_field("id"));
    assert!(collection.has_field("name"));
    assert!(!collection.has_field("email"));
}

#[test]
fn test_validate_document_valid() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema).unwrap();
    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64
    };
    assert!(collection.validate_document(&doc).is_ok());
}

#[test]
fn test_get_documents_empty() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema).unwrap();
    let documents = collection.get_documents();
    assert_eq!(documents.len(), 0);
}

#[test]
fn test_get_documents_with_data() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();
    
    // Add some documents
    let doc1 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64
    };
    let doc2 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Bob",
        "age": 25i64
    };
    
    let id1 = collection.add_document(doc1).unwrap();
    let id2 = collection.add_document(doc2).unwrap();
    
    let documents = collection.get_documents();
    assert_eq!(documents.len(), 2);
    
    // Check that both documents are present
    let doc_ids: Vec<_> = documents.iter().map(|doc| doc.id).collect();
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
    let retrieved_doc = collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 30);
    
    // Verify the id field was added to the document
    assert!(retrieved_doc.data.contains_key("id"));
    assert_eq!(retrieved_doc.data.get_str("id").unwrap(), doc_id.to_string());
}
