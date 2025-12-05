use fhedb_query::prelude::{
    ContextualQuery, DocumentQuery, FieldSelector, QueryOperator, parse_contextual_query,
};

#[test]
fn basic() {
    let input = "GET DOCUMENT FROM users {id = 1, name}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DocumentQuery::Get { .. }));

    let DocumentQuery::Get {
        collection_name,
        conditions,
        selectors,
    } = query
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(collection_name, "users");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[0].operator, QueryOperator::Equal);
    assert_eq!(conditions[0].value, "1");
    assert_eq!(selectors.len(), 1);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "name"));
}

#[test]
fn variations() {
    let input1 = "GET DOC FROM products {id = \"prod_123\", price}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        collection_name: name1,
        conditions: conds1,
        selectors: sels1,
    } = query1
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(name1, "products");
    assert_eq!(conds1.len(), 1);
    assert_eq!(conds1[0].field_name, "id");
    assert_eq!(conds1[0].value, "\"prod_123\"");
    assert_eq!(sels1.len(), 1);
    assert!(matches!(&sels1[0], FieldSelector::Field(name) if name == "price"));

    let input2 = "   GeT    DoCs    fRoM    MyCollection   {status = 'active', name}   ";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        collection_name: name2,
        conditions: conds2,
        selectors: sels2,
    } = query2
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(name2, "MyCollection");
    assert_eq!(conds2.len(), 1);
    assert_eq!(conds2[0].field_name, "status");
    assert_eq!(conds2[0].value, "\"active\"");
    assert_eq!(sels2.len(), 1);

    let input3 = "GET DOCUMENTS FROM test_collection {*}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_ok());

    let Ok(ContextualQuery::Document(query3)) = result3 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        collection_name: name3,
        conditions: conds3,
        selectors: sels3,
    } = query3
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(name3, "test_collection");
    assert_eq!(conds3.len(), 0);
    assert_eq!(sels3.len(), 1);
    assert!(matches!(sels3[0], FieldSelector::AllFields));
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
        bio == 'developer',
        username
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        collection_name,
        conditions,
        selectors,
    } = query
    else {
        panic!("Expected Get variant");
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

    assert_eq!(selectors.len(), 1);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "username"));
}

#[test]
fn wildcard_selectors() {
    let input1 = "GET DOCUMENT FROM users {id = 1, *}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Document(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        selectors: sels1, ..
    } = query1
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(sels1.len(), 1);
    assert!(matches!(sels1[0], FieldSelector::AllFields));

    let input2 = "GET DOCUMENT FROM users {id = 1, **}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Document(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        selectors: sels2, ..
    } = query2
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(sels2.len(), 1);
    assert!(matches!(sels2[0], FieldSelector::AllFieldsRecursive));
}

#[test]
fn simple_nested() {
    let input = "GET DOCUMENT FROM users {id = 1, address {city, country}}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        collection_name,
        conditions,
        selectors,
    } = query
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(collection_name, "users");
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
    let input = "GET DOCUMENT FROM users {
        id = 1,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        selectors,
        conditions,
        ..
    } = query
    else {
        panic!("Expected Get variant");
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
fn deep_nesting() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        profile {
            name,
            address {
                city = 'NYC',
                location {
                    lat > 40.0,
                    lng < -73.0,
                    details {*}
                }
            }
        }
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get { selectors, .. } = query else {
        panic!("Expected Get variant");
    };

    assert_eq!(selectors.len(), 1);

    let FieldSelector::SubDocument {
        field_name: profile_name,
        content: profile_content,
    } = &selectors[0]
    else {
        panic!("Expected SubDocument selector");
    };

    assert_eq!(profile_name, "profile");
    assert_eq!(profile_content.selectors.len(), 2);

    let FieldSelector::SubDocument {
        field_name: address_name,
        content: address_content,
    } = &profile_content.selectors[1]
    else {
        panic!("Expected SubDocument selector for address");
    };

    assert_eq!(address_name, "address");
    assert_eq!(address_content.conditions.len(), 1);
    assert_eq!(address_content.selectors.len(), 1);

    let FieldSelector::SubDocument {
        field_name: location_name,
        content: location_content,
    } = &address_content.selectors[0]
    else {
        panic!("Expected SubDocument selector for location");
    };

    assert_eq!(location_name, "location");
    assert_eq!(location_content.conditions.len(), 2);
    assert_eq!(location_content.selectors.len(), 1);

    let FieldSelector::SubDocument {
        field_name: details_name,
        content: details_content,
    } = &location_content.selectors[0]
    else {
        panic!("Expected SubDocument selector for details");
    };

    assert_eq!(details_name, "details");
    assert_eq!(details_content.selectors.len(), 1);
    assert!(matches!(
        details_content.selectors[0],
        FieldSelector::AllFields
    ));
}

#[test]
fn nested_wildcards() {
    let input = "GET DOCUMENT FROM users {
        id = 1,
        profile {
            *,
            address {**}
        }
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get { selectors, .. } = query else {
        panic!("Expected Get variant");
    };

    let FieldSelector::SubDocument {
        content: profile_content,
        ..
    } = &selectors[0]
    else {
        panic!("Expected SubDocument selector");
    };

    assert_eq!(profile_content.selectors.len(), 2);
    assert!(matches!(
        profile_content.selectors[0],
        FieldSelector::AllFields
    ));

    let FieldSelector::SubDocument {
        content: address_content,
        ..
    } = &profile_content.selectors[1]
    else {
        panic!("Expected SubDocument selector for address");
    };

    assert_eq!(address_content.selectors.len(), 1);
    assert!(matches!(
        address_content.selectors[0],
        FieldSelector::AllFieldsRecursive
    ));
}

#[test]
fn mixed_syntax() {
    let input = "GET DOCUMENT FROM users {
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

    let DocumentQuery::Get {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Get variant");
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
fn empty_braces() {
    let input = "GET DOCUMENT FROM users {id = 1, address {}}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get { selectors, .. } = query else {
        panic!("Expected Get variant");
    };

    let FieldSelector::SubDocument { content, .. } = &selectors[0] else {
        panic!("Expected SubDocument selector");
    };

    assert!(content.conditions.is_empty());
    assert!(content.selectors.is_empty());
    assert!(content.assignments.is_empty());
}

#[test]
fn only_conditions() {
    let input = "GET DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(conditions.len(), 3);
    assert_eq!(conditions[0].field_name, "id");
    assert_eq!(conditions[1].field_name, "age");
    assert_eq!(conditions[2].field_name, "status");
    assert!(selectors.is_empty());
}

#[test]
fn only_selectors() {
    let input = "GET DOCUMENT FROM users {name, email, age}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Get variant");
    };

    assert!(conditions.is_empty());
    assert_eq!(selectors.len(), 3);
    assert!(matches!(&selectors[0], FieldSelector::Field(name) if name == "name"));
    assert!(matches!(&selectors[1], FieldSelector::Field(name) if name == "email"));
    assert!(matches!(&selectors[2], FieldSelector::Field(name) if name == "age"));
}

#[test]
fn complex_values() {
    let input = "GET DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\Users\\Admin\",
        matrix = [[1, 2], [3, 4]],
        name,
        price
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Document(query)) = result else {
        panic!("Expected Ok result");
    };

    let DocumentQuery::Get {
        conditions,
        selectors,
        ..
    } = query
    else {
        panic!("Expected Get variant");
    };

    assert_eq!(conditions.len(), 4);
    assert_eq!(conditions[0].field_name, "tags");
    assert_eq!(conditions[0].operator, QueryOperator::Similar);
    assert_eq!(conditions[1].field_name, "config");
    assert_eq!(conditions[2].field_name, "path");
    assert_eq!(conditions[2].operator, QueryOperator::NotEqual);
    assert_eq!(conditions[3].field_name, "matrix");
    assert_eq!(conditions[3].value, "[[1, 2], [3, 4]]");

    assert_eq!(selectors.len(), 2);
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

    let input2 = "GET";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "GET DOCUMENT";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());

    let Err(errors3) = result3 else {
        panic!("Expected Err result");
    };

    assert!(!errors3.is_empty());
    for error in errors3 {
        assert!(error.context.contains(&"get document".to_string()));
        assert!(error.expected.contains(&"FROM".to_string()));
    }

    let input4 = "GET DOCUMENT FROM";
    let result4 = parse_contextual_query(input4);
    assert!(result4.is_err());

    let Err(errors4) = result4 else {
        panic!("Expected Err result");
    };

    assert!(!errors4.is_empty());
    for error in errors4 {
        assert!(error.context.contains(&"get document".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
    }

    let input5 = "GET DOC FROM users {*}";
    let result5 = parse_contextual_query(input5);
    assert!(result5.is_ok());

    let input6 = "DOCUMENT GET FROM users {id = 1}";
    let result6 = parse_contextual_query(input6);
    assert!(result6.is_err());

    let Err(errors6) = result6 else {
        panic!("Expected Err result");
    };

    assert!(!errors6.is_empty());
    for error in errors6 {
        assert!(error.message.to_lowercase().contains("unknown query"));
    }

    let input7 = "GET DOCUMENT IN users {id = 1}";
    let result7 = parse_contextual_query(input7);
    assert!(result7.is_err());
}

#[test]
fn invalid_field_structure() {
    let input1 = "GET DOC FROM users {id 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "GET DOC FROM users {id = 1";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let Err(errors2) = result2 else {
        panic!("Expected Err result");
    };

    assert!(!errors2.is_empty());
    for error in errors2 {
        assert!(error.expected.contains(&"}".to_string()));
    }

    let input3 = "GET DOCUMENT FROM users {id = 1} EXTRA";
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
fn invalid_assignments_in_get() {
    let input = "GET DOC FROM users {id = 1, name: 'John'}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_assignments_in_nested() {
    let input = "GET DOC FROM users {id = 1, address {city: 'NYC'}}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_empty_body() {
    let input = "GET DOC FROM users {}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_no_body() {
    let input = "GET DOC FROM users";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_malformed_nested() {
    let input1 = "GET DOC FROM users {address {city, country}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "GET DOC FROM users {address city, country}}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());

    let input3 = "GET DOC FROM users {address {{city}}";
    let result3 = parse_contextual_query(input3);
    assert!(result3.is_err());
}

#[test]
fn invalid_empty_conditions() {
    let input1 = "GET DOC FROM users {id =}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "GET DOC FROM users {= 1}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());
}

#[test]
fn invalid_unsupported_operators() {
    let input1 = "GET DOC FROM users {id ~ 1}";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_err());

    let input2 = "GET DOC FROM users {id & 1}";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_err());
}
