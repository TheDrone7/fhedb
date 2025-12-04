use fhedb_query::prelude::parse_contextual_query;

#[test]
fn basic() {
    let result = parse_contextual_query("GET SCHEMA FROM users");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(
            fhedb_query::ast::CollectionQuery::GetSchema {
                name: "users".to_string(),
            }
        )
    );
}

#[test]
fn case_insensitive() {
    let result = parse_contextual_query("GeT sChEmA fRoM MyCollection");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(
            fhedb_query::ast::CollectionQuery::GetSchema {
                name: "MyCollection".to_string(),
            }
        )
    );
}

#[test]
fn with_extra_whitespace() {
    let result = parse_contextual_query("   GET    SCHEMA    FROM    test_collection   ");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(
            fhedb_query::ast::CollectionQuery::GetSchema {
                name: "test_collection".to_string(),
            }
        )
    );
}

#[test]
fn invalid_empty() {
    let result = parse_contextual_query("");
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
    let result = parse_contextual_query("GET");
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
    let result = parse_contextual_query("GET SCHEMA");
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
    let result = parse_contextual_query("GET SCHEMA FROM");
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
    let result = parse_contextual_query("GET SCHEMA FROM users EXTRA_STUFF");
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
    let result = parse_contextual_query("GET SCHEMAS FROM users");
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
    let result = parse_contextual_query("SCHEMA GET FROM users");
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
