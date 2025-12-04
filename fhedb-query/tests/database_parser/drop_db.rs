use fhedb_query::prelude::{DatabaseQuery, parse_database_query};

#[test]
fn basic() {
    let input = "DROP DATABASE test_db";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Drop { .. }));

    let DatabaseQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
    };

    assert_eq!(name, "test_db");
}

#[test]
fn case_insensitive() {
    let input = "DrOp DaTaBaSe MyDatabase";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Drop { .. }));

    let DatabaseQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
    };

    assert_eq!(name, "MyDatabase");
}

#[test]
fn with_extra_whitespace() {
    let input = "   DROP    DATABASE    test_db   ";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Drop { .. }));

    let DatabaseQuery::Drop { name } = query else {
        panic!("Expected Drop variant");
    };

    assert_eq!(name, "test_db");
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_database_query(input);
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
    let input = "DROP DATABASE";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"drop database".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
        assert!(error.expected.contains(&"database name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid drop database query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "DROP DATABASE test_db EXTRA_STUFF";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"end of input".to_string()));
        assert!(error.found == Some("EXTRA_STUFF".to_string()));
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid drop database query")
        );
        assert!(error.expected.contains(&"DATABASE".to_string()));
        assert!(error.context.contains(&"drop database".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASE DROP test_db";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.message.to_lowercase().contains("unknown query"));
        assert_eq!(error.span.start, 0);
    }
}
