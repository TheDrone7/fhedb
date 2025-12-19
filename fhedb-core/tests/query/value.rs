use bson::Bson;
use fhedb_core::prelude::{FieldType, parse_bson_value};

#[test]
fn parse_int() {
    let result = parse_bson_value("18", &FieldType::Int).unwrap();
    assert_eq!(result, Bson::Int64(18));
}

#[test]
fn parse_negative_int() {
    let result = parse_bson_value("-42", &FieldType::Int).unwrap();
    assert_eq!(result, Bson::Int64(-42));
}

#[test]
fn parse_float() {
    let result = parse_bson_value("3.14159", &FieldType::Float).unwrap();
    assert_eq!(result, Bson::Double(3.14159));
}

#[test]
fn parse_negative_float() {
    let result = parse_bson_value("-1.5", &FieldType::Float).unwrap();
    assert_eq!(result, Bson::Double(-1.5));
}

#[test]
fn parse_boolean_true() {
    let result = parse_bson_value("true", &FieldType::Boolean).unwrap();
    assert_eq!(result, Bson::Boolean(true));
}

#[test]
fn parse_boolean_false() {
    let result = parse_bson_value("false", &FieldType::Boolean).unwrap();
    assert_eq!(result, Bson::Boolean(false));
}

#[test]
fn parse_string_double_quotes() {
    let result = parse_bson_value("\"John\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("John".to_string()));
}

#[test]
fn parse_string_single_quotes() {
    let result = parse_bson_value("'Hello World'", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("Hello World".to_string()));
}

#[test]
fn parse_empty_string() {
    let result = parse_bson_value("\"\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("".to_string()));
}

#[test]
fn parse_string_with_escape_sequences() {
    let result = parse_bson_value("\"Hello\\nWorld\\t!\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("Hello\nWorld\t!".to_string()));
}

#[test]
fn parse_null() {
    let result =
        parse_bson_value("null", &FieldType::Nullable(Box::new(FieldType::String))).unwrap();
    assert_eq!(result, Bson::Null);
}

#[test]
fn parse_reference_string() {
    let result = parse_bson_value("\"admin\"", &FieldType::Reference("users".to_string())).unwrap();
    assert_eq!(result, Bson::String("admin".to_string()));
}

#[test]
fn parse_empty_array() {
    let result = parse_bson_value("[]", &FieldType::Array(Box::new(FieldType::String))).unwrap();
    assert_eq!(result, Bson::Array(vec![]));
}

#[test]
fn parse_array_of_strings() {
    let result = parse_bson_value(
        "[\"Alice\", \"Bob\", \"Charlie\"]",
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::String("Alice".to_string()),
            Bson::String("Bob".to_string()),
            Bson::String("Charlie".to_string())
        ])
    );
}

#[test]
fn parse_array_of_ints() {
    let result =
        parse_bson_value("[1, 2, 3]", &FieldType::Array(Box::new(FieldType::Int))).unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![Bson::Int64(1), Bson::Int64(2), Bson::Int64(3)])
    );
}

#[test]
fn parse_array_of_booleans() {
    let result = parse_bson_value(
        "[true, false, true]",
        &FieldType::Array(Box::new(FieldType::Boolean)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::Boolean(true),
            Bson::Boolean(false),
            Bson::Boolean(true)
        ])
    );
}

#[test]
fn parse_array_of_floats() {
    let result = parse_bson_value(
        "[1.5, -2.7, 3.14]",
        &FieldType::Array(Box::new(FieldType::Float)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::Double(1.5),
            Bson::Double(-2.7),
            Bson::Double(3.14)
        ])
    );
}

#[test]
fn parse_nested_arrays() {
    let result = parse_bson_value(
        "[[1, 2], [3, 4]]",
        &FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int)))),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::Array(vec![Bson::Int64(1), Bson::Int64(2)]),
            Bson::Array(vec![Bson::Int64(3), Bson::Int64(4)])
        ])
    );
}

#[test]
fn parse_array_with_brackets_in_strings() {
    let result = parse_bson_value(
        "[\"[item1]\", \"data[0]\", \"array[index]\"]",
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::String("[item1]".to_string()),
            Bson::String("data[0]".to_string()),
            Bson::String("array[index]".to_string())
        ])
    );
}

#[test]
fn parse_array_with_escaped_quotes() {
    let result = parse_bson_value(
        "[\"He said \\\"Hello\\\"\", \"She said 'Hi'\"]",
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::String("He said \"Hello\"".to_string()),
            Bson::String("She said 'Hi'".to_string())
        ])
    );
}

#[test]
fn parse_array_with_escape_sequences() {
    let result = parse_bson_value(
        "[\"Line1\\nLine2\", \"Tab\\tSeparated\", \"Back\\\\slash\"]",
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::String("Line1\nLine2".to_string()),
            Bson::String("Tab\tSeparated".to_string()),
            Bson::String("Back\\slash".to_string())
        ])
    );
}

#[test]
fn parse_unparseable_value_error() {
    let result = parse_bson_value("not_a_valid_value", &FieldType::Int);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot parse"));
}

#[test]
fn parse_type_mismatch_error() {
    let result = parse_bson_value("\"hello\"", &FieldType::Int);
    assert!(result.is_err());
}

#[test]
fn parse_string_with_carriage_return() {
    let result = parse_bson_value("\"line1\\rline2\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("line1\rline2".to_string()));
}

#[test]
fn parse_string_with_null_char() {
    let result = parse_bson_value("\"before\\0after\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("before\0after".to_string()));
}

#[test]
fn parse_string_with_unknown_escape() {
    let result = parse_bson_value("\"unknown\\xescape\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("unknown\\xescape".to_string()));
}

#[test]
fn parse_string_with_escaped_single_quote() {
    let result = parse_bson_value("\"it\\'s\"", &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("it's".to_string()));
}

#[test]
fn parse_id_int() {
    let result = parse_bson_value("123", &FieldType::IdInt).unwrap();
    assert_eq!(result, Bson::Int64(123));
}

#[test]
fn parse_id_string() {
    let result = parse_bson_value("\"user-abc\"", &FieldType::IdString).unwrap();
    assert_eq!(result, Bson::String("user-abc".to_string()));
}

#[test]
fn parse_nullable_with_value() {
    let result = parse_bson_value("42", &FieldType::Nullable(Box::new(FieldType::Int))).unwrap();
    assert_eq!(result, Bson::Int64(42));
}

#[test]
fn parse_whitespace_trimmed() {
    let result = parse_bson_value("  42  ", &FieldType::Int).unwrap();
    assert_eq!(result, Bson::Int64(42));
}
