use fhedb_core::prelude::Unescapable;

#[test]
fn basic_escape_sequences() {
    assert_eq!("Hello\\nWorld".unescape(), "Hello\nWorld");
    assert_eq!("Tab\\tSeparated".unescape(), "Tab\tSeparated");
    assert_eq!("Carriage\\rReturn".unescape(), "Carriage\rReturn");
    assert_eq!("Null\\0Character".unescape(), "Null\0Character");
    assert_eq!("Back\\\\slash".unescape(), "Back\\slash");
    assert_eq!("Double\\\"Quote".unescape(), "Double\"Quote");
    assert_eq!("Single\\'Quote".unescape(), "Single'Quote");
}

#[test]
fn realistic_strings() {
    assert_eq!("Line1\\nLine2\\nLine3".unescape(), "Line1\nLine2\nLine3");
    assert_eq!("\\\"Hello\\\" \\t\\n".unescape(), "\"Hello\" \t\n");
    assert_eq!("\\\\n\\\\t\\\\r".unescape(), "\\n\\t\\r");
    assert_eq!("\\n\\t\\r\\0\\\\\\\"".unescape(), "\n\t\r\0\\\"");
    assert_eq!(
        "Path: C:\\\\Users\\\\Name".unescape(),
        "Path: C:\\Users\\Name"
    );
    assert_eq!(
        "JSON: {\\\"key\\\": \\\"value\\\"}".unescape(),
        "JSON: {\"key\": \"value\"}"
    );
    assert_eq!("Tab\\tdelimited\\tdata".unescape(), "Tab\tdelimited\tdata");
}

#[test]
fn no_escape_sequences() {
    assert_eq!("Hello World".unescape(), "Hello World");
    assert_eq!("".unescape(), "");
    assert_eq!(
        "Simple text with no escapes".unescape(),
        "Simple text with no escapes"
    );
    assert_eq!("123456789".unescape(), "123456789");
}

#[test]
fn invalid_escape_sequences() {
    assert_eq!("\\z".unescape(), "\\z");
    assert_eq!("\\x".unescape(), "\\x");
    assert_eq!("\\1".unescape(), "\\1");
    assert_eq!("Hello\\zWorld".unescape(), "Hello\\zWorld");
    assert_eq!("\\a".unescape(), "\\a");
}

#[test]
fn backslash_at_end() {
    assert_eq!("Hello\\".unescape(), "Hello\\");
    assert_eq!("\\".unescape(), "\\");
    assert_eq!("Test\\n\\".unescape(), "Test\n\\");
}

#[test]
fn mixed_valid_invalid() {
    assert_eq!(
        "\\nValid\\zInvalid\\tValid".unescape(),
        "\nValid\\zInvalid\tValid"
    );
    assert_eq!("\\\"Good\\xBad\\\\Good".unescape(), "\"Good\\xBad\\Good");
}

#[test]
fn edge_cases() {
    assert_eq!("\\\\\\\\".unescape(), "\\\\");
    assert_eq!("\\n\\n\\n".unescape(), "\n\n\n");
    assert_eq!("\\\"\\\"\\\"".unescape(), "\"\"\"");
    assert_eq!("\\'\\'\\'".unescape(), "'''");
}
