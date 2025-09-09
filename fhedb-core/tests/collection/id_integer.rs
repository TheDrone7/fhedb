use bson::doc;
use fhedb_core::prelude::*;
use tempfile::tempdir;

use super::super::common::make_int_schema;

#[test]
fn get_documents_with_data() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

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

    let doc_ids: Vec<_> = documents.iter().map(|doc| doc.id.clone()).collect();
    assert!(doc_ids.contains(&id1));
    assert!(doc_ids.contains(&id2));
}

#[test]
fn add_document_without_id_field() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "name": "Alice",
        "age": 30i64
    };

    let doc_id = collection.add_document(doc).unwrap();

    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 30);
    assert!(retrieved_doc.data.contains_key("id"));

    match retrieved_doc.data.get("id").unwrap() {
        bson::Bson::Int64(i) => {
            assert_eq!(*i, 0);
            assert_eq!(i.to_string(), doc_id.to_string());
        }
        _ => panic!("Expected integer ID"),
    }
}

#[test]
fn add_document_with_custom_id() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 999i64,
        "name": "Alice",
        "age": 30i64
    };

    let doc_id = collection.add_document(doc).unwrap();
    assert_eq!(doc_id.to_string(), "999");

    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_i64("id").unwrap(), 999);
}

#[test]
fn add_document_with_string_id() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

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
fn sequential_id_generation() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1 = doc! { "name": "Alice", "age": 30i64 };
    let doc2 = doc! { "name": "Bob", "age": 25i64 };
    let doc3 = doc! { "name": "Charlie", "age": 35i64 };

    let id1 = collection.add_document(doc1).unwrap();
    let id2 = collection.add_document(doc2).unwrap();
    let id3 = collection.add_document(doc3).unwrap();

    assert_eq!(id1.to_string(), "0");
    assert_eq!(id2.to_string(), "1");
    assert_eq!(id3.to_string(), "2");

    let retrieved1 = collection.get_document(id1).unwrap();
    let retrieved2 = collection.get_document(id2).unwrap();
    let retrieved3 = collection.get_document(id3).unwrap();

    assert_eq!(retrieved1.data.get_i64("id").unwrap(), 0);
    assert_eq!(retrieved2.data.get_i64("id").unwrap(), 1);
    assert_eq!(retrieved3.data.get_i64("id").unwrap(), 2);
}

#[test]
fn update_document_success() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update_doc = doc! {
        "name": "Alice Updated",
        "age": 31i64
    };
    let result = collection.update_document(doc_id.clone(), update_doc);
    assert!(result.is_ok());

    let updated_doc = result.unwrap();
    assert_eq!(updated_doc.data.get_str("name").unwrap(), "Alice Updated");
    assert_eq!(updated_doc.data.get_i64("age").unwrap(), 31);
    assert_eq!(updated_doc.data.get_i64("id").unwrap(), 1);

    let retrieved_doc = collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice Updated");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 31);
}

#[test]
fn update_document_partial_update() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update_doc = doc! {
        "name": "Alice Smith"
    };
    let result = collection.update_document(doc_id.clone(), update_doc);
    assert!(result.is_ok());

    let updated_doc = result.unwrap();
    assert_eq!(updated_doc.data.get_str("name").unwrap(), "Alice Smith");
    assert_eq!(updated_doc.data.get_i64("age").unwrap(), 30);
    assert_eq!(updated_doc.data.get_i64("id").unwrap(), 1);
}

#[test]
fn update_document_nonexistent() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let non_existent_id = DocId::from_u64(999);
    let update_doc = doc! {
        "name": "Alice"
    };
    let result = collection.update_document(non_existent_id, update_doc);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Document with ID")));
    assert!(errors.iter().any(|e| e.contains("not found")));
}

#[test]
fn update_document_cannot_update_id() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update_doc = doc! {
        "id": 2i64,
        "name": "Alice Updated"
    };
    let result = collection.update_document(doc_id, update_doc);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Cannot update ID field")));
}

#[test]
fn update_document_schema_validation() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 1i64,
        "name": "Alice",
        "age": 30i64
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update_doc = doc! {
        "age": "thirty"
    };
    let result = collection.update_document(doc_id, update_doc);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Field 'age'")));
}
