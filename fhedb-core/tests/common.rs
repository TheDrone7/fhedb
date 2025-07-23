#![allow(dead_code)]
use fhedb_core::prelude::*;
use std::collections::HashMap;

/// Creates a simple test schema with just string ID and title
pub fn make_simple_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("title".to_string(), FieldType::String);
    Schema { fields }
}

/// Creates a test schema with string ID type (basic fields: id, name, age)
pub fn make_string_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

/// Creates a test schema with integer ID type (basic fields: id, name, age)
pub fn make_int_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdInt);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    Schema { fields }
}

/// Creates a complex test schema with all field types including nested arrays
pub fn make_complex_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldType::IdString);
    fields.insert("name".to_string(), FieldType::String);
    fields.insert("age".to_string(), FieldType::Int);
    fields.insert("salary".to_string(), FieldType::Float);
    fields.insert("active".to_string(), FieldType::Boolean);
    fields.insert(
        "scores".to_string(),
        FieldType::Array(Box::new(FieldType::Float)),
    );
    fields.insert(
        "tags".to_string(),
        FieldType::Array(Box::new(FieldType::String)),
    );
    fields.insert(
        "nested_numbers".to_string(),
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int)))),
    );
    fields.insert(
        "department".to_string(),
        FieldType::Reference("departments".to_string()),
    );
    fields.insert(
        "nickname".to_string(),
        FieldType::Nullable(Box::new(FieldType::String)),
    );
    Schema { fields }
}
