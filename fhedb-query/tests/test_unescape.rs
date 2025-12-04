use fhedb_query::prelude::unescape;

#[test]
fn basic_escape_sequences() {
    assert_eq!(unescape("Hello\\nWorld"), "Hello\nWorld");
    assert_eq!(unescape("Tab\\tSeparated"), "Tab\tSeparated");
    assert_eq!(unescape("Carriage\\rReturn"), "Carriage\rReturn");
    assert_eq!(unescape("Null\\0Character"), "Null\0Character");
    assert_eq!(unescape("Back\\\\slash"), "Back\\slash");
    assert_eq!(unescape("Double\\\"Quote"), "Double\"Quote");
    assert_eq!(unescape("Single\\'Quote"), "Single'Quote");
}

#[test]
fn realistic_strings() {
    assert_eq!(unescape("Line1\\nLine2\\nLine3"), "Line1\nLine2\nLine3");
    assert_eq!(unescape("\\\"Hello\\\" \\t\\n"), "\"Hello\" \t\n");
    assert_eq!(unescape("\\\\n\\\\t\\\\r"), "\\n\\t\\r");
    assert_eq!(unescape("\\n\\t\\r\\0\\\\\\\""), "\n\t\r\0\\\"");
    assert_eq!(
        unescape("Path: C:\\\\Users\\\\Name"),
        "Path: C:\\Users\\Name"
    );
    assert_eq!(
        unescape("JSON: {\\\"key\\\": \\\"value\\\"}"),
        "JSON: {\"key\": \"value\"}"
    );
    assert_eq!(unescape("Tab\\tdelimited\\tdata"), "Tab\tdelimited\tdata");
}

#[test]
fn no_escape_sequences() {
    assert_eq!(unescape("Hello World"), "Hello World");
    assert_eq!(unescape(""), "");
    assert_eq!(
        unescape("Simple text with no escapes"),
        "Simple text with no escapes"
    );
    assert_eq!(unescape("123456789"), "123456789");
}

#[test]
fn invalid_escape_sequences() {
    assert_eq!(unescape("\\z"), "\\z");
    assert_eq!(unescape("\\x"), "\\x");
    assert_eq!(unescape("\\1"), "\\1");
    assert_eq!(unescape("Hello\\zWorld"), "Hello\\zWorld");
    assert_eq!(unescape("\\a"), "\\a");
}

#[test]
fn backslash_at_end() {
    assert_eq!(unescape("Hello\\"), "Hello\\");
    assert_eq!(unescape("\\"), "\\");
    assert_eq!(unescape("Test\\n\\"), "Test\n\\");
}

#[test]
fn mixed_valid_invalid() {
    assert_eq!(
        unescape("\\nValid\\zInvalid\\tValid"),
        "\nValid\\zInvalid\tValid"
    );
    assert_eq!(unescape("\\\"Good\\xBad\\\\Good"), "\"Good\\xBad\\Good");
}

#[test]
fn edge_cases() {
    assert_eq!(unescape("\\\\\\\\"), "\\\\");
    assert_eq!(unescape("\\n\\n\\n"), "\n\n\n");
    assert_eq!(unescape("\\\"\\\"\\\""), "\"\"\"");
    assert_eq!(unescape("\\'\\'\\'"), "'''");
}
