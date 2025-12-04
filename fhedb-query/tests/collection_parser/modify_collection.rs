use fhedb_core::db::schema::{FieldDefinition, FieldType};
use fhedb_query::ast::FieldModification;
use fhedb_query::prelude::{CollectionQuery, ContextualQuery, parse_contextual_query};

#[test]
fn basic() {
    let input = "MODIFY COLLECTION users {name: string, age: drop}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify {
        name,
        modifications,
    } = query
    else {
        panic!("Expected Modify variant");
    };

    assert_eq!(name, "users");
    assert_eq!(modifications.len(), 2);
    assert_eq!(
        modifications.get("name"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::String
        )))
    );
    assert_eq!(modifications.get("age"), Some(&FieldModification::Drop));

    let input = "ALTER COLLECTION products {price: float, old_field: drop}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify {
        name,
        modifications,
    } = query
    else {
        panic!("Expected Modify variant");
    };

    assert_eq!(name, "products");
    assert_eq!(modifications.len(), 2);
    assert_eq!(
        modifications.get("price"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Float
        )))
    );
    assert_eq!(
        modifications.get("old_field"),
        Some(&FieldModification::Drop)
    );
}

#[test]
fn case_insensitive() {
    let input = "MoDiFy CoLlEcTiOn MyCollection {FiElD: dROp}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify {
        name,
        modifications,
    } = query
    else {
        panic!("Expected Modify variant");
    };

    assert_eq!(name, "MyCollection");
    assert_eq!(modifications.len(), 1);
    assert_eq!(modifications.get("FiElD"), Some(&FieldModification::Drop));
}

#[test]
fn with_extra_whitespace() {
    let input = "   MODIFY    COLLECTION    test_collection   {field1: int, field2: drop}   ";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify {
        name,
        modifications,
    } = query
    else {
        panic!("Expected Modify variant");
    };

    assert_eq!(name, "test_collection");
    assert_eq!(modifications.len(), 2);
    assert_eq!(
        modifications.get("field1"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Int
        )))
    );
    assert_eq!(modifications.get("field2"), Some(&FieldModification::Drop));
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
    let input = "MODIFY COLLECTION";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"modify collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid modify collection query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "MODIFY COLLECTION test_collection {field: int} EXTRA_STUFF";
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
    let input = "MODIFY test_collection {field: int}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"COLLECTION".to_string()));
        assert!(error.context.contains(&"modify collection".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid modify collection query")
        );
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION MODIFY test_collection {field: int}";
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
