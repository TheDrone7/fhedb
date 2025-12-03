#[test]
fn basic_escape_sequences() {
    "Hello\\nWorld";
    "Tab\\tSeparated";
    "Carriage\\rReturn";
    "Null\\0Character";
    "Back\\\\slash";
    "Double\\\"Quote";
    "Single\\'Quote";
}

#[test]
fn realistic_strings() {
    "Line1\\nLine2\\nLine3";
    "\\\"Hello\\\" \\t\\n";
    "\\\\n\\\\t\\\\r";
    "\\n\\t\\r\\0\\\\\\\"";
    "Path: C:\\\\Users\\\\Name";
    "JSON: {\\\"key\\\": \\\"value\\\"}";
    "Tab\\tdelimited\\tdata";
}

#[test]
fn no_escape_sequences() {
    "Hello World";
    "";
    "Simple text with no escapes";
    "123456789";
}

#[test]
fn invalid_escape_sequences() {
    "\\z";
    "\\x";
    "\\123";
    "Hello\\zWorld";
    "\\a\\b\\c";
}

#[test]
fn backslash_at_end() {
    "Hello\\";
    "\\";
    "Test\\n\\";
}

#[test]
fn mixed_valid_invalid() {
    "\\nValid\\zInvalid\\tValid";
    "\\\"Good\\xBad\\\\Good";
}

#[test]
fn edge_cases() {
    "\\\\\\\\";
    "\\n\\n\\n";
    "\\\"\\\"\\\"";
    "\\'\\'\\'";
}
