use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "GET SCHEMA FROM users";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::GetSchema { name } => {
            assert_eq!(name, "users");
        }
        _ => panic!("Expected CollectionQuery::GetSchema, got {:?}", result),
    }
}

#[test]
fn case_insensitive() {
    let input = "GeT sChEmA fRoM MyCollection";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::GetSchema { name } => {
            assert_eq!(name, "MyCollection");
        }
        _ => panic!("Expected CollectionQuery::GetSchema, got {:?}", result),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   GET    SCHEMA    FROM    test_collection   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::GetSchema { name } => {
            assert_eq!(name, "test_collection");
        }
        _ => panic!("Expected CollectionQuery::GetSchema, got {:?}", result),
    }
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_missing_schema() {
    let input = "GET";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"\", code: MultiSpace }"
            );
        }
    }
}

#[test]
fn invalid_missing_from() {
    let input = "GET SCHEMA";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"\", code: MultiSpace }"
            );
        }
    }
}

#[test]
fn invalid_missing_collection_name() {
    let input = "GET SCHEMA FROM";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"\", code: MultiSpace }"
            );
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "GET SCHEMA FROM users EXTRA_STUFF";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after collection query");
        }
    }
}

#[test]
fn invalid_wrong_keyword() {
    let input = "GET SCHEMAS FROM users";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"S FROM users\", code: MultiSpace }"
            );
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "SCHEMA GET FROM users";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"SCHEMA GET FROM users\", code: Tag }"
            );
        }
    }
}
