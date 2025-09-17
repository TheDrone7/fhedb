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

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"\", code: MultiSpace }"
            );
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "LIST DATABASES EXTRA_STUFF";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after database query");
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASES LIST";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse database query: Parsing Error: Error { input: \"DATABASES LIST\", code: Tag }"
            );
        }
    }
}
