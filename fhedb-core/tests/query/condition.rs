use bson::{Bson, doc};
use fhedb_core::prelude::{FieldDefinition, FieldType, Schema, evaluate_condition};
use fhedb_types::FieldCondition;
use std::collections::HashMap;

fn test_schema() -> Schema {
    let mut fields = HashMap::new();
    fields.insert("id".to_string(), FieldDefinition::new(FieldType::IdInt));
    fields.insert("age".to_string(), FieldDefinition::new(FieldType::Int));
    fields.insert("score".to_string(), FieldDefinition::new(FieldType::Float));
    fields.insert("name".to_string(), FieldDefinition::new(FieldType::String));
    fields.insert(
        "active".to_string(),
        FieldDefinition::new(FieldType::Boolean),
    );
    fields.insert(
        "nullable_val".to_string(),
        FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::Int))),
    );
    fields.insert(
        "tags".to_string(),
        FieldDefinition::new(FieldType::Array(Box::new(FieldType::String))),
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

#[test]
fn equal_int_match() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn equal_int_no_match() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn equal_float_match() {
    let doc = doc! { "score": 75.5 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", "=", "75.5"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn equal_float_no_match() {
    let doc = doc! { "score": 80.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", "=", "75.5"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn equal_string_match() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "=", "\"alice\""), &test_schema()),
        Ok(true)
    );
}

#[test]
fn equal_string_no_match() {
    let doc = doc! { "name": "bob" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "=", "\"alice\""), &test_schema()),
        Ok(false)
    );
}

#[test]
fn equal_bool_match() {
    let doc = doc! { "active": true };
    assert_eq!(
        evaluate_condition(&doc, &condition("active", "=", "true"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn equal_bool_no_match() {
    let doc = doc! { "active": false };
    assert_eq!(
        evaluate_condition(&doc, &condition("active", "=", "true"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn not_equal_int_match() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "!=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn not_equal_int_no_match() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "!=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn not_equal_string_match() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "!=", "\"bob\""), &test_schema()),
        Ok(true)
    );
}

#[test]
fn not_equal_string_no_match() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "!=", "\"alice\""), &test_schema()),
        Ok(false)
    );
}

#[test]
fn greater_than_int_true() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn greater_than_int_false() {
    let doc = doc! { "age": 20_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn greater_than_int_equal_false() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn greater_than_float_true() {
    let doc = doc! { "score": 75.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", ">", "50.0"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn greater_than_float_false() {
    let doc = doc! { "score": 40.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", ">", "50.0"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn greater_than_string_true() {
    let doc = doc! { "name": "bob" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", ">", "\"alice\""), &test_schema()),
        Ok(true)
    );
}

#[test]
fn greater_than_string_false() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", ">", "\"bob\""), &test_schema()),
        Ok(false)
    );
}

#[test]
fn gte_int_greater_true() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn gte_int_equal_true() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn gte_int_false() {
    let doc = doc! { "age": 20_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", ">=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn gte_float_equal_true() {
    let doc = doc! { "score": 50.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", ">=", "50.0"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn less_than_int_true() {
    let doc = doc! { "age": 20_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn less_than_int_false() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn less_than_int_equal_false() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn less_than_float_true() {
    let doc = doc! { "score": 40.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", "<", "50.0"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn lte_int_less_true() {
    let doc = doc! { "age": 20_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn lte_int_equal_true() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<=", "25"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn lte_int_false() {
    let doc = doc! { "age": 30_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "<=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn lte_float_equal_true() {
    let doc = doc! { "score": 100.0 };
    assert_eq!(
        evaluate_condition(&doc, &condition("score", "<=", "100.0"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn similar_string_contains_true() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "==", "\"li\""), &test_schema()),
        Ok(true)
    );
}

#[test]
fn similar_string_contains_false() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("name", "==", "\"bob\""), &test_schema()),
        Ok(false)
    );
}

#[test]
fn similar_array_contains_true() {
    let doc = doc! { "tags": ["rust", "go", "python"] };
    assert_eq!(
        evaluate_condition(&doc, &condition("tags", "==", "\"rust\""), &test_schema()),
        Ok(true)
    );
}

#[test]
fn similar_array_contains_false() {
    let doc = doc! { "tags": ["rust", "go"] };
    assert_eq!(
        evaluate_condition(&doc, &condition("tags", "==", "\"java\""), &test_schema()),
        Ok(false)
    );
}

#[test]
fn null_equals_null() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(
            &doc,
            &condition("nullable_val", "=", "null"),
            &test_schema()
        ),
        Ok(true)
    );
}

#[test]
fn null_not_equals_value() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", "!=", "5"), &test_schema()),
        Ok(true)
    );
}

#[test]
fn value_not_equals_null() {
    let doc = doc! { "nullable_val": 5_i64 };
    assert_eq!(
        evaluate_condition(
            &doc,
            &condition("nullable_val", "!=", "null"),
            &test_schema()
        ),
        Ok(true)
    );
}

#[test]
fn null_gt_returns_false() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", ">", "5"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn null_lt_returns_false() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", "<", "5"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn missing_field_returns_false() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn missing_field_not_equal_returns_false() {
    let doc = doc! { "name": "alice" };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "!=", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn unknown_field_error() {
    let doc = doc! { "age": 25_i64 };
    let result = evaluate_condition(&doc, &condition("unknown", "=", "5"), &test_schema());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown field"));
}

#[test]
fn similar_on_int_returns_false() {
    let doc = doc! { "age": 25_i64 };
    assert_eq!(
        evaluate_condition(&doc, &condition("age", "==", "25"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn similar_on_bool_returns_false() {
    let doc = doc! { "active": true };
    assert_eq!(
        evaluate_condition(&doc, &condition("active", "==", "true"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn parse_value_error() {
    let doc = doc! { "age": 25_i64 };
    let result = evaluate_condition(&doc, &condition("age", "=", "not_a_number"), &test_schema());
    assert!(result.is_err());
}

#[test]
fn null_gte_returns_false() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", ">=", "5"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn null_lte_returns_false() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", "<=", "5"), &test_schema()),
        Ok(false)
    );
}

#[test]
fn null_similar_returns_false() {
    let doc = doc! { "nullable_val": Bson::Null };
    assert_eq!(
        evaluate_condition(&doc, &condition("nullable_val", "==", "5"), &test_schema()),
        Ok(false)
    );
}
