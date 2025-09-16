use fhedb_query::parser::utilities::unescape_string;

#[test]
fn basic_escape_sequences() {
    assert_eq!(unescape_string("Hello\\nWorld"), "Hello\nWorld");
    assert_eq!(unescape_string("Tab\\tSeparated"), "Tab\tSeparated");
    assert_eq!(unescape_string("Carriage\\rReturn"), "Carriage\rReturn");
    assert_eq!(unescape_string("Null\\0Character"), "Null\0Character");
    assert_eq!(unescape_string("Back\\\\slash"), "Back\\slash");
    assert_eq!(unescape_string("Double\\\"Quote"), "Double\"Quote");
    assert_eq!(unescape_string("Single\\'Quote"), "Single'Quote");
}

#[test]
fn realistic_strings() {
    assert_eq!(
        unescape_string("Line1\\nLine2\\nLine3"),
        "Line1\nLine2\nLine3"
    );
    assert_eq!(unescape_string("\\\"Hello\\\" \\t\\n"), "\"Hello\" \t\n");
    assert_eq!(unescape_string("\\\\n\\\\t\\\\r"), "\\n\\t\\r");
    assert_eq!(unescape_string("\\n\\t\\r\\0\\\\\\\""), "\n\t\r\0\\\"");
    assert_eq!(
        unescape_string("Path: C:\\\\Users\\\\Name"),
        "Path: C:\\Users\\Name"
    );
    assert_eq!(
        unescape_string("JSON: {\\\"key\\\": \\\"value\\\"}"),
        "JSON: {\"key\": \"value\"}"
    );
    assert_eq!(
        unescape_string("Tab\\tdelimited\\tdata"),
        "Tab\tdelimited\tdata"
    );
}

#[test]
fn no_escape_sequences() {
    assert_eq!(unescape_string("Hello World"), "Hello World");
    assert_eq!(unescape_string(""), "");
    assert_eq!(
        unescape_string("Simple text with no escapes"),
        "Simple text with no escapes"
    );
    assert_eq!(unescape_string("123456789"), "123456789");
}

#[test]
fn invalid_escape_sequences() {
    assert_eq!(unescape_string("\\z"), "\\z");
    assert_eq!(unescape_string("\\x"), "\\x");
    assert_eq!(unescape_string("\\123"), "\\123");
    assert_eq!(unescape_string("Hello\\zWorld"), "Hello\\zWorld");
    assert_eq!(unescape_string("\\a\\b\\c"), "\\a\\b\\c");
}

#[test]
fn backslash_at_end() {
    assert_eq!(unescape_string("Hello\\"), "Hello\\");
    assert_eq!(unescape_string("\\"), "\\");
    assert_eq!(unescape_string("Test\\n\\"), "Test\n\\");
}

#[test]
fn mixed_valid_invalid() {
    assert_eq!(
        unescape_string("\\nValid\\zInvalid\\tValid"),
        "\nValid\\zInvalid\tValid"
    );
    assert_eq!(
        unescape_string("\\\"Good\\xBad\\\\Good"),
        "\"Good\\xBad\\Good"
    );
}

#[test]
fn edge_cases() {
    assert_eq!(unescape_string("\\\\\\\\"), "\\\\");
    assert_eq!(unescape_string("\\n\\n\\n"), "\n\n\n");
    assert_eq!(unescape_string("\\\"\\\"\\\""), "\"\"\"");
    assert_eq!(unescape_string("\\'\\'\\'"), "'''");
}
