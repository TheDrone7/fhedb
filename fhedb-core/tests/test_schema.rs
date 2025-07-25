use bson::doc;
use fhedb_core::prelude::{FieldDefinition, FieldType, IdType, Schema};
use std::collections::HashMap;

fn make_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert(
        "int_field".to_string(),
        FieldDefinition::new(FieldType::Int),
    );
    fields.insert(
        "float_field".to_string(),
        FieldDefinition::new(FieldType::Float),
    );
    fields.insert(
        "bool_field".to_string(),
        FieldDefinition::new(FieldType::Boolean),
    );
    fields.insert(
        "string_field".to_string(),
        FieldDefinition::new(FieldType::String),
    );
    fields.insert(
        "array_field".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::Int))),
    );
    fields.insert(
        "ref_field".to_string(),
        FieldDefinition::new(FieldType::Reference("other_collection".to_string())),
    );
    fields.insert(
        "id_field".to_string(),
        FieldDefinition::new(FieldType::IdInt),
    );
    Schema { fields }
}

#[test]
fn test_valid_document() {
    let schema = make_schema();
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": 42i64,
    };
    assert!(schema.validate_document(&doc).is_ok());
}

#[test]
fn test_missing_field() {
    let schema = make_schema();
    let doc = doc! {
        "int_field": 69i64,
        // missing float_field
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": 42i64,
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
    let doc = doc! {
        "int_field": "not an int",
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": 42i64,
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
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, "not an int", 3i32],
        "ref_field": "other_collection_id",
        "id_field": 42i64,
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
fn test_invalid_id_type() {
    let schema = make_schema();
    let doc = doc! {
        "int_field": 69i64,
        "float_field": 3.14f64,
        "bool_field": true,
        "string_field": "hello world",
        "array_field": [1i32, 2i32, 3i32],
        "ref_field": "other_collection_id",
        "id_field": "not-an-integer",
    };
    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Expected ID as integer")));
}

#[test]
fn test_ensure_id_no_id_field() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    let mut schema = Schema { fields };

    let (id_field, id_type) = schema.ensure_id().unwrap();
    assert_eq!(id_field, "id");
    assert_eq!(id_type, IdType::Int);
    assert!(schema.fields.contains_key("id"));
    assert_eq!(
        schema.fields.get("id").unwrap().field_type,
        FieldType::IdInt
    );
}

#[test]
fn test_ensure_id_with_existing_id_field() {
    let mut fields = HashMap::new();
    fields.insert(
        "custom_id".to_string(),
        FieldDefinition::new(FieldType::IdString),
    );
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut schema = Schema { fields };

    let (id_field, id_type) = schema.ensure_id().unwrap();
    assert_eq!(id_field, "custom_id");
    assert_eq!(id_type, IdType::String);
    assert!(schema.fields.contains_key("custom_id"));
    assert!(!schema.fields.contains_key("id")); // Should not add default "id"
}

#[test]
fn test_ensure_id_multiple_id_fields() {
    let mut fields = HashMap::new();
    fields.insert("id1".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("id2".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut schema = Schema { fields };

    let result = schema.ensure_id();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Schema must contain at most one field with type IdString or IdInt")
    );
}

#[test]
fn test_validate_document_missing_id_field() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    let schema = Schema { fields };

    let doc = doc! {
        // Missing id field
        "name": "Alice",
        "age": 30i64
    };

    // Should pass validation since Id fields are allowed to be missing
    assert!(schema.validate_document(&doc).is_ok());
}

#[test]
fn test_validate_document_missing_other_field() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    let schema = Schema { fields };

    let doc = doc! {
        "id": "some-uuid-string",
        // Missing age field
        "name": "Alice"
    };

    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Missing field: 'age'")));
}

#[test]
fn test_validate_document_missing_id_and_other_field() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    let schema = Schema { fields };

    let doc = doc! {
        // Missing both id and age fields
        "name": "Alice"
    };

    let result = schema.validate_document(&doc);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Missing field: 'age'")));
    assert!(!errors.iter().any(|e| e.contains("Missing field: 'id'")));
}

#[test]
fn test_nullable_fields() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "nickname".to_string(),
        FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::String))),
    );
    fields.insert(
        "tags".to_string(),
        FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::Array(Box::new(
            FieldType::String,
        ))))),
    );
    let schema = Schema { fields };

    // Test with all nullable fields present
    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "nickname": "Al",
        "tags": ["tag1", "tag2"],
    };
    assert!(schema.validate_document(&doc1).is_ok());

    // Test with nullable fields as null
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "nickname": null,
        "tags": null,
    };
    assert!(schema.validate_document(&doc2).is_ok());

    // Test with nullable fields missing
    let doc3 = doc! {
        "id": 3i64,
        "name": "Charlie",
    };
    assert!(schema.validate_document(&doc3).is_ok());

    // Test with wrong type for nullable field
    let doc4 = doc! {
        "id": 4i64,
        "name": "Dave",
        "nickname": 123i64, // Wrong type
    };
    let result = schema.validate_document(&doc4);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("nickname") && e.contains("Expected string"))
    );
}

#[test]
fn test_default_values() {
    use bson::Bson;

    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    // Required field without default
    fields.insert("email".to_string(), FieldDefinition::new(FieldType::String));
    // Fields with default values
    fields.insert(
        "age".to_string(),
        FieldDefinition::with_default(FieldType::Int, Bson::Int64(18)),
    );
    fields.insert(
        "active".to_string(),
        FieldDefinition::with_default(FieldType::Boolean, Bson::Boolean(true)),
    );
    fields.insert(
        "role".to_string(),
        FieldDefinition::with_default(FieldType::String, Bson::String("user".to_string())),
    );

    let schema = Schema { fields };

    // Test with all fields present - should validate successfully
    let doc1 = doc! {
        "id": 1i64,
        "name": "Alice",
        "email": "alice@example.com",
        "age": 25i64,
        "active": false,
        "role": "admin"
    };
    assert!(schema.validate_document(&doc1).is_ok());

    // Test with only required fields - should validate successfully (missing fields have defaults)
    let doc2 = doc! {
        "id": 2i64,
        "name": "Bob",
        "email": "bob@example.com"
    };
    assert!(schema.validate_document(&doc2).is_ok());

    // Test missing required field without default - should fail validation
    let doc3 = doc! {
        "id": 3i64,
        "name": "Charlie"
        // missing email field (no default)
    };
    let result = schema.validate_document(&doc3);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Missing field: 'email'")));

    // Test apply_defaults functionality
    let mut doc4 = doc! {
        "id": 4i64,
        "name": "David",
        "email": "david@example.com"
    };

    let applied_count = schema.apply_defaults(&mut doc4);
    assert_eq!(applied_count, 3); // age, active, role should be applied
    assert_eq!(doc4.get_i64("age").unwrap(), 18);
    assert_eq!(doc4.get_bool("active").unwrap(), true);
    assert_eq!(doc4.get_str("role").unwrap(), "user");

    // Test apply_defaults when some fields already exist
    let mut doc5 = doc! {
        "id": 5i64,
        "name": "Eve",
        "email": "eve@example.com",
        "age": 30i64  // This field already exists, should not be overwritten
    };

    let applied_count = schema.apply_defaults(&mut doc5);
    assert_eq!(applied_count, 2); // only active and role should be applied
    assert_eq!(doc5.get_i64("age").unwrap(), 30); // Should not be changed
    assert_eq!(doc5.get_bool("active").unwrap(), true);
    assert_eq!(doc5.get_str("role").unwrap(), "user");
}
