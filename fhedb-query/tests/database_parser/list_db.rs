use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "LIST DATABASES";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::List => {}
        _ => panic!("Expected DatabaseQuery::List"),
    }
}

#[test]
fn case_insensitive() {
    let input = "LiSt DaTaBaSeS";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::List => {}
        _ => panic!("Expected DatabaseQuery::List"),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   LIST    DATABASES   ";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::List => {}
        _ => panic!("Expected DatabaseQuery::List"),
    }
}

#[test]
fn invalid_missing_databases() {
    let input = "LIST";
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
            assert_eq!(column, 5);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "LIST DATABASES EXTRA_STUFF";
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
            assert_eq!(column, 16);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASES LIST";
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
            assert!(message.contains("Expected keyword") || message.contains("found 'DATABASES"));
            assert_eq!(line, 1);
            assert_eq!(column, 1);
            assert_eq!(context_path, vec!["query", "database"]);
        }
    }
}
