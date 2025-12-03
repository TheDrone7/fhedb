#[test]
fn parse_bson_value_int() {
    "18";
    "-42";
}

#[test]
fn parse_bson_value_float() {
    "3.14159";
    "-1.5";
}

#[test]
fn parse_bson_value_boolean() {
    "true";
    "false";
}

#[test]
fn parse_bson_value_string() {
    "\"John\"";
    "\"Software Engineer\"";
    "'Hello World'";
    "\"\"";
    "\"null\"";
    "\"Hello\\nWorld\\t!\"";
}

#[test]
fn parse_bson_value_null() {
    "null";
}

#[test]
fn parse_bson_value_reference() {
    "\"admin\"";
    "\"default-company\"";
    "'uncategorized'";
    "\"data\\\\user\\tinfo\"";
}

#[test]
fn parse_bson_value_array() {
    "[]";
    "[\"Alice\", \"Bob\", \"Charlie\"]";
    "[1, 2, 3]";
    "[true, false, true]";
    "[1.5, 2.7, 3.14]";
    "[\"[item1]\", \"data[0]\", \"array[index]\"]";
    "[[1, 2], [3, 4]]";
    "[\"data[0]\", \"config[env]\", \"array[key]\"]";
    "[\"He said \\\"Hello\\\"\", \"She said 'Hi'\"]";
    "[\"Line1\\nLine2\", \"Tab\\tSeparated\", \"Back\\\\slash\"]";
}
