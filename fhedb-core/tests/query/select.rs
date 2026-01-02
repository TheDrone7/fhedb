use bson::{Bson, doc};
use fhedb_core::prelude::{FieldDefinition, FieldSelectable, FieldType, Schema};
use fhedb_types::FieldSelector;
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

fn schema_with_reference() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "manager".to_string(),
        FieldDefinition::new(FieldType::Reference("users".to_string())),
    );
    Schema { fields }
}

fn empty_content() -> fhedb_types::ParsedDocContent {
    fhedb_types::ParsedDocContent {
        assignments: HashMap::new(),
        conditions: vec![],
        selectors: vec![],
    }
}

#[test]
fn empty_selectors_returns_empty() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let result = doc.select_fields(&[], &test_schema()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn select_single_field() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let selectors = vec![FieldSelector::Field("name".to_string())];
    let result = doc.select_fields(&selectors, &test_schema()).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result.get("name").unwrap(),
        &Bson::String("Alice".to_string())
    );
    assert!(result.get("id").is_none());
    assert!(result.get("age").is_none());
    assert!(result.get("active").is_none());
}

#[test]
fn select_multiple_fields() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let selectors = vec![
        FieldSelector::Field("name".to_string()),
        FieldSelector::Field("age".to_string()),
    ];
    let result = doc.select_fields(&selectors, &test_schema()).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get("name").unwrap(),
        &Bson::String("Alice".to_string())
    );
    assert_eq!(result.get("age").unwrap(), &Bson::Int64(30));
    assert!(result.get("id").is_none());
    assert!(result.get("active").is_none());
}

#[test]
fn select_all_fields() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let selectors = vec![FieldSelector::AllFields];
    let result = doc.select_fields(&selectors, &test_schema()).unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(result.get("id").unwrap(), &Bson::Int64(1));
    assert_eq!(
        result.get("name").unwrap(),
        &Bson::String("Alice".to_string())
    );
    assert_eq!(result.get("age").unwrap(), &Bson::Int64(30));
    assert_eq!(result.get("active").unwrap(), &Bson::Boolean(true));
}

#[test]
fn select_unknown_field_error() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let selectors = vec![FieldSelector::Field("unknown".to_string())];
    let result = doc.select_fields(&selectors, &test_schema());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown field"));
}

#[test]
fn select_with_null_value() {
    let doc = doc! { "id": 1_i64, "name": Bson::Null, "age": 30_i64, "active": true };
    let selectors = vec![FieldSelector::Field("name".to_string())];
    let result = doc.select_fields(&selectors, &test_schema()).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result.get("name").unwrap(), &Bson::Null);
}

#[test]
fn duplicate_selectors() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true };
    let selectors = vec![
        FieldSelector::Field("name".to_string()),
        FieldSelector::Field("name".to_string()),
    ];
    let result = doc.select_fields(&selectors, &test_schema()).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result.get("name").unwrap(),
        &Bson::String("Alice".to_string())
    );
}

#[test]
fn subdocument_with_value() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "manager": "ref-123" };
    let selectors = vec![FieldSelector::SubDocument {
        field_name: "manager".to_string(),
        content: empty_content(),
    }];
    let result = doc
        .select_fields(&selectors, &schema_with_reference())
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result.get("manager").unwrap(),
        &Bson::String("ref-123".to_string())
    );
}

#[test]
fn subdocument_defaults_to_null() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "manager": Bson::Null };
    let selectors = vec![FieldSelector::SubDocument {
        field_name: "manager".to_string(),
        content: empty_content(),
    }];
    let result = doc
        .select_fields(&selectors, &schema_with_reference())
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result.get("manager").unwrap(), &Bson::Null);
}

#[test]
fn subdocument_unknown_field_error() {
    let doc = doc! { "id": 1_i64, "name": "Alice", "manager": "ref-123" };
    let selectors = vec![FieldSelector::SubDocument {
        field_name: "unknown".to_string(),
        content: empty_content(),
    }];
    let result = doc.select_fields(&selectors, &schema_with_reference());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown field"));
}
