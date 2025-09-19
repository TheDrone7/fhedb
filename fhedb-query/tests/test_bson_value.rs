use bson::Bson;
use fhedb_core::db::schema::FieldType;
use fhedb_query::parser::utilities::parse_bson_value;

#[test]
fn parse_bson_value_int() {
    let result = parse_bson_value("18".to_string(), &FieldType::Int).unwrap();
    assert_eq!(result, Bson::Int64(18));

    let result = parse_bson_value("-42".to_string(), &FieldType::Int).unwrap();
    assert_eq!(result, Bson::Int64(-42));
}

#[test]
fn parse_bson_value_float() {
    let result = parse_bson_value("3.14159".to_string(), &FieldType::Float).unwrap();
    assert_eq!(result, Bson::Double(3.14159));

    let result = parse_bson_value("-1.5".to_string(), &FieldType::Float).unwrap();
    assert_eq!(result, Bson::Double(-1.5));
}

#[test]
fn parse_bson_value_boolean() {
    let result = parse_bson_value("true".to_string(), &FieldType::Boolean).unwrap();
    assert_eq!(result, Bson::Boolean(true));

    let result = parse_bson_value("false".to_string(), &FieldType::Boolean).unwrap();
    assert_eq!(result, Bson::Boolean(false));
}

#[test]
fn parse_bson_value_string() {
    let result = parse_bson_value("\"John\"".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("John".to_string()));

    let result = parse_bson_value("\"Software Engineer\"".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("Software Engineer".to_string()));

    let result = parse_bson_value("'Hello World'".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("Hello World".to_string()));

    let result = parse_bson_value("\"\"".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("".to_string()));

    let result = parse_bson_value("\"null\"".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("null".to_string()));

    let result = parse_bson_value("\"Hello\\nWorld\\t!\"".to_string(), &FieldType::String).unwrap();
    assert_eq!(result, Bson::String("Hello\nWorld\t!".to_string()));
}

#[test]
fn parse_bson_value_null() {
    let result = parse_bson_value(
        "null".to_string(),
        &FieldType::Nullable(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(result, Bson::Null);
}

#[test]
fn parse_bson_value_reference() {
    let result = parse_bson_value(
        "\"admin\"".to_string(),
        &FieldType::Reference("users".to_string()),
    )
    .unwrap();
    assert_eq!(result, Bson::String("admin".to_string()));

    let result = parse_bson_value(
        "\"default-company\"".to_string(),
        &FieldType::Reference("companies".to_string()),
    )
    .unwrap();
    assert_eq!(result, Bson::String("default-company".to_string()));

    let result = parse_bson_value(
        "'uncategorized'".to_string(),
        &FieldType::Reference("categories".to_string()),
    )
    .unwrap();
    assert_eq!(result, Bson::String("uncategorized".to_string()));

    let result = parse_bson_value(
        "\"data\\\\user\\tinfo\"".to_string(),
        &FieldType::Reference("paths".to_string()),
    )
    .unwrap();
    assert_eq!(result, Bson::String("data\\user\tinfo".to_string()));
}

#[test]
fn parse_bson_value_array() {
    let result = parse_bson_value(
        "[]".to_string(),
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(result, Bson::Array(vec![]));

    let result = parse_bson_value(
        "[\"Alice\", \"Bob\", \"Charlie\"]".to_string(),
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

    let result = parse_bson_value(
        "[1, 2, 3]".to_string(),
        &FieldType::Array(Box::new(FieldType::Int)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![Bson::Int64(1), Bson::Int64(2), Bson::Int64(3)])
    );

    let result = parse_bson_value(
        "[true, false, true]".to_string(),
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

    let result = parse_bson_value(
        "[1.5, 2.7, 3.14]".to_string(),
        &FieldType::Array(Box::new(FieldType::Float)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::Double(1.5),
            Bson::Double(2.7),
            Bson::Double(3.14)
        ])
    );

    let result = parse_bson_value(
        "[\"[item1]\", \"data[0]\", \"array[index]\"]".to_string(),
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

    let result = parse_bson_value(
        "[[1, 2], [3, 4]]".to_string(),
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

    let result = parse_bson_value(
        "[\"data[0]\", \"config[env]\", \"array[key]\"]".to_string(),
        &FieldType::Array(Box::new(FieldType::String)),
    )
    .unwrap();
    assert_eq!(
        result,
        Bson::Array(vec![
            Bson::String("data[0]".to_string()),
            Bson::String("config[env]".to_string()),
            Bson::String("array[key]".to_string())
        ])
    );

    let result = parse_bson_value(
        "[\"He said \\\"Hello\\\"\", \"She said 'Hi'\"]".to_string(),
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

    let result = parse_bson_value(
        "[\"Line1\\nLine2\", \"Tab\\tSeparated\", \"Back\\\\slash\"]".to_string(),
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
