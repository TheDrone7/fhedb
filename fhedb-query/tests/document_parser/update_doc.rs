use fhedb_query::prelude::{
    ContextualQuery, DocumentQuery, FieldSelector, QueryOperator, parse_contextual_query,
};

#[test]
fn basic() {
    let input = "UPDATE DOCUMENT IN users {id = 1, name: \"Jane Doe\", age: 30}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DocumentQuery::Update { .. }));

    let DocumentQuery::Update {
        collection_name,
        conditions,
        updates,
        selectors,
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(collection_name, "users");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].operator, QueryOperator::Equal);
    assert_eq!(conditions[0].value, "1");
    assert_eq!(updates.len(), 2);
    assert_eq!(updates["name"], "\"Jane Doe\"");
    assert_eq!(updates["age"], "30");
    assert!(selectors.is_empty());
}

#[test]
fn variations() {
    let input1 = "UPDATE DOC IN products {id = \"prod_123\", price: 1099.99, stock: 50}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        collection_name: name1,
        conditions: conds1,
        updates: updates1,
        ..
    } = query1
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(name1, "products");
    assert_eq!(conds1.len(), 1);
    assert_eq!(conds1[0].field_name, "id");
    assert_eq!(conds1[0].value, "\"prod_123\"");
    assert_eq!(updates1.len(), 2);
    assert_eq!(updates1["price"], "1099.99");
    assert_eq!(updates1["stock"], "50");

    let input2 = "   UpDaTe    DoC    iN    MyCollection   {field: 'value', num: 42}   ";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        collection_name: name2,
        updates: updates2,
        ..
    } = query2
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(name2, "MyCollection");
    assert_eq!(updates2.len(), 2);
    assert_eq!(updates2["field"], "\"value\"");
    assert_eq!(updates2["num"], "42");
}

#[test]
fn complex_data_types() {
    let input = "UPDATE DOCUMENT IN products {
        id = 42,
        name: \"Gaming Laptop\",
        price: 1299.99,
        in_stock: true,
        tags: [\"gaming\", \"laptop\"],
        config: \"{\\\"theme\\\": \\\"dark\\\"}\",
        matrix: [[1, 2], [3, 4]],
        path: \"C:\\Users\\Admin\"
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        collection_name,
        conditions,
        updates,
        ..
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(collection_name, "products");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].value, "42");
    assert_eq!(updates.len(), 7);
    assert_eq!(updates["name"], "\"Gaming Laptop\"");
    assert_eq!(updates["price"], "1299.99");
    assert_eq!(updates["in_stock"], "true");
    assert_eq!(updates["tags"], "[\"gaming\", \"laptop\"]");
    assert_eq!(updates["config"], "\"{\"theme\": \"dark\"}\"");
    assert_eq!(updates["matrix"], "[[1, 2], [3, 4]]");
    assert_eq!(updates["path"], "\"C:\\Users\\Admin\"");
}

#[test]
fn nested_assignments() {
    let input = "UPDATE DOCUMENT IN users {
        id = 1,
        address: \"{\\\"city\\\": \\\"Boston\\\", \\\"zipcode\\\": \\\"02101\\\"}\",
        profile: \"{\\\"bio\\\": \\\"Developer\\\", \\\"age\\\": 30}\"
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        conditions,
        updates,
        ..
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(conditions.len(), 1);
    assert_eq!(updates.len(), 2);
    assert!(updates.contains_key("address"));
    assert!(updates.contains_key("profile"));
}

#[test]
fn with_conditions() {
    let input = "UPDATE DOCUMENT IN users {
        id = 1,
        age > 18,
        name: \"Updated Name\",
        status: \"active\"
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        conditions,
        updates,
        ..
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].operator, QueryOperator::Equal);
    assert_eq!(conditions[1].field_name, "age");
    assert_eq!(conditions[1].operator, QueryOperator::GreaterThan);
    assert_eq!(updates.len(), 2);
    assert_eq!(updates["name"], "\"Updated Name\"");
    assert_eq!(updates["status"], "\"active\"");
}

#[test]
fn with_selectors() {
    let input = "UPDATE DOCUMENT IN users {
        id = 1,
        name: \"John\",
        email,
        age
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        conditions,
        updates,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert_eq!(conditions.len(), 1);
    assert_eq!(updates.len(), 1);
    assert_eq!(updates["name"], "\"John\"");
    assert_eq!(selectors.len(), 2);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "email"));
    assert!(matches!(&selectors[1], FieldSelector::Field(name) if name == "age"));
}

#[test]
fn nested_with_wildcards() {
    let input = "UPDATE DOCUMENT IN users {
        id = 1,
        profile: \"{\\\"name\\\": \\\"John\\\"}\",
        address {*}
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Update {
        selectors, updates, ..
    } = query
    else {
        panic!("Expected Update variant");
    };

    assert!(updates.contains_key("profile"));

    assert!(!selectors.is_empty());

    let has_address_selector = selectors.iter().any(|s| {
        matches!(s, FieldSelector::SubDocument { field_name, content }
            if field_name == "address" && content.selectors.iter().any(|inner| matches!(inner, FieldSelector::AllFields)))
    });
    assert!(has_address_selector);
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
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input2 = "UPDATE";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.context.contains(&"update document".to_string()));
        assert!(error.context.contains(&"document query".to_string()));
    }

    let input3 = "UPDATE DOCUMENT";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let Err(errors3) = result3 else {
        panic!("Expected Err result");
    };

    assert!(!errors3.is_empty());
    for error in errors3 {
        assert!(error.context.contains(&"update document".to_string()));
        assert!(error.expected.contains(&"IN".to_string()));
    }

    let input4 = "UPDATE DOCUMENT IN";
    let result4 = parse_contextual_query(input4);
    assert!(result4.is_err());

    let Err(errors4) = result4 else {
        panic!("Expected Err result");
    };

    assert!(!errors4.is_empty());
    for error in errors4 {
        assert!(error.context.contains(&"update document".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
    }

    let input5 = "UPDATE DOC IN users";
    let result5 = parse_contextual_query(input5);
    assert!(result5.is_err());

    let Err(errors5) = result5 else {
        panic!("Expected Err result");
    };

    assert!(!errors5.is_empty());
    for error in errors5 {
        assert!(error.context.contains(&"update document".to_string()));
        assert!(error.expected.contains(&"document body".to_string()));
    }

    let input6 = "DOCUMENT UPDATE IN users {id = 1, name: \"test\"}";
    let result6 = parse_contextual_query(input6);
    assert!(result6.is_err());

    let Err(errors6) = result6 else {
        panic!("Expected Err result");
    };

    assert!(!errors6.is_empty());
    for error in errors6 {
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input7 = "UPDATE DOCUMENT ON users {id = 1, name: \"test\"}";
    let result7 = parse_contextual_query(input7);
    assert!(result7.is_err());

    let Err(errors7) = result7 else {
        panic!("Expected Err result");
    };

    assert!(!errors7.is_empty());
    for error in errors7 {
        assert!(error.context.contains(&"update document".to_string()));
        assert!(error.expected.contains(&"IN".to_string()));
    }
}

#[test]
fn invalid_field_structure() {
    let input1 = "UPDATE DOC IN users {id 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "UPDATE DOC IN users {id = 1";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.expected.contains(&"}".to_string()));
    }

    let input3 = "UPDATE DOCUMENT IN users {id = 1, name: \"test\"} EXTRA";
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
fn invalid_no_assignments() {
    let input = "UPDATE DOC IN users {id = 1, name}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input2 = "UPDATE DOC IN users {id = 1}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "UPDATE DOC IN users {name, email}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());
}

#[test]
fn duplicate_fields() {
    let input = "UPDATE DOC IN users {id = 1, name: \"John\", name: \"Jane\"}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input = "UPDATE DOCUMENT IN products {id = 1, price: 10, stock: 5, price: 20}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}
