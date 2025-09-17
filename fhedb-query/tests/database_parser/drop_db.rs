use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "DROP DATABASE test_db";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Drop { name } => {
            assert_eq!(name, "test_db");
        }
        _ => panic!("Expected DatabaseQuery::Drop"),
    }
}

#[test]
fn case_insensitive() {
    let input = "DrOp DaTaBaSe MyDatabase";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Drop { name } => {
            assert_eq!(name, "MyDatabase");
        }
        _ => panic!("Expected DatabaseQuery::Drop"),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   DROP    DATABASE    test_db   ";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Drop { name } => {
            assert_eq!(name, "test_db");
        }
        _ => panic!("Expected DatabaseQuery::Drop"),
    }
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_missing_name() {
    let input = "DROP DATABASE";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"DROP DATABASE\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "DROP DATABASE test_db EXTRA_STUFF";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after database query");
        }
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"DROP test_db\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASE DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"DATABASE DROP test_db\", code: Tag }"
            );
        }
    }
}
