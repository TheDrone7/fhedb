//! # BSON Comparison
//!
//! Provides comparison operations for BSON values.

use bson::Bson;
use fhedb_types::QueryOperator;

/// Trait for comparing BSON values.
pub trait BsonComparable {
    /// Compares this BSON value to another using the given operator.
    ///
    /// ## Arguments
    ///
    /// * `other` - The value to compare against.
    /// * `op` - The comparison operator.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`bool`]) with the comparison result, or [`Err`]\([`String`]) for
    /// incompatible types or unsupported operations.
    fn compare_to(&self, other: &Bson, op: &QueryOperator) -> Result<bool, String>;
}

impl BsonComparable for Bson {
    fn compare_to(&self, other: &Bson, op: &QueryOperator) -> Result<bool, String> {
        let result = match (self, other) {
            (Bson::Int64(x), Bson::Int64(y)) => compare_ord(x, y, op),
            (Bson::Double(x), Bson::Double(y)) => compare_ord(x, y, op),
            (Bson::Int64(x), Bson::Double(y)) => compare_ord(&(*x as f64), y, op),
            (Bson::Double(x), Bson::Int64(y)) => compare_ord(x, &(*y as f64), op),
            (Bson::String(x), Bson::String(y)) => compare_ord(x, y, op),
            (Bson::Array(_), _) | (_, Bson::Array(_)) => {
                return Err("Comparison operators not supported for arrays.".to_string());
            }
            (Bson::Null, _) | (_, Bson::Null) => false,
            _ => return Err("Incompatible types for comparison.".to_string()),
        };
        Ok(result)
    }
}

/// Compares two values implementing [`PartialOrd`] using the given operator.
/// Non-comparison operators return false.
///
/// ## Arguments
///
/// * `a` - First value.
/// * `b` - Second value.
/// * `op` - The comparison operator.
fn compare_ord<T: PartialOrd>(a: &T, b: &T, op: &QueryOperator) -> bool {
    match op {
        QueryOperator::GreaterThan => a > b,
        QueryOperator::GreaterThanOrEqual => a >= b,
        QueryOperator::LessThan => a < b,
        QueryOperator::LessThanOrEqual => a <= b,
        _ => false,
    }
}
