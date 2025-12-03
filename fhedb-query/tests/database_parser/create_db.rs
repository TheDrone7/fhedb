use fhedb_query::prelude::parse_database_query;

#[test]
fn basic() {
    let result = parse_database_query("CREATE DATABASE test_db");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Create {
            name: "test_db".to_string(),
            drop_if_exists: false,
        }
    );
}

#[test]
fn case_insensitive() {
    let result = parse_database_query("CrEaTe DaTaBaSe MyDatabase");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Create {
            name: "MyDatabase".to_string(),
            drop_if_exists: false,
        }
    );
}

#[test]
fn with_drop_if_exists() {
    let result = parse_database_query("CREATE DATABASE test_db DROP IF EXISTS");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Create {
            name: "test_db".to_string(),
            drop_if_exists: true,
        }
    );
}

#[test]
fn with_extra_whitespace() {
    let result = parse_database_query("   CREATE    DATABASE    test_db   ");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Create {
            name: "test_db".to_string(),
            drop_if_exists: false,
        }
    );

    let result = parse_database_query("   CREATE    DATABASE    test_db    DROP   IF   EXISTS   ");
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert_eq!(
        query,
        fhedb_query::ast::DatabaseQuery::Create {
            name: "test_db".to_string(),
            drop_if_exists: true,
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
    let result = parse_database_query("CREATE DATABASE");
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"create database".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
        assert!(error.expected.contains(&"database name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid create database query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let result = parse_database_query("CREATE DATABASE test_db EXTRA_STUFF");
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"database query".to_string()));
        assert!(error.context.contains(&"create database".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid create database query")
        );
        assert!(error.expected.contains(&"end of input".to_string()));
    }
}

#[test]
fn invalid_no_keyword() {
    let result = parse_database_query("CREATE test_db");
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
                .contains("invalid create database query")
        );
        assert!(error.expected.contains(&"DATABASE".to_string()));
        assert!(error.context.contains(&"create database".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
    }
}

#[test]
fn invalid_wrong_order() {
    let result = parse_database_query("DATABASE CREATE test_db");
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
