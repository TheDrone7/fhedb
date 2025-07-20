use bson::doc;
use fhedb_core::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

fn make_test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

#[test]
fn test_get_documents_with_data() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();

    let uuid1 = Uuid::new_v4().to_string();
    let uuid2 = Uuid::new_v4().to_string();
    
    // Add some documents
    let doc1 = doc! {
        "id": &uuid1,
        "name": "Alice",
        "age": 30i64
    };
    let doc2 = doc! {
        "id": &uuid2,
        "name": "Bob",
        "age": 25i64
    };
    
    let id1 = collection.add_document(doc1).unwrap();
    let id2 = collection.add_document(doc2).unwrap();

    assert_eq!(id1.to_string(), uuid1);
    assert_eq!(id2.to_string(), uuid2);
    
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
    
    // Check the ID value - should be a string (UUID)
    match retrieved_doc.data.get("id").unwrap() {
        bson::Bson::String(s) => {
            assert_eq!(s, &doc_id.to_string());
            // Verify it's a valid UUID
            assert!(Uuid::parse_str(s).is_ok());
        },
        _ => panic!("Expected string ID"),
    }
}

#[test]
fn test_add_document_with_custom_string_id() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();
    
    // Add a document with a custom string ID (not UUID)
    let doc = doc! {
        "id": "custom-user-123",
        "name": "Alice",
        "age": 30i64
    };
    
    let doc_id = collection.add_document(doc).unwrap();
    assert_eq!(doc_id.to_string(), "custom-user-123");
    
    // Verify the document was added
    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_str("id").unwrap(), "custom-user-123");
}

#[test]
fn test_add_document_with_integer_id_should_fail() {
    let schema = make_test_schema();
    let mut collection = Collection::new("users", schema).unwrap();
    
    // Try to add a document with an integer ID to a string ID collection
    let doc = doc! {
        "id": 42i64,
        "name": "Alice",
        "age": 30i64
    };
    
    let result = collection.add_document(doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Field 'id': Expected ID as string")));
} 