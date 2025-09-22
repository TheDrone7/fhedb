use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "DELETE DOCUMENT FROM users {id = 1}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "1");
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete, got {:?}", result),
    }
}

#[test]
fn with_remove_keyword() {
    let input = "REMOVE DOC FROM products {id = \"prod_123\"}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions.len(), 1);
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].value, "\"prod_123\"");
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn invalid_empty_delete_query() {
    let input = "DELETE DOCUMENT FROM users {}";
    let result = parse_document_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert!(message.contains("Parsing Error"));
        }
    }
}

#[test]
fn variations() {
    let input = "DELETE DOC FROM products {id = \"prod_123\"}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].value, "\"prod_123\"");
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }

    let input = "   DeLeTe    DoCs    fRoM    MyCollection   {status = 'active'}   ";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "MyCollection");
            assert_eq!(conditions[0].field_name, "status");
            assert_eq!(conditions[0].value, "'active'");
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }

    let input = "REMOVE DOCUMENTS FROM test_collection {active = true}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "test_collection");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn all_operators() {
    let input = "DELETE DOCUMENT FROM users {
        id = 1,
        age > 18,
        salary >= 50000,
        rating < 5.0,
        experience <= 10,
        name != 'admin',
        bio == 'developer'
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 7);
            assert_eq!(selectors.len(), 0);

            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "1");

            assert_eq!(conditions[1].field_name, "age");
            assert_eq!(conditions[1].operator, QueryOperator::GreaterThan);
            assert_eq!(conditions[1].value, "18");

            assert_eq!(conditions[2].field_name, "salary");
            assert_eq!(conditions[2].operator, QueryOperator::GreaterThanOrEqual);
            assert_eq!(conditions[2].value, "50000");

            assert_eq!(conditions[3].field_name, "rating");
            assert_eq!(conditions[3].operator, QueryOperator::LessThan);
            assert_eq!(conditions[3].value, "5.0");

            assert_eq!(conditions[4].field_name, "experience");
            assert_eq!(conditions[4].operator, QueryOperator::LessThanOrEqual);
            assert_eq!(conditions[4].value, "10");

            assert_eq!(conditions[5].field_name, "name");
            assert_eq!(conditions[5].operator, QueryOperator::NotEqual);
            assert_eq!(conditions[5].value, "'admin'");

            assert_eq!(conditions[6].field_name, "bio");
            assert_eq!(conditions[6].operator, QueryOperator::Similar);
            assert_eq!(conditions[6].value, "'developer'");
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn with_selectors() {
    let input = "DELETE DOCUMENT FROM users {id = 1, name, email}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 2);
            match &selectors[0] {
                FieldSelector::Field(name) => assert_eq!(name, "name"),
                _ => panic!("Expected FieldSelector::Field"),
            }
            match &selectors[1] {
                FieldSelector::Field(name) => assert_eq!(name, "email"),
                _ => panic!("Expected FieldSelector::Field"),
            }
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn wildcard_selectors() {
    let input = "DELETE DOCUMENT FROM users {id = 1, *}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::AllFields => {}
                _ => panic!("Expected FieldSelector::AllFields"),
            }
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }

    let input = "DELETE DOCUMENT FROM users {id = 1, **}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::AllFieldsRecursive => {}
                _ => panic!("Expected FieldSelector::AllFieldsRecursive"),
            }
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn simple_nested() {
    let input = "DELETE DOCUMENT FROM users {id = 1, address {city, country}}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 1);

            match &selectors[0] {
                FieldSelector::SubDocument {
                    field_name,
                    content,
                } => {
                    assert_eq!(field_name, "address");
                    assert_eq!(content.conditions.len(), 0);
                    assert_eq!(content.assignments.len(), 0);
                    assert_eq!(content.selectors.len(), 2);

                    match &content.selectors[0] {
                        FieldSelector::Field(name) => assert_eq!(name, "city"),
                        _ => panic!("Expected FieldSelector::Field"),
                    }
                    match &content.selectors[1] {
                        FieldSelector::Field(name) => assert_eq!(name, "country"),
                        _ => panic!("Expected FieldSelector::Field"),
                    }
                }
                _ => panic!("Expected FieldSelector::SubDocument"),
            }
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn nested_with_conditions() {
    let input = "DELETE DOCUMENT FROM users {
        id = 1,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 1);

            match &selectors[0] {
                FieldSelector::SubDocument {
                    field_name,
                    content,
                } => {
                    assert_eq!(field_name, "address");
                    assert_eq!(content.conditions.len(), 2);
                    assert_eq!(content.selectors.len(), 1);

                    assert_eq!(content.conditions[0].field_name, "city");
                    assert_eq!(content.conditions[0].operator, QueryOperator::Equal);
                    assert_eq!(content.conditions[0].value, "'NYC'");

                    assert_eq!(content.conditions[1].field_name, "zipcode");
                    assert_eq!(content.conditions[1].operator, QueryOperator::NotEqual);
                    assert_eq!(content.conditions[1].value, "'10001'");
                }
                _ => panic!("Expected FieldSelector::SubDocument"),
            }
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn only_conditions() {
    let input = "DELETE DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 3);
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn only_selectors() {
    let input = "DELETE DOCUMENT FROM users {name, email, age}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 0);
            assert_eq!(selectors.len(), 3);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn mixed_syntax() {
    let input = "DELETE DOCUMENT FROM users {
        id = 1,
        name,
        age > 18,
        address {city = 'NYC'},
        *
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 2);
            assert_eq!(selectors.len(), 3);
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn complex_values() {
    let input = "DELETE DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\\\Users\\\\Admin\",
        matrix = [[1, 2], [3, 4]]
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Delete {
            collection_name,
            conditions,
            selectors: _,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions.len(), 4);
            assert_eq!(conditions[0].value, "'[\"electronics\", \"mobile\"]'");
            assert_eq!(conditions[1].value, "\"{\\\"theme\\\": \\\"dark\\\"}\"");
            assert_eq!(conditions[2].value, "\"C:\\\\Users\\\\Admin\"");
            assert_eq!(conditions[3].value, "[[1, 2], [3, 4]]");
        }
        _ => panic!("Expected DocumentQuery::Delete"),
    }
}

#[test]
fn invalid_syntax() {
    let result = parse_document_query("");
    assert!(result.is_err());

    let result = parse_document_query("DELETE");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOCUMENT");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOCUMENT FROM");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users");
    assert!(result.is_err());

    let result = parse_document_query("DOCUMENT DELETE FROM users {id = 1}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOCUMENT IN users {id = 1}");
    assert!(result.is_err());
}

#[test]
fn invalid_field_structure() {
    let result = parse_document_query("DELETE DOC FROM users {id 1}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users {id = 1");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOCUMENT FROM users {id = 1} EXTRA");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after document query");
        }
    }
}

#[test]
fn invalid_empty_query() {
    let input = "DELETE DOCUMENT FROM users {}";
    let result = parse_document_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert!(message.contains("Parsing Error"));
        }
    }
}

#[test]
fn invalid_assignments() {
    let input = "DELETE DOC FROM users {id = 1, name: 'John'}";
    let result = parse_document_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert!(message.contains("Parsing Error"));
        }
    }
}

#[test]
fn invalid_assignments_in_nested() {
    let input = "DELETE DOC FROM users {id = 1, address {city: 'NYC'}}";
    let result = parse_document_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert!(message.contains("Parsing Error"));
        }
    }
}

#[test]
fn invalid_malformed_nested() {
    let result = parse_document_query("DELETE DOC FROM users {address {city, country}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users {address city, country}}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users {address {{city}}");
    assert!(result.is_err());
}

#[test]
fn invalid_empty_conditions() {
    let result = parse_document_query("DELETE DOC FROM users {id =}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users {= 1}");
    assert!(result.is_err());
}

#[test]
fn invalid_unsupported_operators() {
    let result = parse_document_query("DELETE DOC FROM users {id ~ 1}");
    assert!(result.is_err());

    let result = parse_document_query("DELETE DOC FROM users {id & 1}");
    assert!(result.is_err());
}
