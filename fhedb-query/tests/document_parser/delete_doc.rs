use fhedb_query::prelude::{
    ContextualQuery, DocumentQuery, FieldSelector, QueryOperator, parse_contextual_query,
};

#[test]
fn basic() {
    let input = "DELETE DOCUMENT FROM users {id = 1}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DocumentQuery::Delete { .. }));

    let DocumentQuery::Delete {
        collection_name,
        conditions,
        selectors,
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(collection_name, "users");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].operator, QueryOperator::Equal);
    assert_eq!(conditions[0].value, "1");
    assert!(selectors.is_empty());
}

#[test]
fn with_remove_keyword() {
    let input = "REMOVE DOC FROM products {id = \"prod_123\"}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        collection_name,
        conditions,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(collection_name, "products");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].value, "\"prod_123\"");
}

#[test]
fn variations() {
    let input1 = "DELETE DOC FROM products {id = \"prod_123\"}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        collection_name: name1,
        conditions: conds1,
        ..
    } = query1
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(name1, "products");
    assert_eq!(conds1.len(), 1);
    assert_eq!(conds1[0].field_name, "id");

    let input2 = "   DeLeTe    DoCs    fRoM    MyCollection   {status = 'active'}   ";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        collection_name: name2,
        conditions: conds2,
        ..
    } = query2
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(name2, "MyCollection");
    assert_eq!(conds2.len(), 1);
    assert_eq!(conds2[0].field_name, "status");
    assert_eq!(conds2[0].value, "\"active\"");

    let input3 = "REMOVE DOCUMENTS FROM test_collection {active = true}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_ok());

    let Ok(ContextualQuery::Document(query3)) = result3 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        collection_name: name3,
        conditions: conds3,
        ..
    } = query3
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(name3, "test_collection");
    assert_eq!(conds3.len(), 1);
    assert_eq!(conds3[0].field_name, "active");
    assert_eq!(conds3[0].value, "true");
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
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        collection_name,
        conditions,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(collection_name, "users");
    assert_eq!(conditions.len(), 7);

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
    assert_eq!(conditions[5].value, "\"admin\"");

    assert_eq!(conditions[6].field_name, "bio");
    assert_eq!(conditions[6].operator, QueryOperator::Similar);
    assert_eq!(conditions[6].value, "\"developer\"");
}

#[test]
fn with_selectors() {
    let input = "DELETE DOCUMENT FROM users {id = 1, name, email}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 1);
    assert_eq!(selectors.len(), 2);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "name"));
    assert!(matches!(&selectors[1], FieldSelector::Field(name) if name == "email"));
}

#[test]
fn wildcard_selectors() {
    let input1 = "DELETE DOCUMENT FROM users {id = 1, *}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        selectors: sels1, ..
    } = query1
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(sels1.len(), 1);
    assert!(matches!(sels1[0], FieldSelector::AllFields));

    let input2 = "DELETE DOCUMENT FROM users {id = 1, **}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        selectors: sels2, ..
    } = query2
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(sels2.len(), 1);
    assert!(matches!(sels2[0], FieldSelector::AllFieldsRecursive));
}

#[test]
fn simple_nested() {
    let input = "DELETE DOCUMENT FROM users {id = 1, address {city, country}}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 1);
    assert_eq!(selectors.len(), 1);

    let FieldSelector::SubDocument {
        field_name,
        content,
    } = &selectors[0]
    else {
        panic!("Expected SubDocument selector");
    };

    assert_eq!(field_name, "address");
    assert_eq!(content.selectors.len(), 2);
    assert!(matches!(&content.selectors[0], FieldSelector::Field(name) if name == "city"));
    assert!(matches!(&content.selectors[1], FieldSelector::Field(name) if name == "country"));
}

#[test]
fn nested_with_conditions() {
    let input = "DELETE DOCUMENT FROM users {
        id = 1,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 1);
    assert_eq!(selectors.len(), 1);

    let FieldSelector::SubDocument {
        field_name,
        content,
    } = &selectors[0]
    else {
        panic!("Expected SubDocument selector");
    };

    assert_eq!(field_name, "address");
    assert_eq!(content.conditions.len(), 2);
    assert_eq!(content.conditions[0].field_name, "city");
    assert_eq!(content.conditions[0].operator, QueryOperator::Equal);
    assert_eq!(content.conditions[1].field_name, "zipcode");
    assert_eq!(content.conditions[1].operator, QueryOperator::NotEqual);
    assert_eq!(content.selectors.len(), 1);
    assert!(matches!(&content.selectors[0], FieldSelector::Field(name) if name == "country"));
}

#[test]
fn only_conditions() {
    let input = "DELETE DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 3);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[1].field_name, "age");
    assert_eq!(conditions[2].field_name, "status");
    assert!(selectors.is_empty());
}

#[test]
fn only_selectors() {
    let input = "DELETE DOCUMENT FROM users {name, email, age}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert!(conditions.is_empty());
    assert_eq!(selectors.len(), 3);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "name"));
    assert!(matches!(&selectors[1], FieldSelector::Field(name) if name == "email"));
    assert!(matches!(&selectors[2], FieldSelector::Field(name) if name == "age"));
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
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[1].field_name, "age");

    assert_eq!(selectors.len(), 3);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "name"));
    assert!(
        matches!(&selectors[1], FieldSelector::SubDocument { field_name, .. } if field_name == "address")
    );
    assert!(matches!(selectors[2], FieldSelector::AllFields));
}

#[test]
fn complex_values() {
    let input = "DELETE DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\\\Users\\\\Admin\",
        matrix = [[1, 2], [3, 4]]
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Delete { conditions, .. } = query else {
        panic!("Expected Delete variant");
    };

    assert_eq!(conditions.len(), 4);
    assert_eq!(conditions[0].field_name, "tags");
    assert_eq!(conditions[0].operator, QueryOperator::Similar);
    assert_eq!(conditions[1].field_name, "config");
    assert_eq!(conditions[2].field_name, "path");
    assert_eq!(conditions[2].operator, QueryOperator::NotEqual);
    assert_eq!(conditions[3].field_name, "matrix");
    assert_eq!(conditions[3].value, "[[1, 2], [3, 4]]");
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

    let input2 = "DELETE";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.context.contains(&"delete document".to_string()));
        assert!(error.context.contains(&"document query".to_string()));
    }

    let input3 = "DELETE DOCUMENT";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let Err(errors3) = result3 else {
        panic!("Expected Err result");
    };

    assert!(!errors3.is_empty());
    for error in errors3 {
        assert!(error.context.contains(&"delete document".to_string()));
        assert!(error.expected.contains(&"FROM".to_string()));
    }

    let input4 = "DELETE DOCUMENT FROM";
    let result4 = parse_contextual_query(input4);
    assert!(result4.is_err());

    let Err(errors4) = result4 else {
        panic!("Expected Err result");
    };

    assert!(!errors4.is_empty());
    for error in errors4 {
        assert!(error.context.contains(&"delete document".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
    }

    let input5 = "DELETE DOC FROM users";
    let result5 = parse_contextual_query(input5);
    assert!(result5.is_err());

    let Err(errors5) = result5 else {
        panic!("Expected Err result");
    };

    assert!(!errors5.is_empty());
    for error in errors5 {
        assert!(error.context.contains(&"delete document".to_string()));
        assert!(error.expected.contains(&"document body".to_string()));
    }

    let input6 = "DOCUMENT DELETE FROM users {id = 1}";
    let result6 = parse_contextual_query(input6);
    assert!(result6.is_err());

    let Err(errors6) = result6 else {
        panic!("Expected Err result");
    };

    assert!(!errors6.is_empty());
    for error in errors6 {
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input7 = "DELETE DOCUMENT IN users {id = 1}";
    let result7 = parse_contextual_query(input7);
    assert!(result7.is_err());

    let Err(errors7) = result7 else {
        panic!("Expected Err result");
    };

    assert!(!errors7.is_empty());
    for error in errors7 {
        assert!(error.context.contains(&"delete document".to_string()));
        assert!(error.expected.contains(&"FROM".to_string()));
    }
}

#[test]
fn invalid_field_structure() {
    let input1 = "DELETE DOC FROM users {id 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "DELETE DOC FROM users {id = 1";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.expected.contains(&"}".to_string()));
    }

    let input3 = "DELETE DOCUMENT FROM users {id = 1} EXTRA";
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
fn invalid_empty_query() {
    let input = "DELETE DOCUMENT FROM users {}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_assignments() {
    let input = "DELETE DOC FROM users {id = 1, name: 'John'}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_assignments_in_nested() {
    let input = "DELETE DOC FROM users {id = 1, address {city: 'NYC'}}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_malformed_nested() {
    let input1 = "DELETE DOC FROM users {address {city, country}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "DELETE DOC FROM users {address city, country}}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "DELETE DOC FROM users {address {{city}}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());
}

#[test]
fn invalid_empty_conditions() {
    let input1 = "DELETE DOC FROM users {id =}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "DELETE DOC FROM users {= 1}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());
}

#[test]
fn invalid_unsupported_operators() {
    let input1 = "DELETE DOC FROM users {id ~ 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "DELETE DOC FROM users {id & 1}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());
}
