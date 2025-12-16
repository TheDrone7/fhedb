use fhedb_query::prelude::parse_contextual_query;
use fhedb_types::{CollectionQuery, ContextualQuery};

#[test]
fn basic() {
    let input = "LIST COLLECTIONS";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::List));
}

#[test]
fn case_insensitive() {
    let input = "LiSt CoLlEcTiOnS";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::List));
}

#[test]
fn with_extra_whitespace() {
    let input = "   LIST    COLLECTIONS   ";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::List));
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
fn invalid_missing_collections() {
    let input = "LIST";
    let result = parse_contextual_query(input);
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
    let input = "LIST COLLECTIONS EXTRA_STUFF";
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
    let input = "LIST COLLECTION";
    let result = parse_contextual_query(input);
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
    let input = "COLLECTIONS LIST";
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
