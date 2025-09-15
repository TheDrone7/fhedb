use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "DROP COLLECTION test_collection";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Drop { name } => {
            assert_eq!(name, "test_collection");
        }
        _ => panic!("Expected CollectionQuery::Drop, got {:?}", result),
    }
}

#[test]
fn case_insensitive() {
    let input = "DrOp CoLlEcTiOn MyCollection";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Drop { name } => {
            assert_eq!(name, "MyCollection");
        }
        _ => panic!("Expected CollectionQuery::Drop, got {:?}", result),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   DROP    COLLECTION    test_collection   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Drop { name } => {
            assert_eq!(name, "test_collection");
        }
        _ => panic!("Expected CollectionQuery::Drop, got {:?}", result),
    }
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse collection query: Parsing Error: Error { input: \"\", code: Tag }");
        }
    }
}

#[test]
fn invalid_missing_name() {
    let input = "DROP COLLECTION";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse collection query: Parsing Error: Error { input: \"DROP COLLECTION\", code: Tag }");
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "DROP COLLECTION test_collection EXTRA_STUFF";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after collection query");
        }
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "DROP test_collection";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse collection query: Parsing Error: Error { input: \"DROP test_collection\", code: Tag }");
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION DROP test_collection";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Failed to parse collection query: Parsing Error: Error { input: \"COLLECTION DROP test_collection\", code: Tag }");
        }
    }
}
