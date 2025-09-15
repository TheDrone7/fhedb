use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "CREATE DATABASE test_db";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            assert_eq!(name, "test_db");
            assert_eq!(drop_if_exists, false);
        }
        _ => panic!("Expected DatabaseQuery::Create"),
    }
}

#[test]
fn case_insensitive() {
    let input = "CrEaTe DaTaBaSe MyDatabase";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            assert_eq!(name, "MyDatabase");
            assert_eq!(drop_if_exists, false);
        }
        _ => panic!("Expected DatabaseQuery::Create"),
    }
}

#[test]
fn with_drop_if_exists() {
    let input = "CREATE DATABASE test_db DROP IF EXISTS";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            assert_eq!(name, "test_db");
            assert_eq!(drop_if_exists, true);
        }
        _ => panic!("Expected DatabaseQuery::Create"),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   CREATE    DATABASE    test_db   ";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            assert_eq!(name, "test_db");
            assert_eq!(drop_if_exists, false);
        }
        _ => panic!("Expected DatabaseQuery::Create"),
    }

    let input = "   CREATE    DATABASE    test_db    DROP   IF   EXISTS   ";
    let result = parse_database_query(input).unwrap();

    match result {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            assert_eq!(name, "test_db");
            assert_eq!(drop_if_exists, true);
        }
        _ => panic!("Expected DatabaseQuery::Create"),
    }
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse database query: Parsing Error: Error { input: \"\", code: Tag }");
        }
    }
}

#[test]
fn invalid_missing_name() {
    let input = "CREATE DATABASE";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse database query: Parsing Error: Error { input: \"CREATE DATABASE\", code: Tag }");
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "CREATE DATABASE test_db EXTRA_STUFF";
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
    let input = "CREATE test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse database query: Parsing Error: Error { input: \"CREATE test_db\", code: Tag }");
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASE CREATE test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse database query: Parsing Error: Error { input: \"DATABASE CREATE test_db\", code: Tag }");
        }
    }
}
