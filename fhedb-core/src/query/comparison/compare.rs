//! # BSON Comparison
//!
//! Provides comparison operations for BSON values.

use bson::Bson;
use fhedb_types::QueryOperator;

/// Compares two BSON values using the given operator.
///
/// Supports comparison between:
/// - Integers (Int64)
/// - Floats (Double)
/// - Mixed Int64/Double (converted to f64)
/// - Strings (lexicographic)
///
/// ## Arguments
///
/// * `a` - First value.
/// * `b` - Second value.
/// * `op` - The comparison operator (GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual).
///
/// ## Returns
///
/// Returns [`Ok`]\([`bool`]) with the comparison result, or [`Err`]\([`String`]) for:
/// - Array operands (not supported)
/// - Incompatible types (e.g., String vs Int)
pub fn compare_bson(a: &Bson, b: &Bson, op: &QueryOperator) -> Result<bool, String> {
    let result = match (a, b) {
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

/// Compares two values implementing Ord using the given operator.
///
/// ## Arguments
///
/// * `a` - First value.
/// * `b` - Second value.
/// * `op` - The comparison operator.
///
/// ## Returns
///
/// Returns the comparison result. Non-comparison operators return false.
fn compare_ord<T: PartialOrd>(a: &T, b: &T, op: &QueryOperator) -> bool {
    match op {
        QueryOperator::GreaterThan => a > b,
        QueryOperator::GreaterThanOrEqual => a >= b,
        QueryOperator::LessThan => a < b,
        QueryOperator::LessThanOrEqual => a <= b,
        _ => false,
    }
}
