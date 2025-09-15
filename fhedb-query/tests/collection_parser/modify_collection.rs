use fhedb_core::prelude::FieldType;
use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "MODIFY COLLECTION users {name: string, age: drop}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            assert_eq!(name, "users");
            assert_eq!(modifications.len(), 2);

            match &modifications["name"] {
                FieldModification::Set(field_def) => {
                    assert_eq!(field_def.field_type, FieldType::String);
                }
                _ => panic!("Expected FieldModification::Set for name field"),
            }

            match &modifications["age"] {
                FieldModification::Drop => {}
                _ => panic!("Expected FieldModification::Drop for age field"),
            }
        }
        _ => panic!("Expected CollectionQuery::Modify, got {:?}", result),
    }

    let input = "ALTER COLLECTION products {price: float, old_field: drop}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            assert_eq!(name, "products");
            assert_eq!(modifications.len(), 2);

            match &modifications["price"] {
                FieldModification::Set(field_def) => {
                    assert_eq!(field_def.field_type, FieldType::Float);
                }
                _ => panic!("Expected FieldModification::Set for price field"),
            }

            match &modifications["old_field"] {
                FieldModification::Drop => {}
                _ => panic!("Expected FieldModification::Drop for old_field"),
            }
        }
        _ => panic!("Expected CollectionQuery::Modify, got {:?}", result),
    }
}

#[test]
fn case_insensitive() {
    let input = "MoDiFy CoLlEcTiOn MyCollection {FiElD: dROp}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            assert_eq!(name, "MyCollection");
            assert_eq!(modifications.len(), 1);

            match &modifications["FiElD"] {
                FieldModification::Drop => {}
                _ => panic!("Expected FieldModification::Drop"),
            }
        }
        _ => panic!("Expected CollectionQuery::Modify, got {:?}", result),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   MODIFY    COLLECTION    test_collection   {field1: int, field2: drop}   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            assert_eq!(name, "test_collection");
            assert_eq!(modifications.len(), 2);

            match &modifications["field1"] {
                FieldModification::Set(field_def) => {
                    assert_eq!(field_def.field_type, FieldType::Int);
                }
                _ => panic!("Expected FieldModification::Set for field1"),
            }

            match &modifications["field2"] {
                FieldModification::Drop => {}
                _ => panic!("Expected FieldModification::Drop for field2"),
            }
        }
        _ => panic!("Expected CollectionQuery::Modify, got {:?}", result),
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
fn invalid_missing_name() {
    let input = "MODIFY COLLECTION";
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
    let input = "MODIFY COLLECTION test_collection {field: int} EXTRA_STUFF";
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
    let input = "MODIFY test_collection {field: int}";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"test_collection {field: int}\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION MODIFY test_collection {field: int}";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"COLLECTION MODIFY test_collection {field: int}\", code: Tag }"
            );
        }
    }
}
