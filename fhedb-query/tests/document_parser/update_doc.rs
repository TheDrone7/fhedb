use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "UPDATE DOCUMENT FROM users {id = 1, name: \"John Doe\"}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "1");
            assert_eq!(updates.len(), 1);
            assert_eq!(updates["name"], "\"John Doe\"");
            assert_eq!(selectors.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Update, got {:?}", result),
    }
}

#[test]
fn variations() {
    let input = "UPDATE DOC FROM products {id = \"prod_123\", price: 999.99}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            ..
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "\"prod_123\"");
            assert_eq!(updates["price"], "999.99");
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }

    let input = "   UpDaTe    DoCs    fRoM    MyCollection   {status = 'active', description: 'updated'}   ";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            ..
        } => {
            assert_eq!(collection_name, "MyCollection");
            assert_eq!(conditions[0].field_name, "status");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "'active'");
            assert_eq!(updates["description"], "'updated'");
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn multiple_conditions_and_updates() {
    let input = "UPDATE DOCUMENT FROM users {
        id = 1,
        age > 18,
        name: \"Updated Name\",
        email: \"new@email.com\",
        status: \"active\"
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            ..
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 2);
            assert_eq!(updates.len(), 3);

            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(conditions[0].operator, QueryOperator::Equal);
            assert_eq!(conditions[0].value, "1");

            assert_eq!(conditions[1].field_name, "age");
            assert_eq!(conditions[1].operator, QueryOperator::GreaterThan);
            assert_eq!(conditions[1].value, "18");

            assert_eq!(updates["name"], "\"Updated Name\"");
            assert_eq!(updates["email"], "\"new@email.com\"");
            assert_eq!(updates["status"], "\"active\"");
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn all_operators() {
    let input = "UPDATE DOCUMENT FROM users {
        id = 1,
        age > 18,
        salary >= 50000,
        rating < 5.0,
        experience <= 10,
        name != 'admin',
        bio == 'developer',
        status: \"updated\"
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            ..
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 7);
            assert_eq!(updates.len(), 1);

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

            assert_eq!(updates["status"], "\"updated\"");
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn with_selectors() {
    let input = "UPDATE DOCUMENT FROM users {id = 1, name: \"John\", email}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            selectors,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(conditions.len(), 1);
            assert_eq!(updates.len(), 1);
            assert_eq!(selectors.len(), 1);

            match &selectors[0] {
                FieldSelector::Field(name) => assert_eq!(name, "email"),
                _ => panic!("Expected FieldSelector::Field"),
            }
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn wildcard_selectors() {
    let input = "UPDATE DOCUMENTS FROM users {id = 1, name: \"John\", *}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update { selectors, .. } => {
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::AllFields => {}
                _ => panic!("Expected FieldSelector::AllFields"),
            }
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }

    let input = "UPDATE DOC FROM users {id = 1, status: \"active\", **}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update { selectors, .. } => {
            assert_eq!(selectors.len(), 1);
            match &selectors[0] {
                FieldSelector::AllFieldsRecursive => {}
                _ => panic!("Expected FieldSelector::AllFieldsRecursive"),
            }
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn complex_data_types() {
    let input = "UPDATE DOCUMENT FROM products {
        id = 42,
        name: \"Updated Gaming Laptop\",
        price: 1399.99,
        in_stock: false,
        tags: [\"gaming\", \"laptop\", \"updated\"],
        config: \"{\\\"theme\\\": \\\"light\\\"}\",
        matrix: [[5, 6], [7, 8]]
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Update {
            collection_name,
            conditions,
            updates,
            ..
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(conditions.len(), 1);
            assert_eq!(updates.len(), 6);
            assert_eq!(conditions[0].field_name, "id");
            assert_eq!(updates["name"], "\"Updated Gaming Laptop\"");
            assert_eq!(updates["price"], "1399.99");
            assert_eq!(updates["tags"], "[\"gaming\", \"laptop\", \"updated\"]");
            assert_eq!(updates["config"], "\"{\\\"theme\\\": \\\"light\\\"}\"");
            assert_eq!(updates["matrix"], "[[5, 6], [7, 8]]");
        }
        _ => panic!("Expected DocumentQuery::Update"),
    }
}

#[test]
fn invalid_syntax() {
    let result = parse_document_query("");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOCUMENT");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOCUMENT FROM");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOC FROM users");
    assert!(result.is_err());

    let result = parse_document_query("DOCUMENT UPDATE FROM users {id = 1, name: \"John\"}");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOCUMENT IN users {id = 1, name: \"John\"}");
    assert!(result.is_err());
}

#[test]
fn invalid_field_structure() {
    let result = parse_document_query("UPDATE DOC FROM users {id 1}");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOC FROM users {id = 1");
    assert!(result.is_err());

    let result = parse_document_query("UPDATE DOCUMENT FROM users {id = 1, name: \"John\"} EXTRA");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after document query");
        }
    }
}

#[test]
fn duplicate_fields() {
    let input = "UPDATE DOC FROM users {id = 1, name: \"John\", name: \"Jane\"}";
    let result = parse_document_query(input);
    assert!(result.is_err());
}
