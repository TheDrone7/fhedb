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

    let err = result.unwrap_err();
    match err {
        ParserError::SyntaxError {
            message,
            line,
            column,
            context_path,
            ..
        } => {
            assert!(message.contains("Expected keyword") || message.contains("end of input"));
            assert_eq!(line, 1);
            assert_eq!(column, 1);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_missing_name() {
    let input = "DROP DATABASE";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ParserError::SyntaxError {
            message,
            line,
            column,
            context_path,
            ..
        } => {
            assert!(message.contains("Expected identifier") || message.contains("end of input"));
            assert_eq!(line, 1);
            assert_eq!(column, 14);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "DROP DATABASE test_db EXTRA_STUFF";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ParserError::SyntaxError {
            message,
            line,
            column,
            context_path,
            ..
        } => {
            assert_eq!(message, "Unexpected input after database query");
            assert_eq!(line, 1);
            assert_eq!(column, 23);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ParserError::SyntaxError {
            message,
            line,
            column,
            context_path,
            ..
        } => {
            assert!(message.contains("Expected keyword") || message.contains("found 'test_db'"));
            assert_eq!(line, 1);
            assert_eq!(column, 1);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASE DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ParserError::SyntaxError {
            message,
            line,
            column,
            context_path,
            ..
        } => {
            assert!(message.contains("Expected keyword") || message.contains("found 'DATABASE"));
            assert_eq!(line, 1);
            assert_eq!(column, 1);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}
