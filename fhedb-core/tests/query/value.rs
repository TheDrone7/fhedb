use bson::Bson;
use fhedb_core::prelude::{FieldType, ValueParseable};

#[test]
fn parse_null() {
    let result = "null".parse_as_bson(&FieldType::Nullable(Box::new(FieldType::Int)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Null);
}

#[test]
fn parse_boolean_true() {
    let result = "true".parse_as_bson(&FieldType::Boolean);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Boolean(true));
}

#[test]
fn parse_boolean_false() {
    let result = "false".parse_as_bson(&FieldType::Boolean);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Boolean(false));
}

#[test]
fn parse_int() {
    let result = "42".parse_as_bson(&FieldType::Int);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Int64(42));
}

#[test]
fn parse_negative_int() {
    let result = "-42".parse_as_bson(&FieldType::Int);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Int64(-42));
}

#[test]
fn parse_float() {
    let result = "1.49".parse_as_bson(&FieldType::Float);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Double(1.49));
}

#[test]
fn parse_negative_float() {
    let result = "-1.46".parse_as_bson(&FieldType::Float);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Double(-1.46));
}

#[test]
fn parse_string_double_quotes() {
    let result = "\"hello world\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("hello world".to_string()));
}

#[test]
fn parse_string_single_quotes() {
    let result = "'hello world'".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("hello world".to_string()));
}

#[test]
fn parse_empty_string() {
    let result = "\"\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("".to_string()));
}

#[test]
fn parse_string_with_escape_sequences() {
    let result = "\"hello\\nworld\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("hello\nworld".to_string()));
}

#[test]
fn parse_empty_array() {
    let result = "[]".parse_as_bson(&FieldType::Array(Box::new(FieldType::Int)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Array(vec![]));
}

#[test]
fn parse_array_of_ints() {
    let result = "[1, 2, 3]".parse_as_bson(&FieldType::Array(Box::new(FieldType::Int)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![Bson::Int64(1), Bson::Int64(2), Bson::Int64(3)])
    );
}

#[test]
fn parse_array_of_strings() {
    let result =
        "[\"a\", \"b\", \"c\"]".parse_as_bson(&FieldType::Array(Box::new(FieldType::String)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::String("a".to_string()),
            Bson::String("b".to_string()),
            Bson::String("c".to_string())
        ])
    );
}

#[test]
fn parse_array_of_floats() {
    let result = "[1.1, 2.2, 3.3]".parse_as_bson(&FieldType::Array(Box::new(FieldType::Float)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::Double(1.1),
            Bson::Double(2.2),
            Bson::Double(3.3)
        ])
    );
}

#[test]
fn parse_array_of_booleans() {
    let result =
        "[true, false, true]".parse_as_bson(&FieldType::Array(Box::new(FieldType::Boolean)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::Boolean(true),
            Bson::Boolean(false),
            Bson::Boolean(true)
        ])
    );
}

#[test]
fn parse_nested_arrays() {
    let inner_type = FieldType::Array(Box::new(FieldType::Int));
    let result = "[[1, 2], [3, 4]]".parse_as_bson(&FieldType::Array(Box::new(inner_type)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::Array(vec![Bson::Int64(1), Bson::Int64(2)]),
            Bson::Array(vec![Bson::Int64(3), Bson::Int64(4)])
        ])
    );
}

#[test]
fn parse_array_with_escaped_quotes() {
    let result =
        "[\"hello\\\"world\"]".parse_as_bson(&FieldType::Array(Box::new(FieldType::String)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![Bson::String("hello\"world".to_string())])
    );
}

#[test]
fn parse_array_with_brackets_in_strings() {
    let result =
        "[\"[test]\", \"{value}\"]".parse_as_bson(&FieldType::Array(Box::new(FieldType::String)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::String("[test]".to_string()),
            Bson::String("{value}".to_string())
        ])
    );
}

#[test]
fn parse_reference_string() {
    let result = "\"ref-12345\"".parse_as_bson(&FieldType::Reference("other".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("ref-12345".to_string()));
}

#[test]
fn parse_array_with_escape_sequences() {
    let result = "[\"Line1\\nLine2\", \"Tab\\tSeparated\", \"Back\\\\slash\"]"
        .parse_as_bson(&FieldType::Array(Box::new(FieldType::String)));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::Array(vec![
            Bson::String("Line1\nLine2".to_string()),
            Bson::String("Tab\tSeparated".to_string()),
            Bson::String("Back\\slash".to_string())
        ])
    );
}

#[test]
fn parse_unparseable_value_error() {
    let result = "not_a_valid_value".parse_as_bson(&FieldType::Int);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot parse"));
}

#[test]
fn parse_type_mismatch_error() {
    let result = "\"hello\"".parse_as_bson(&FieldType::Int);
    assert!(result.is_err());
}

#[test]
fn parse_string_with_carriage_return() {
    let result = "\"line1\\rline2\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("line1\rline2".to_string()));
}

#[test]
fn parse_string_with_null_char() {
    let result = "\"before\\0after\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("before\0after".to_string()));
}

#[test]
fn parse_string_with_unknown_escape() {
    let result = "\"unknown\\xescape\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Bson::String("unknown\\xescape".to_string())
    );
}

#[test]
fn parse_string_with_escaped_single_quote() {
    let result = "\"it\\'s\"".parse_as_bson(&FieldType::String);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("it's".to_string()));
}

#[test]
fn parse_id_int() {
    let result = "123".parse_as_bson(&FieldType::IdInt);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Int64(123));
}

#[test]
fn parse_id_string() {
    let result = "\"user-abc\"".parse_as_bson(&FieldType::IdString);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::String("user-abc".to_string()));
}

#[test]
fn parse_nullable_with_value() {
    let result = "42".parse_as_bson(&FieldType::Nullable(Box::new(FieldType::Int)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Int64(42));
}

#[test]
fn parse_whitespace_trimmed() {
    let result = "  42  ".parse_as_bson(&FieldType::Int);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Bson::Int64(42));
}
