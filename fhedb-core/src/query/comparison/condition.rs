//! # Condition Evaluation
//!
//! Provides condition evaluation for document queries.

use bson::{Bson, Document as BsonDocument};
use fhedb_types::{FieldCondition, FieldType, QueryOperator, Schema};

use super::compare::compare_bson;
use crate::query::value::parse_bson_value;

/// Evaluates a single condition against a document.
///
/// ## Arguments
///
/// * `doc` - The BSON document to evaluate.
/// * `condition` - The condition to check.
/// * `schema` - The collection schema for field validation.
///
/// ## Returns
///
/// Returns [`Ok`]\([`bool`]) indicating whether the condition matches,
/// or [`Err`]\([`String`]) if the field is unknown.
pub fn evaluate_condition(
    doc: &BsonDocument,
    condition: &FieldCondition,
    schema: &Schema,
) -> Result<bool, String> {
    let field_def = schema
        .fields
        .get(&condition.field_name)
        .ok_or_else(|| format!("Unknown field '{}'.", condition.field_name))?;

    let parse_type = get_parse_type(&field_def.field_type, &condition.operator);
    let condition_value = parse_bson_value(&condition.value, parse_type)?;

    match doc.get(&condition.field_name) {
        None => Ok(false),
        Some(Bson::Null) => Ok(match condition.operator {
            QueryOperator::Equal => condition_value == Bson::Null,
            QueryOperator::NotEqual => condition_value != Bson::Null,
            _ => false,
        }),
        Some(doc_val) => match &condition.operator {
            QueryOperator::Equal => Ok(doc_val == &condition_value),
            QueryOperator::NotEqual => Ok(doc_val != &condition_value),
            QueryOperator::GreaterThan
            | QueryOperator::GreaterThanOrEqual
            | QueryOperator::LessThan
            | QueryOperator::LessThanOrEqual => {
                compare_bson(doc_val, &condition_value, &condition.operator)
            }
            QueryOperator::Similar => Ok(match (doc_val, &condition_value) {
                (Bson::String(s), Bson::String(p)) => s.contains(p.as_str()),
                (Bson::Array(arr), _) => arr.contains(&condition_value),
                _ => false,
            }),
        },
    }
}

/// Determines the parse type for the condition value.
///
/// For Similar operator on arrays, returns the element type instead of the array type.
///
/// ## Arguments
///
/// * `field_type` - The field's declared type.
/// * `operator` - The query operator.
///
/// ## Returns
///
/// Returns the type to use for parsing the condition value.
fn get_parse_type<'a>(field_type: &'a FieldType, operator: &QueryOperator) -> &'a FieldType {
    if *operator == QueryOperator::Similar {
        if let FieldType::Array(inner) = field_type {
            return inner.as_ref();
        }
    }
    field_type
}
