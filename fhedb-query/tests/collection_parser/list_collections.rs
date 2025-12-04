use fhedb_query::prelude::parse_contextual_query;

#[test]
fn basic() {
    let result = parse_contextual_query("LIST COLLECTIONS");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(fhedb_query::ast::CollectionQuery::List)
    );
}

#[test]
fn case_insensitive() {
    let result = parse_contextual_query("LiSt CoLlEcTiOnS");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(fhedb_query::ast::CollectionQuery::List)
    );
}

#[test]
fn with_extra_whitespace() {
    let result = parse_contextual_query("   LIST    COLLECTIONS   ");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::ContextualQuery::Collection(fhedb_query::ast::CollectionQuery::List)
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
fn invalid_missing_collections() {
    let result = parse_contextual_query("LIST");
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"list collections".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"COLLECTIONS".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid list collections query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let result = parse_contextual_query("LIST COLLECTIONS EXTRA_STUFF");
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
    let result = parse_contextual_query("LIST COLLECTION");
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"COLLECTIONS".to_string()));
        assert!(error.context.contains(&"list collections".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid list collections query")
        );
    }
}

#[test]
fn invalid_wrong_order() {
    let result = parse_contextual_query("COLLECTIONS LIST");
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
