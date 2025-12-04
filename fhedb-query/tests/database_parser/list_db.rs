use fhedb_query::prelude::{DatabaseQuery, parse_database_query};

#[test]
fn basic() {
    let input = "LIST DATABASES";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::List));
}

#[test]
fn case_insensitive() {
    let input = "LiSt DaTaBaSeS";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::List));
}

#[test]
fn with_extra_whitespace() {
    let input = "   LIST    DATABASES   ";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::List));
}

#[test]
fn invalid_missing_databases() {
    let input = "LIST";
    let result = parse_database_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"list databases".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
        assert!(error.expected.contains(&"DATABASES".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid list databases query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "LIST DATABASES EXTRA_STUFF";
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
fn invalid_wrong_order() {
    let input = "DATABASES LIST";
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
