use fhedb_query::prelude::{CollectionQuery, ContextualQuery, parse_contextual_query};

#[test]
fn basic() {
    let input = "DROP COLLECTION test_collection";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Drop { .. }));

    let CollectionQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
    };

    assert_eq!(name, "test_collection");
}

#[test]
fn case_insensitive() {
    let input = "DrOp CoLlEcTiOn MyCollection";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Drop { .. }));

    let CollectionQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
    };

    assert_eq!(name, "MyCollection");
}

#[test]
fn with_extra_whitespace() {
    let input = "   DROP    COLLECTION    test_collection   ";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Drop { .. }));

    let CollectionQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
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
fn invalid_missing_name() {
    let input = "DROP COLLECTION";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"drop collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid drop collection query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "DROP COLLECTION test_collection EXTRA_STUFF";
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
fn invalid_no_keyword() {
    let input = "DROP test_collection";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"COLLECTION".to_string()));
        assert!(error.context.contains(&"drop collection".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid drop collection query")
        );
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION DROP test_collection";
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
