#![allow(dead_code)]
use fhedb_core::prelude::*;
use std::collections::HashMap;

/// Creates a simple test schema with just string ID and title
pub fn make_simple_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("title".to_string(), FieldDefinition::new(FieldType::String));
    Schema { fields }
}

/// Creates a test schema with string ID type (basic fields: id, name, age)
pub fn make_string_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    Schema { fields }
}

/// Creates a test schema with integer ID type (basic fields: id, name, age)
pub fn make_int_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    Schema { fields }
}

/// Creates a complex test schema with all field types including nested arrays
pub fn make_complex_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    fields.insert("salary".to_string(), FieldDefinition::new(FieldType::Float));
    fields.insert(
        "active".to_string(),
        FieldDefinition::new(FieldType::Boolean),
    );
    fields.insert(
        "scores".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::Float))),
    );
    fields.insert(
        "tags".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::String))),
    );
    fields.insert(
        "nested_numbers".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::Array(Box::new(
            FieldType::Int,
        ))))),
    );
    fields.insert(
        "department".to_string(),
        FieldDefinition::new(FieldType::Reference("departments".to_string())),
    );
    fields.insert(
        "nickname".to_string(),
        FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::String))),
    );
    Schema { fields }
}

/// Creates a test schema with default values
pub fn make_schema_with_defaults() -> Schema {
    use bson::Bson;

    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdString));
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
    fields.insert(
        "score".to_string(),
        FieldDefinition::with_default(FieldType::Float, Bson::Double(0.0)),
    );

    Schema { fields }
}
