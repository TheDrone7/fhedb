use fhedb_query::prelude::parse_database_query;

#[test]
fn basic() {
    let result = parse_database_query("DROP DATABASE test_db");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Drop {
            name: "test_db".to_string(),
        }
    );
}

#[test]
fn case_insensitive() {
    let result = parse_database_query("DrOp DaTaBaSe MyDatabase");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Drop {
            name: "MyDatabase".to_string(),
        }
    );
}

#[test]
fn with_extra_whitespace() {
    let result = parse_database_query("   DROP    DATABASE    test_db   ");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Drop {
            name: "test_db".to_string(),
        }
    );
}

#[test]
fn invalid_empty() {
    let result = parse_database_query("");
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
    let result = parse_database_query("DROP DATABASE");
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
    let result = parse_database_query("DROP DATABASE test_db EXTRA_STUFF");
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
    let result = parse_database_query("DROP test_db");
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
    let result = parse_database_query("DATABASE DROP test_db");
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
