use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "INSERT DOCUMENT INTO users {id: 1, name: \"John Doe\"}";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => {
            assert_eq!(collection_name, "users");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields["id"], "1");
            assert_eq!(fields["name"], "\"John Doe\"");
        }
        _ => panic!("Expected DocumentQuery::Insert, got {:?}", result),
    }
}

#[test]
fn variations() {
    let input = "INSERT DOC INTO products {id: \"prod_123\", price: 999.99}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(fields["id"], "\"prod_123\"");
            assert_eq!(fields["price"], "999.99");
        }
        _ => panic!("Expected DocumentQuery::Insert"),
    }

    let input = "   InSeRt    DoC    iNtO    MyCollection   {field: 'value'}   ";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => {
            assert_eq!(collection_name, "MyCollection");
            assert_eq!(fields["field"], "'value'");
        }
        _ => panic!("Expected DocumentQuery::Insert"),
    }

    let input = "INSERT DOCUMENT INTO empty_test {}";
    let result = parse_document_query(input).unwrap();
    match result {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => {
            assert_eq!(collection_name, "empty_test");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected DocumentQuery::Insert"),
    }
}

#[test]
fn complex_data_types() {
    let input = "INSERT DOCUMENT INTO products {
        id: 42,
        name: \"Gaming Laptop\",
        price: 1299.99,
        in_stock: true,
        tags: [\"gaming\", \"laptop\"],
        config: \"{\\\"theme\\\": \\\"dark\\\"}\",
        matrix: [[1, 2], [3, 4]],
        path: \"C:\\\\Users\\\\Admin\"
    }";
    let result = parse_document_query(input).unwrap();

    match result {
        DocumentQuery::Insert {
            collection_name,
            fields,
        } => {
            assert_eq!(collection_name, "products");
            assert_eq!(fields.len(), 8);
            assert_eq!(fields["id"], "42");
            assert_eq!(fields["name"], "\"Gaming Laptop\"");
            assert_eq!(fields["tags"], "[\"gaming\", \"laptop\"]");
            assert_eq!(fields["config"], "\"{\\\"theme\\\": \\\"dark\\\"}\"");
            assert_eq!(fields["matrix"], "[[1, 2], [3, 4]]");
            assert_eq!(fields["path"], "\"C:\\\\Users\\\\Admin\"");
        }
        _ => panic!("Expected DocumentQuery::Insert"),
    }
}

#[test]
fn invalid_syntax() {
    let result = parse_document_query("");
    assert!(result.is_err());

    let result = parse_document_query("INSERT");
    assert!(result.is_err());
    let result = parse_document_query("INSERT DOCUMENT");
    assert!(result.is_err());
    let result = parse_document_query("INSERT DOCUMENT INTO");
    assert!(result.is_err());

    let result = parse_document_query("INSERT DOC INTO users");
    assert!(result.is_err());
    let result = parse_document_query("DOCUMENT INSERT INTO users {id: 1}");
    assert!(result.is_err());
    let result = parse_document_query("INSERT DOCUMENT IN users {id: 1}");
    assert!(result.is_err());
}

#[test]
fn invalid_field_structure() {
    let result = parse_document_query("INSERT DOC INTO users {id 1}");
    assert!(result.is_err());

    let result = parse_document_query("INSERT DOC INTO users {id: 1");
    assert!(result.is_err());

    let result = parse_document_query("INSERT DOCUMENT INTO users {id: 1} EXTRA");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after document query");
        }
    }
}

#[test]
fn duplicate_fields() {
    let input = "INSERT DOC INTO users {id: 1, name: \"John\", id: 2}";
    let result = parse_document_query(input);
    assert!(result.is_err());
}
