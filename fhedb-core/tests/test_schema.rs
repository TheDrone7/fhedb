use bson::doc;
use fhedb_core::prelude::{FieldType, Schema};
use std::collections::HashMap;
use uuid::Uuid;

fn make_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("int_field".to_string(), FieldType::Int);
    fields.insert("float_field".to_string(), FieldType::Float);
    fields.insert("bool_field".to_string(), FieldType::Boolean);
    fields.insert("string_field".to_string(), FieldType::String);
    fields.insert(
        "array_field".to_string(),
        FieldType::Array(Box::new(FieldType::Int)),
    );
    fields.insert(
        "ref_field".to_string(),
        FieldType::Reference("other_collection".to_string()),
    );
    fields.insert("id_field".to_string(), FieldType::Id);
    Schema { fields }
}

#[test]
fn test_valid_document() {
    let schema = make_schema();
    let uuid = Uuid::new_v4().to_string();
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": uuid,
    };
    assert!(schema.validate_document(&doc).is_ok());
}

#[test]
fn test_missing_field() {
    let schema = make_schema();
    let uuid = Uuid::new_v4().to_string();
    let doc = doc! {
        "int_field": 69i64,
        // missing float_field
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": uuid,
    };
    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Missing field: 'float_field'"))
    );
}

#[test]
fn test_type_mismatch() {
    let schema = make_schema();
    let uuid = Uuid::new_v4().to_string();
    let doc = doc! {
        "int_field": "not an int",
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": uuid,
    };
    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Field 'int_field': Expected int"))
    );
}

#[test]
fn test_array_type_mismatch() {
    let schema = make_schema();
    let uuid = Uuid::new_v4().to_string();
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, "not an int", 3i32],
        "ref_field": "other_collection_id",
        "id_field": uuid,
    };
    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Array element 1: Expected int"))
    );
}

#[test]
fn test_invalid_uuid() {
    let schema = make_schema();
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": "not-a-uuid",
    };
    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Expected valid UUID string"))
    );
}
