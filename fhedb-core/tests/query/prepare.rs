use bson::Bson;
use fhedb_core::prelude::{DocumentPreparable, FieldDefinition, FieldType, Schema};
use std::collections::HashMap;

fn test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    fields.insert(
        "active".to_string(),
        FieldDefinition::new(FieldType::Boolean),
    );
    Schema { fields }
}

fn nullable_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "nickname".to_string(),
        FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::String))),
    );
    Schema { fields }
}

fn array_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "tags".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::String))),
    );
    Schema { fields }
}

fn reference_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "manager".to_string(),
        FieldDefinition::new(FieldType::Reference("users".to_string())),
    );
    Schema { fields }
}

fn default_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut age_def = FieldDefinition::new(FieldType::Int);
    age_def.default_value = Some(Bson::Int64(0));
    fields.insert("age".to_string(), age_def);
    Schema { fields }
}

#[test]
fn all_fields_provided() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());
    assignments.insert("age".to_string(), "30".to_string());
    assignments.insert("active".to_string(), "true".to_string());

    let result = assignments.prepare_document(&test_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("name").unwrap(), "Alice");
    assert_eq!(doc.get_i64("age").unwrap(), 30);
    assert!(doc.get_bool("active").unwrap());
}

#[test]
fn missing_required_field_error() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&test_schema());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Missing required field"));
    assert!(err.contains("age") || err.contains("active"));
}

#[test]
fn nullable_defaults_to_null() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&nullable_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("name").unwrap(), "Alice");
    assert_eq!(doc.get("nickname").unwrap(), &Bson::Null);
}

#[test]
fn array_defaults_to_empty() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&array_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("name").unwrap(), "Alice");
    assert_eq!(doc.get_array("tags").unwrap(), &vec![]);
}

#[test]
fn reference_defaults_to_null() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&reference_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("name").unwrap(), "Alice");
    assert_eq!(doc.get("manager").unwrap(), &Bson::Null);
}

#[test]
fn use_schema_default() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&default_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("name").unwrap(), "Alice");
    assert_eq!(doc.get_i64("age").unwrap(), 0);
}

#[test]
fn unknown_field_error() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());
    assignments.insert("nonexistent".to_string(), "\"value\"".to_string());

    let result = assignments.prepare_document(&test_schema());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Unknown field"));
    assert!(err.contains("nonexistent"));
}

#[test]
fn parsing_error() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());
    assignments.insert("age".to_string(), "not_a_number".to_string());
    assignments.insert("active".to_string(), "true".to_string());

    let result = assignments.prepare_document(&test_schema());

    assert!(result.is_err());
}

#[test]
fn id_field_excluded() {
    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());
    assignments.insert("age".to_string(), "30".to_string());
    assignments.insert("active".to_string(), "true".to_string());

    let result = assignments.prepare_document(&test_schema());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert!(doc.get("id").is_none());
}

#[test]
fn nullable_uses_schema_default() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut nickname_def = FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::String)));
    nickname_def.default_value = Some(Bson::String("Anonymous".to_string()));
    fields.insert("nickname".to_string(), nickname_def);
    let schema = Schema { fields };

    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&schema);
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("nickname").unwrap(), "Anonymous");
}

#[test]
fn array_uses_schema_default() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut tags_def = FieldDefinition::new(FieldType::Array(Box::new(FieldType::String)));
    tags_def.default_value = Some(Bson::Array(vec![Bson::String("default".to_string())]));
    fields.insert("tags".to_string(), tags_def);
    let schema = Schema { fields };

    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&schema);
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(
        doc.get_array("tags").unwrap(),
        &vec![Bson::String("default".to_string())]
    );
}

#[test]
fn reference_uses_schema_default() {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    let mut manager_def = FieldDefinition::new(FieldType::Reference("users".to_string()));
    manager_def.default_value = Some(Bson::String("admin".to_string()));
    fields.insert("manager".to_string(), manager_def);
    let schema = Schema { fields };

    let mut assignments = HashMap::new();
    assignments.insert("name".to_string(), "\"Alice\"".to_string());

    let result = assignments.prepare_document(&schema);
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert_eq!(doc.get_str("manager").unwrap(), "admin");
}
