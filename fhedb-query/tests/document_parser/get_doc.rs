use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "GET DOCUMENT FROM users {id = 1, name}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "1");
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::Field(name) => assert_eq!(name, "name"),
                _ => panic!("Expected FieldSelector::Field"),
            }
        }
        _ => panic!("Expected DocumentQuery::Get, got {:?}", result),
    }
}

#[test]
fn variations() {
    let input = "GET DOC FROM products {id = \"prod_123\", price}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "\"prod_123\"");
            assert_eq!(selectors.len(), 1);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }

    let input = "   GeT    DoCs    fRoM    MyCollection   {status = 'active', name}   ";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "MyCollection");
            assert_eq!(conditions[0].field_name, "status");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "'active'");
            assert_eq!(selectors.len(), 1);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }

    let input = "GET DOCUMENTS FROM test_collection {*}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "test_collection");
            assert_eq!(conditions.len(), 0);
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::AllFields => {}
                _ => panic!("Expected FieldSelector::AllFields"),
            }
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn all_operators() {
    let input = "GET DOCUMENT FROM users {
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
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 7);
            assert_eq!(selectors.len(), 0);

            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[1].field_name, "age");
            assert_eq!(conditions[1].operator, QueryOperator::GreaterThan);
            assert_eq!(conditions[2].field_name, "salary");
            assert_eq!(conditions[2].operator, QueryOperator::GreaterThanOrEqual);
            assert_eq!(conditions[3].field_name, "rating");
            assert_eq!(conditions[3].operator, QueryOperator::LessThan);
            assert_eq!(conditions[4].field_name, "experience");
            assert_eq!(conditions[4].operator, QueryOperator::LessThanOrEqual);
            assert_eq!(conditions[5].field_name, "name");
            assert_eq!(conditions[5].operator, QueryOperator::NotEqual);
            assert_eq!(conditions[6].field_name, "bio");
            assert_eq!(conditions[6].operator, QueryOperator::Similar);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn wildcard_selectors() {
    let input = "GET DOCUMENT FROM users {id = 1, *}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
        _ => panic!("Expected DocumentQuery::Get"),
    }

    let input = "GET DOCUMENT FROM users {id = 1, **}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn simple_nested() {
    let input = "GET DOCUMENT FROM users {id = 1, address {city, country}}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn nested_with_conditions() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        name,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(selectors.len(), 2);

            match &selectors[1] {
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
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn deep_nesting() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        profile {
            personal {name, age > 18},
            address {city, country {code = '+1'}}
        }
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
                    assert_eq!(field_name, "profile");
                    assert_eq!(content.selectors.len(), 2);

                    match &content.selectors[0] {
                        FieldSelector::SubDocument {
                            field_name,
                            content,
                        } => {
                            assert_eq!(field_name, "personal");
                            assert_eq!(content.conditions.len(), 1);
                            assert_eq!(content.selectors.len(), 1);
                        }
                        _ => panic!("Expected FieldSelector::SubDocument for personal"),
                    }

                    match &content.selectors[1] {
                        FieldSelector::SubDocument {
                            field_name,
                            content,
                        } => {
                            assert_eq!(field_name, "address");
                            assert_eq!(content.selectors.len(), 2);

                            match &content.selectors[1] {
                                FieldSelector::SubDocument {
                                    field_name,
                                    content,
                                } => {
                                    assert_eq!(field_name, "country");
                                    assert_eq!(content.conditions.len(), 1);
                                    assert_eq!(content.conditions[0].field_name, "code");
                                    assert_eq!(content.conditions[0].value, "'+1'");
                                }
                                _ => panic!("Expected FieldSelector::SubDocument for country"),
                            }
                        }
                        _ => panic!("Expected FieldSelector::SubDocument for address"),
                    }
                }
                _ => panic!("Expected FieldSelector::SubDocument for profile"),
            }
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn nested_wildcards() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        profile {*},
        settings {**}
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions: _,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(selectors.len(), 2);

            match &selectors[0] {
                FieldSelector::SubDocument {
                    field_name,
                    content,
                } => {
                    assert_eq!(field_name, "profile");
                    assert_eq!(content.selectors.len(), 1);
                    match &content.selectors[0] {
                        FieldSelector::AllFields => {}
                        _ => panic!("Expected FieldSelector::AllFields"),
                    }
                }
                _ => panic!("Expected FieldSelector::SubDocument"),
            }

            match &selectors[1] {
                FieldSelector::SubDocument {
                    field_name,
                    content,
                } => {
                    assert_eq!(field_name, "settings");
                    assert_eq!(content.selectors.len(), 1);
                    match &content.selectors[0] {
                        FieldSelector::AllFieldsRecursive => {}
                        _ => panic!("Expected FieldSelector::AllFieldsRecursive"),
                    }
                }
                _ => panic!("Expected FieldSelector::SubDocument"),
            }
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn mixed_syntax() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        name,
        age > 18,
        address {city = 'NYC'},
        *,
        settings {theme, **}
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 2);
            assert_eq!(selectors.len(), 4);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn empty_braces() {
    let input = "GET DOCUMENT FROM users {id = 1, address {}}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
                    assert_eq!(content.selectors.len(), 0);
                    assert_eq!(content.assignments.len(), 0);
                }
                _ => panic!("Expected FieldSelector::SubDocument"),
            }
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn only_conditions() {
    let input = "GET DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 3);
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn only_selectors() {
    let input = "GET DOCUMENT FROM users {name, email, age}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
            collection_name,
            conditions,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 0);
            assert_eq!(selectors.len(), 3);
        }
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn complex_values() {
    let input = "GET DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\\\Users\\\\Admin\",
        matrix = [[1, 2], [3, 4]]
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Get {
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
        _ => panic!("Expected DocumentQuery::Get"),
    }
}

#[test]
fn invalid_syntax() {
    let result = parse_document_query("");
    assert!(result.is_err());

    let result = parse_document_query("GET");
    assert!(result.is_err());

    let result = parse_document_query("GET DOCUMENT");
    assert!(result.is_err());

    let result = parse_document_query("GET DOCUMENT FROM");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users");
    assert!(result.is_err());

    let result = parse_document_query("DOCUMENT GET FROM users {id = 1}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOCUMENT IN users {id = 1}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOCUMENT FROM users {}");
    assert!(result.is_err());
}

#[test]
fn invalid_field_structure() {
    let result = parse_document_query("GET DOC FROM users {id 1}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users {id = 1");
    assert!(result.is_err());

    let result = parse_document_query("GET DOCUMENT FROM users {id = 1} EXTRA");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after document query");
        }
    }
}

#[test]
fn invalid_assignments_in_get() {
    let input = "GET DOC FROM users {id = 1, name: 'John'}";
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
    let input = "GET DOC FROM users {id = 1, address {city: 'NYC'}}";
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
    let result = parse_document_query("GET DOC FROM users {address {city, country}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users {address city, country}}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users {address {{city}}");
    assert!(result.is_err());
}

#[test]
fn invalid_empty_conditions() {
    let result = parse_document_query("GET DOC FROM users {id =}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users {= 1}");
    assert!(result.is_err());
}

#[test]
fn invalid_unsupported_operators() {
    let result = parse_document_query("GET DOC FROM users {id ~ 1}");
    assert!(result.is_err());

    let result = parse_document_query("GET DOC FROM users {id & 1}");
    assert!(result.is_err());
}
