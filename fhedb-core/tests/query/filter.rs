use bson::doc;
use fhedb_core::prelude::{Database, FieldDefinition, FieldType, Filterable, Schema};
use fhedb_types::FieldCondition;
use std::collections::HashMap;
use tempfile::TempDir;

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

fn condition(field: &str, op: &str, value: &str) -> FieldCondition {
    use fhedb_types::QueryOperator;
    let operator = match op {
        "=" => QueryOperator::Equal,
        "!=" => QueryOperator::NotEqual,
        ">" => QueryOperator::GreaterThan,
        ">=" => QueryOperator::GreaterThanOrEqual,
        "<" => QueryOperator::LessThan,
        "<=" => QueryOperator::LessThanOrEqual,
        "==" => QueryOperator::Similar,
        _ => panic!("Unknown operator: {op}"),
    };
    FieldCondition {
        field_name: field.to_string(),
        operator,
        value: value.to_string(),
    }
}

fn setup_collection() -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());
    db.create_collection("users", test_schema()).unwrap();

    let col = db.get_collection_mut("users").unwrap();
    col.add_document(doc! { "id": 1_i64, "name": "Alice", "age": 30_i64, "active": true })
        .unwrap();
    col.add_document(doc! { "id": 2_i64, "name": "Bob", "age": 25_i64, "active": true })
        .unwrap();
    col.add_document(doc! { "id": 3_i64, "name": "Charlie", "age": 35_i64, "active": false })
        .unwrap();
    col.add_document(doc! { "id": 4_i64, "name": "Diana", "age": 28_i64, "active": true })
        .unwrap();

    (temp_dir, db)
}

#[test]
fn empty_conditions_returns_all() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let result = col.filter(&[]).unwrap();
    assert_eq!(result.len(), 4);
}

#[test]
fn single_equal_condition() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("name", "=", "\"Alice\"")];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].data.get_str("name").unwrap(), "Alice");
}

#[test]
fn single_condition_no_matches() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("name", "=", "\"Nonexistent\"")];
    let result = col.filter(&conditions).unwrap();

    assert!(result.is_empty());
}

#[test]
fn single_condition_multiple_matches() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("active", "=", "true")];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 3);
}

#[test]
fn multiple_conditions_and_logic() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![
        condition("active", "=", "true"),
        condition("age", ">", "26"),
    ];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn multiple_conditions_narrow_to_one() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![
        condition("active", "=", "true"),
        condition("age", ">=", "30"),
    ];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].data.get_str("name").unwrap(), "Alice");
}

#[test]
fn multiple_conditions_no_matches() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![
        condition("active", "=", "false"),
        condition("age", "<", "30"),
    ];
    let result = col.filter(&conditions).unwrap();

    assert!(result.is_empty());
}

#[test]
fn comparison_operators() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let lt_result = col.filter(&[condition("age", "<", "28")]).unwrap();
    assert_eq!(lt_result.len(), 1);

    let lte_result = col.filter(&[condition("age", "<=", "28")]).unwrap();
    assert_eq!(lte_result.len(), 2);

    let gt_result = col.filter(&[condition("age", ">", "30")]).unwrap();
    assert_eq!(gt_result.len(), 1);

    let gte_result = col.filter(&[condition("age", ">=", "30")]).unwrap();
    assert_eq!(gte_result.len(), 2);
}

#[test]
fn not_equal_operator() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("active", "!=", "true")];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].data.get_str("name").unwrap(), "Charlie");
}

#[test]
fn unknown_field_error() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("nonexistent_field", "=", "\"value\"")];
    let result = col.filter(&conditions);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown field"));
}

#[test]
fn parse_error_propagates() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![condition("age", "=", "not_a_number")];
    let result = col.filter(&conditions);

    assert!(result.is_err());
}

#[test]
fn empty_collection_returns_empty() {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("empty_test", temp_dir.path());
    db.create_collection("empty", test_schema()).unwrap();
    let col = db.get_collection("empty").unwrap();

    let result = col.filter(&[condition("name", "=", "\"Alice\"")]).unwrap();
    assert!(result.is_empty());
}

#[test]
fn empty_collection_empty_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("empty_test2", temp_dir.path());
    db.create_collection("empty", test_schema()).unwrap();
    let col = db.get_collection("empty").unwrap();

    let result = col.filter(&[]).unwrap();
    assert!(result.is_empty());
}

#[test]
fn three_conditions_and_logic() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![
        condition("active", "=", "true"),
        condition("age", ">=", "25"),
        condition("age", "<=", "30"),
    ];
    let result = col.filter(&conditions).unwrap();

    assert_eq!(result.len(), 3);
}

#[test]
fn first_condition_fails_short_circuits() {
    let (_temp, db) = setup_collection();
    let col = db.get_collection("users").unwrap();

    let conditions = vec![
        condition("unknown", "=", "\"value\""),
        condition("name", "=", "\"Alice\""),
    ];
    let result = col.filter(&conditions);

    assert!(result.is_err());
}
