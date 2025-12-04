use fhedb_query::prelude::{CollectionQuery, ContextualQuery, parse_contextual_query};

#[test]
fn basic() {
    let input = "GET SCHEMA FROM users";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::GetSchema { .. }));

    let CollectionQuery::GetSchema { name } = query else {
        panic!("Expected GetSchema variant");
    };

    assert_eq!(name, "users");
}

#[test]
fn case_insensitive() {
    let input = "GeT sChEmA fRoM MyCollection";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::GetSchema { .. }));

    let CollectionQuery::GetSchema { name } = query else {
        panic!("Expected GetSchema variant");
    };

    assert_eq!(name, "MyCollection");
}

#[test]
fn with_extra_whitespace() {
    let input = "   GET    SCHEMA    FROM    test_collection   ";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::GetSchema { .. }));

    let CollectionQuery::GetSchema { name } = query else {
        panic!("Expected GetSchema variant");
    };

    assert_eq!(name, "test_collection");
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.span.start == 0 && error.span.end == 0);
        assert!(error.found.is_none());
        assert!(error.message.to_lowercase().contains("unknown query"));
    }
}

#[test]
fn invalid_missing_schema() {
    let input = "GET";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"get collection schema".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"SCHEMA".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid get collection schema query")
        );
    }
}

#[test]
fn invalid_missing_from() {
    let input = "GET SCHEMA";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"get collection schema".to_string()));
        assert!(error.expected.contains(&"FROM".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid get collection schema query")
        );
    }
}

#[test]
fn invalid_missing_collection_name() {
    let input = "GET SCHEMA FROM";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"get collection schema".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid get collection schema query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "GET SCHEMA FROM users EXTRA_STUFF";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"end of input".to_string()));
        assert!(error.found == Some("EXTRA_STUFF".to_string()));
        assert!(error.message.to_lowercase().contains("unexpected input"));
    }
}

#[test]
fn invalid_wrong_keyword() {
    let input = "GET SCHEMAS FROM users";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"SCHEMA".to_string()));
        assert!(error.context.contains(&"get collection schema".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid get collection schema query")
        );
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "SCHEMA GET FROM users";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert_eq!(error.span.start, 0);
        assert!(error.message.to_lowercase().contains("unknown query"));
    }
}
