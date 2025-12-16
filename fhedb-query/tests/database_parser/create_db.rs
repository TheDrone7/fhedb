use fhedb_query::prelude::parse_database_query;
use fhedb_types::DatabaseQuery;

#[test]
fn basic() {
    let input = "CREATE DATABASE test_db";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Create { .. }));

    let DatabaseQuery::Create {
        name,
        drop_if_exists,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "test_db");
    assert!(!drop_if_exists);
}

#[test]
fn case_insensitive() {
    let input = "CrEaTe DaTaBaSe MyDatabase";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Create { .. }));

    let DatabaseQuery::Create {
        name,
        drop_if_exists,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "MyDatabase");
    assert!(!drop_if_exists);
}

#[test]
fn with_drop_if_exists() {
    let input = "CREATE DATABASE test_db DROP IF EXISTS";
    let result = parse_database_query(input);
    assert!(result.is_ok());

    let Ok(query) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, DatabaseQuery::Create { .. }));

    let DatabaseQuery::Create {
        name,
        drop_if_exists,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "test_db");
    assert!(drop_if_exists);
}

#[test]
fn with_extra_whitespace() {
    let input1 = "   CREATE    DATABASE    test_db   ";
    let result1 = parse_database_query(input1);
    assert!(result1.is_ok());

    let Ok(query1) = result1 else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query1, DatabaseQuery::Create { .. }));

    let DatabaseQuery::Create {
        name: name1,
        drop_if_exists: drop_if_exists1,
    } = query1
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name1, "test_db");
    assert!(!drop_if_exists1);

    let input2 = "   CREATE    DATABASE    test_db    DROP   IF   EXISTS   ";
    let result2 = parse_database_query(input2);
    assert!(result2.is_ok());

    let Ok(query2) = result2 else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query2, DatabaseQuery::Create { .. }));

    let DatabaseQuery::Create {
        name: name2,
        drop_if_exists: drop_if_exists2,
    } = query2
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name2, "test_db");
    assert!(drop_if_exists2);
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
    let input = "CREATE DATABASE";
    let result = parse_database_query(input);
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
    let input = "CREATE DATABASE test_db EXTRA_STUFF";
    let result = parse_database_query(input);
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
    let input = "CREATE test_db";
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
                .contains("invalid create database query")
        );
        assert!(error.expected.contains(&"DATABASE".to_string()));
        assert!(error.context.contains(&"create database".to_string()));
        assert!(error.context.contains(&"database query".to_string()));
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "DATABASE CREATE test_db";
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
