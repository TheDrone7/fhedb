use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "LIST COLLECTIONS";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::List => {}
        _ => panic!("Expected CollectionQuery::List, got {:?}", result),
    }
}

#[test]
fn case_insensitive() {
    let input = "LiSt CoLlEcTiOnS";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::List => {}
        _ => panic!("Expected CollectionQuery::List, got {:?}", result),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   LIST    COLLECTIONS   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::List => {}
        _ => panic!("Expected CollectionQuery::List, got {:?}", result),
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
fn invalid_missing_collections() {
    let input = "LIST";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"LIST\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "LIST COLLECTIONS EXTRA_STUFF";
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
    let input = "LIST COLLECTION";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"LIST COLLECTION\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTIONS LIST";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"COLLECTIONS LIST\", code: Tag }"
            );
        }
    }
}
