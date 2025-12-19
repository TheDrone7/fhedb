use bson::Bson;
use fhedb_core::prelude::compare_bson;
use fhedb_types::QueryOperator;

#[test]
fn int_gt_true() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(true));
}

#[test]
fn int_gt_false() {
    let a = Bson::Int64(3);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn int_gt_equal_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn int_gte_true() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(3);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn int_gte_equal_true() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn int_gte_false() {
    let a = Bson::Int64(3);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn int_lt_true() {
    let a = Bson::Int64(3);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn int_lt_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn int_lt_equal_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn int_lte_true() {
    let a = Bson::Int64(3);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn int_lte_equal_true() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn int_lte_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(3);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn float_gt_true() {
    let a = Bson::Double(5.5);
    let b = Bson::Double(3.3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(true));
}

#[test]
fn float_gt_false() {
    let a = Bson::Double(3.3);
    let b = Bson::Double(5.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn float_gt_equal_false() {
    let a = Bson::Double(5.0);
    let b = Bson::Double(5.0);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn float_gte_true() {
    let a = Bson::Double(5.5);
    let b = Bson::Double(3.3);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn float_gte_equal_true() {
    let a = Bson::Double(5.0);
    let b = Bson::Double(5.0);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn float_gte_false() {
    let a = Bson::Double(3.3);
    let b = Bson::Double(5.5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn float_lt_true() {
    let a = Bson::Double(3.3);
    let b = Bson::Double(5.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn float_lt_false() {
    let a = Bson::Double(5.5);
    let b = Bson::Double(3.3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn float_lt_equal_false() {
    let a = Bson::Double(5.0);
    let b = Bson::Double(5.0);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn float_lte_true() {
    let a = Bson::Double(3.3);
    let b = Bson::Double(5.5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn float_lte_equal_true() {
    let a = Bson::Double(5.0);
    let b = Bson::Double(5.0);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn float_lte_false() {
    let a = Bson::Double(5.5);
    let b = Bson::Double(3.3);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn mixed_int_gt_float_true() {
    let a = Bson::Int64(5);
    let b = Bson::Double(3.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(true));
}

#[test]
fn mixed_int_gt_float_false() {
    let a = Bson::Int64(3);
    let b = Bson::Double(5.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn mixed_float_gt_int_true() {
    let a = Bson::Double(5.5);
    let b = Bson::Int64(3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(true));
}

#[test]
fn mixed_float_gt_int_false() {
    let a = Bson::Double(3.5);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn mixed_int_gte_float_true() {
    let a = Bson::Int64(5);
    let b = Bson::Double(5.0);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn mixed_int_gte_float_false() {
    let a = Bson::Int64(3);
    let b = Bson::Double(5.0);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn mixed_float_gte_int_true() {
    let a = Bson::Double(5.0);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn mixed_float_gte_int_false() {
    let a = Bson::Double(3.5);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn mixed_int_lt_float_true() {
    let a = Bson::Int64(3);
    let b = Bson::Double(5.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn mixed_int_lt_float_false() {
    let a = Bson::Int64(5);
    let b = Bson::Double(3.5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn mixed_float_lt_int_true() {
    let a = Bson::Double(3.5);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn mixed_float_lt_int_false() {
    let a = Bson::Double(5.5);
    let b = Bson::Int64(3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn mixed_int_lte_float_true() {
    let a = Bson::Int64(5);
    let b = Bson::Double(5.0);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn mixed_int_lte_float_false() {
    let a = Bson::Int64(5);
    let b = Bson::Double(3.5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn mixed_float_lte_int_true() {
    let a = Bson::Double(5.0);
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn mixed_float_lte_int_false() {
    let a = Bson::Double(5.5);
    let b = Bson::Int64(3);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn string_gt_true() {
    let a = Bson::String("b".to_string());
    let b = Bson::String("a".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(true));
}

#[test]
fn string_gt_false() {
    let a = Bson::String("a".to_string());
    let b = Bson::String("b".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn string_gt_equal_false() {
    let a = Bson::String("abc".to_string());
    let b = Bson::String("abc".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn string_gte_true() {
    let a = Bson::String("b".to_string());
    let b = Bson::String("a".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn string_gte_equal_true() {
    let a = Bson::String("abc".to_string());
    let b = Bson::String("abc".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(true)
    );
}

#[test]
fn string_gte_false() {
    let a = Bson::String("a".to_string());
    let b = Bson::String("b".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn string_lt_true() {
    let a = Bson::String("a".to_string());
    let b = Bson::String("b".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn string_lt_false() {
    let a = Bson::String("b".to_string());
    let b = Bson::String("a".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn string_lt_equal_false() {
    let a = Bson::String("abc".to_string());
    let b = Bson::String("abc".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn string_lte_true() {
    let a = Bson::String("a".to_string());
    let b = Bson::String("b".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn string_lte_equal_true() {
    let a = Bson::String("abc".to_string());
    let b = Bson::String("abc".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(true)
    );
}

#[test]
fn string_lte_false() {
    let a = Bson::String("b".to_string());
    let b = Bson::String("a".to_string());
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn string_lexicographic_order() {
    let a = Bson::String("apple".to_string());
    let b = Bson::String("banana".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(true));
}

#[test]
fn array_left_operand_error() {
    let a = Bson::Array(vec![Bson::Int64(1)]);
    let b = Bson::Int64(2);
    let result = compare_bson(&a, &b, &QueryOperator::GreaterThan);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("array"));
}

#[test]
fn array_right_operand_error() {
    let a = Bson::Int64(1);
    let b = Bson::Array(vec![Bson::Int64(2)]);
    let result = compare_bson(&a, &b, &QueryOperator::GreaterThan);
    assert!(result.is_err());
}

#[test]
fn both_arrays_error() {
    let a = Bson::Array(vec![Bson::Int64(1)]);
    let b = Bson::Array(vec![Bson::Int64(2)]);
    let result = compare_bson(&a, &b, &QueryOperator::GreaterThan);
    assert!(result.is_err());
}

#[test]
fn null_gt_value_false() {
    let a = Bson::Null;
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn value_gt_null_false() {
    let a = Bson::Int64(5);
    let b = Bson::Null;
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
}

#[test]
fn null_gte_value_false() {
    let a = Bson::Null;
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn value_gte_null_false() {
    let a = Bson::Int64(5);
    let b = Bson::Null;
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
}

#[test]
fn null_lt_value_false() {
    let a = Bson::Null;
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn value_lt_null_false() {
    let a = Bson::Int64(5);
    let b = Bson::Null;
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
}

#[test]
fn null_lte_value_false() {
    let a = Bson::Null;
    let b = Bson::Int64(5);
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn value_lte_null_false() {
    let a = Bson::Int64(5);
    let b = Bson::Null;
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn null_null_all_ops_false() {
    let a = Bson::Null;
    let b = Bson::Null;
    assert_eq!(compare_bson(&a, &b, &QueryOperator::GreaterThan), Ok(false));
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::GreaterThanOrEqual),
        Ok(false)
    );
    assert_eq!(compare_bson(&a, &b, &QueryOperator::LessThan), Ok(false));
    assert_eq!(
        compare_bson(&a, &b, &QueryOperator::LessThanOrEqual),
        Ok(false)
    );
}

#[test]
fn incompatible_string_vs_int_error() {
    let a = Bson::String("a".to_string());
    let b = Bson::Int64(5);
    let result = compare_bson(&a, &b, &QueryOperator::GreaterThan);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Incompatible"));
}

#[test]
fn incompatible_int_vs_string_error() {
    let a = Bson::Int64(5);
    let b = Bson::String("a".to_string());
    let result = compare_bson(&a, &b, &QueryOperator::LessThan);
    assert!(result.is_err());
}

#[test]
fn incompatible_string_vs_float_error() {
    let a = Bson::String("a".to_string());
    let b = Bson::Double(5.0);
    let result = compare_bson(&a, &b, &QueryOperator::GreaterThan);
    assert!(result.is_err());
}

#[test]
fn incompatible_float_vs_string_error() {
    let a = Bson::Double(5.0);
    let b = Bson::String("a".to_string());
    let result = compare_bson(&a, &b, &QueryOperator::LessThan);
    assert!(result.is_err());
}

#[test]
fn unsupported_equal_returns_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(5);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::Equal), Ok(false));
}

#[test]
fn unsupported_not_equal_returns_false() {
    let a = Bson::Int64(5);
    let b = Bson::Int64(3);
    assert_eq!(compare_bson(&a, &b, &QueryOperator::NotEqual), Ok(false));
}

#[test]
fn unsupported_similar_returns_false() {
    let a = Bson::String("hello".to_string());
    let b = Bson::String("hello".to_string());
    assert_eq!(compare_bson(&a, &b, &QueryOperator::Similar), Ok(false));
}
