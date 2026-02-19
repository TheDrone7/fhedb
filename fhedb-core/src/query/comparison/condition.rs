//! # Condition Evaluation
//!
//! Provides condition evaluation for document queries.

use bson::{Bson, Document as BsonDocument};
use fhedb_types::{FieldCondition, FieldType, QueryOperator, Schema};

use crate::query::{comparison::compare::BsonComparable, value::ValueParseable};

/// Trait for evaluating conditions against documents.
pub trait ConditionEvaluable {
    /// Evaluates a condition against this document.
    ///
    /// ## Arguments
    ///
    /// * `condition` - The condition to check.
    /// * `schema` - The collection schema for field validation.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`bool`]) indicating whether the condition matches,
    /// or [`Err`]\([`String`]) if the field is unknown.
    fn matches(&self, condition: &FieldCondition, schema: &Schema) -> Result<bool, String>;
}

impl ConditionEvaluable for BsonDocument {
    fn matches(&self, condition: &FieldCondition, schema: &Schema) -> Result<bool, String> {
        let field_def = schema
            .fields
            .get(&condition.field_name)
            .ok_or_else(|| format!("Unknown field '{}'.", condition.field_name))?;

        let parse_type = get_parse_type(&field_def.field_type, &condition.operator);
        let condition_value = condition.value.parse_as_bson(parse_type)?;

        match self.get(&condition.field_name) {
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
                    doc_val.compare_to(&condition_value, &condition.operator)
                }
                QueryOperator::Similar => Ok(match (doc_val, &condition_value) {
                    (Bson::String(s), Bson::String(p)) => s.contains(p.as_str()),
                    (Bson::Array(arr), _) => arr.contains(&condition_value),
                    _ => false,
                }),
            },
        }
    }
}

/// Determines the parse type for the condition value.
/// For [`QueryOperator::Similar`] on arrays, returns the element type instead.
///
/// ## Arguments
///
/// * `field_type` - The field's declared type.
/// * `operator` - The query operator.
fn get_parse_type<'a>(field_type: &'a FieldType, operator: &QueryOperator) -> &'a FieldType {
    if *operator == QueryOperator::Similar
        && let FieldType::Array(inner) = field_type
    {
        return inner.as_ref();
    }
    field_type
}
