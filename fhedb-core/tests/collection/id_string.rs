use bson::doc;
use fhedb_core::prelude::*;
use tempfile::tempdir;
use uuid::Uuid;

use super::super::common::make_string_schema;

#[test]
fn get_documents_with_data() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid1 = Uuid::new_v4().to_string();
    let uuid2 = Uuid::new_v4().to_string();

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

    let doc_ids: Vec<_> = documents.iter().map(|doc| doc.id.clone()).collect();
    assert!(doc_ids.contains(&id1));
    assert!(doc_ids.contains(&id2));
}

#[test]
fn add_document_without_id_field() {
    let schema = make_string_schema();
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
        bson::Bson::String(s) => {
            assert_eq!(s, &doc_id.to_string());
            assert!(Uuid::parse_str(s).is_ok());
        }
        _ => panic!("Expected string ID"),
    }
}

#[test]
fn add_document_with_custom_id() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": "custom-user-123",
        "name": "Alice",
        "age": 30i64
    };

    let doc_id = collection.add_document(doc).unwrap();
    assert_eq!(doc_id.to_string(), "custom-user-123");

    let retrieved_doc = collection.get_document(doc_id.clone()).unwrap();
    assert_eq!(retrieved_doc.data.get_str("id").unwrap(), "custom-user-123");
}

#[test]
fn add_document_with_integer_id() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc = doc! {
        "id": 42i64,
        "name": "Alice",
        "age": 30i64
    };

    let result = collection.add_document(doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Field 'id': Expected ID as string"))
    );
}

#[test]
fn update_document_success() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid = Uuid::new_v4().to_string();

    let doc = doc! {
        "id": &uuid,
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
    assert_eq!(updated_doc.data.get_str("id").unwrap(), uuid);

    let retrieved_doc = collection.get_document(doc_id).unwrap();
    assert_eq!(retrieved_doc.data.get_str("name").unwrap(), "Alice Updated");
    assert_eq!(retrieved_doc.data.get_i64("age").unwrap(), 31);
}

#[test]
fn update_document_partial_update() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid = Uuid::new_v4().to_string();

    let doc = doc! {
        "id": &uuid,
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
    assert_eq!(updated_doc.data.get_str("id").unwrap(), uuid);
}

#[test]
fn update_document_nonexistent() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let non_existent_id = DocId::from_string("non-existent-uuid".to_string());
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
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid = Uuid::new_v4().to_string();

    let doc = doc! {
        "id": &uuid,
        "name": "Alice",
        "age": 30i64
    };
    let doc_id = collection.add_document(doc).unwrap();

    let update_doc = doc! {
        "id": "different-uuid",
        "name": "Alice Updated"
    };
    let result = collection.update_document(doc_id, update_doc);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Cannot update ID field")));
}

#[test]
fn update_document_schema_validation() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid = Uuid::new_v4().to_string();

    let doc = doc! {
        "id": &uuid,
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

#[test]
fn add_document_duplicate_id() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let uuid = Uuid::new_v4().to_string();
    let doc1 = doc! { "id": &uuid, "name": "Alice", "age": 30i64 };
    let doc2 = doc! { "id": &uuid, "name": "Bob", "age": 25i64 };

    collection.add_document(doc1).unwrap();
    let result = collection.add_document(doc2);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("already exists")));

    let documents = collection.get_documents();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].data.get_str("name").unwrap(), "Alice");
}
