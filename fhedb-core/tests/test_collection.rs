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
    let collection = Collection::new("users", schema.clone());
    assert_eq!(collection.name, "users");
    assert_eq!(collection.schema.fields.len(), 3);
}

#[test]
fn test_has_field() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema);
    assert!(collection.has_field("id"));
    assert!(collection.has_field("name"));
    assert!(!collection.has_field("email"));
}

#[test]
fn test_validate_document_valid() {
    let schema = make_test_schema();
    let collection = Collection::new("users", schema);
    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64
    };
    assert!(collection.validate_document(&doc).is_ok());
}
