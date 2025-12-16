use fhedb_query::prelude::parse_contextual_query;
use fhedb_types::{ContextualQuery, DocumentQuery};

#[test]
fn basic() {
    let input = "INSERT DOCUMENT INTO users {id: 1, name: \"John Doe\"}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DocumentQuery::Insert { .. }));

    let DocumentQuery::Insert {
        collection_name,
        fields,
    } = query
    else {
        panic!("Expected Insert variant");
    };

    assert_eq!(collection_name, "users");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields["id"], "1");
    assert_eq!(fields["name"], "\"John Doe\"");
}

#[test]
fn variations() {
    let input1 = "INSERT DOC INTO products {id: \"prod_123\", price: 999.99}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Insert {
        collection_name: name1,
        fields: fields1,
    } = query1
    else {
        panic!("Expected Insert variant");
    };

    assert_eq!(name1, "products");
    assert_eq!(fields1.len(), 2);
    assert_eq!(fields1["id"], "\"prod_123\"");
    assert_eq!(fields1["price"], "999.99");

    let input2 = "   InSeRt    DoCs    iNtO    MyCollection   {field: 'value'}   ";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Insert {
        collection_name: name2,
        fields: fields2,
    } = query2
    else {
        panic!("Expected Insert variant");
    };

    assert_eq!(name2, "MyCollection");
    assert_eq!(fields2.len(), 1);
    assert_eq!(fields2["field"], "\"value\"");

    let input3 = "INSERT DOCUMENTS INTO empty_test {}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_ok());

    let Ok(ContextualQuery::Document(query3)) = result3 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Insert {
        collection_name: name3,
        fields: fields3,
    } = query3
    else {
        panic!("Expected Insert variant");
    };

    assert_eq!(name3, "empty_test");
    assert_eq!(fields3.len(), 0);
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
        path: \"C:\\\\\\\\Users\\\\\\\\Admin\"
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Insert {
        collection_name,
        fields,
    } = query
    else {
        panic!("Expected Insert variant");
    };

    assert_eq!(collection_name, "products");
    assert_eq!(fields.len(), 8);
    assert_eq!(fields["id"], "42");
    assert_eq!(fields["name"], "\"Gaming Laptop\"");
    assert_eq!(fields["price"], "1299.99");
    assert_eq!(fields["in_stock"], "true");
    assert_eq!(fields["tags"], "[\"gaming\", \"laptop\"]");
    assert_eq!(fields["config"], "\"{\"theme\": \"dark\"}\"");
    assert_eq!(fields["matrix"], "[[1, 2], [3, 4]]");
    assert_eq!(fields["path"], "\"C:\\\\Users\\\\Admin\"");
}

#[test]
fn invalid_syntax() {
    let input1 = "";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let Err(errors1) = result1 else {
        panic!("Expected Err result");
    };

    assert!(!errors1.is_empty());
    for error in errors1 {
        assert!(error.span.start == 0 && error.span.end == 0);
        assert!(error.found.is_none());
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input2 = "INSERT";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.context.contains(&"document query".to_string()));
    }

    let input3 = "INSERT DOCUMENT";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let Err(errors3) = result3 else {
        panic!("Expected Err result");
    };

    assert!(!errors3.is_empty());
    for error in errors3 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.expected.contains(&"INTO".to_string()));
    }

    let input4 = "INSERT DOCUMENT INTO";
    let result4 = parse_contextual_query(input4);
    assert!(result4.is_err());

    let Err(errors4) = result4 else {
        panic!("Expected Err result");
    };

    assert!(!errors4.is_empty());
    for error in errors4 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
    }

    let input5 = "INSERT DOC INTO users";
    let result5 = parse_contextual_query(input5);
    assert!(result5.is_err());

    let Err(errors5) = result5 else {
        panic!("Expected Err result");
    };

    assert!(!errors5.is_empty());
    for error in errors5 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.expected.contains(&"document body".to_string()));
    }

    let input6 = "DOCUMENT INSERT INTO users {id: 1}";
    let result6 = parse_contextual_query(input6);
    assert!(result6.is_err());

    let Err(errors6) = result6 else {
        panic!("Expected Err result");
    };

    assert!(!errors6.is_empty());
    for error in errors6 {
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input7 = "INSERT DOCUMENT IN users {id: 1}";
    let result7 = parse_contextual_query(input7);
    assert!(result7.is_err());

    let Err(errors7) = result7 else {
        panic!("Expected Err result");
    };

    assert!(!errors7.is_empty());
    for error in errors7 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.expected.contains(&"INTO".to_string()));
    }
}

#[test]
fn invalid_field_structure() {
    let input1 = "INSERT DOC INTO users {id 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let Err(errors1) = result1 else {
        panic!("Expected Err result");
    };

    assert!(!errors1.is_empty());
    for error in errors1 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.context.contains(&"document body".to_string()));
    }

    let input2 = "INSERT DOC INTO users {id: 1";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.context.contains(&"insert document".to_string()));
        assert!(error.context.contains(&"document body".to_string()));
        assert!(error.expected.contains(&"}".to_string()));
    }

    let input3 = "INSERT DOCUMENT INTO users {id: 1} EXTRA";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let Err(errors3) = result3 else {
        panic!("Expected Err result");
    };

    assert!(!errors3.is_empty());
    for error in errors3 {
        assert!(error.expected.contains(&"end of input".to_string()));
        assert!(error.message == "Unexpected input after query");
    }
}

#[test]
fn invalid_duplicate_fields() {
    let input = "INSERT DOC INTO users {id: 1, name: \"John\", id: 2}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_conditions_in_insert() {
    let input = "INSERT DOC INTO users {id = 1, name: \"John\"}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input2 = "INSERT DOCUMENT INTO users {name: \"John\", age > 18}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "INSERT DOC INTO users {status != \"deleted\"}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());
}

#[test]
fn invalid_selectors_in_insert() {
    let input = "INSERT DOC INTO users {name: \"John\", email}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input2 = "INSERT DOCUMENT INTO users {name: \"John\", *}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "INSERT DOC INTO users {name: \"John\", **}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let input4 = "INSERT DOC INTO users {name: \"John\", address {city}}";
    let result4 = parse_contextual_query(input4);
    assert!(result4.is_err());
}
