use fhedb_core::db::schema::{FieldDefinition, FieldType};
use fhedb_query::prelude::parse_contextual_query;
use fhedb_types::{CollectionQuery, ContextualQuery, FieldModification};

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

#[test]
fn all_field_types() {
    let input = "MODIFY COLLECTION test {a: int, b: float, c: string, d: boolean}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(modifications.len(), 4);
    assert_eq!(
        modifications.get("a"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Int
        )))
    );
    assert_eq!(
        modifications.get("b"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Float
        )))
    );
    assert_eq!(
        modifications.get("c"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::String
        )))
    );
    assert_eq!(
        modifications.get("d"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Boolean
        )))
    );
}

#[test]
fn id_types() {
    let input = "MODIFY COLLECTION test {a: id_int, b: id_string}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("a"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::IdInt
        )))
    );
    assert_eq!(
        modifications.get("b"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::IdString
        )))
    );
}

#[test]
fn array_types() {
    let input = "MODIFY COLLECTION test {tags: array<string>, numbers: array<int>}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("tags"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Array(Box::new(FieldType::String))
        )))
    );
    assert_eq!(
        modifications.get("numbers"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Array(Box::new(FieldType::Int))
        )))
    );
}

#[test]
fn nested_array_types() {
    let input = "MODIFY COLLECTION test {matrix: array<array<int>>}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("matrix"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int))))
        )))
    );
}

#[test]
fn reference_types() {
    let input = "MODIFY COLLECTION test {user_ref: ref<users>, company_ref: ref<companies>}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("user_ref"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Reference("users".to_string())
        )))
    );
    assert_eq!(
        modifications.get("company_ref"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Reference("companies".to_string())
        )))
    );
}

#[test]
fn nullable_modifier() {
    let input = "MODIFY COLLECTION test {name: string(nullable), age: int(nullable)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("name"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Nullable(Box::new(FieldType::String))
        )))
    );
    assert_eq!(
        modifications.get("age"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Nullable(Box::new(FieldType::Int))
        )))
    );
}

#[test]
fn nullable_with_complex_types() {
    let input =
        "MODIFY COLLECTION test {items: array<string>(nullable), owner: ref<users>(nullable)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("items"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Nullable(Box::new(FieldType::Array(Box::new(FieldType::String))))
        )))
    );
    assert_eq!(
        modifications.get("owner"),
        Some(&FieldModification::Set(FieldDefinition::new(
            FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())))
        )))
    );
}

#[test]
fn default_modifier() {
    let input =
        "MODIFY COLLECTION test {name: string(default = \"Unknown\"), count: int(default = 0)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("name"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::String,
            bson::Bson::String("Unknown".to_string())
        )))
    );
    assert_eq!(
        modifications.get("count"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::Int,
            bson::Bson::Int64(0)
        )))
    );
}

#[test]
fn default_modifier_other_types() {
    let input =
        "MODIFY COLLECTION test {active: boolean(default = true), score: float(default = 0.0)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("active"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::Boolean,
            bson::Bson::Boolean(true)
        )))
    );
    assert_eq!(
        modifications.get("score"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::Float,
            bson::Bson::Double(0.0)
        )))
    );
}

#[test]
fn nullable_and_default_combined() {
    let input = "MODIFY COLLECTION test {name: string(nullable, default = null)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("name"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::Nullable(Box::new(FieldType::String)),
            bson::Bson::Null
        )))
    );
}

#[test]
fn nullable_with_non_null_default() {
    let input = "MODIFY COLLECTION test {count: int(nullable, default = 42)}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(
        modifications.get("count"),
        Some(&FieldModification::Set(FieldDefinition::with_default(
            FieldType::Nullable(Box::new(FieldType::Int)),
            bson::Bson::Int64(42)
        )))
    );
}

#[test]
fn empty_modification_schema() {
    let input = "MODIFY COLLECTION test {}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Modify { .. }));

    let CollectionQuery::Modify { modifications, .. } = query else {
        panic!("Expected Modify variant");
    };

    assert_eq!(modifications.len(), 0);
}

#[test]
fn trailing_commas() {
    let input = "MODIFY COLLECTION test {name: string,}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let input = "MODIFY COLLECTION test {name: string, age: int,}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let input = "MODIFY COLLECTION test {field: drop,}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());
}

#[test]
fn duplicate_field_names() {
    let input = "MODIFY COLLECTION test {name: string, name: int}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input = "MODIFY COLLECTION test {field: drop, field: string}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let input = "MODIFY COLLECTION test {a: int, b: string, a: drop}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}

#[test]
fn invalid_missing_modification_schema() {
    let input = "MODIFY COLLECTION test";
    let result = parse_contextual_query(input);
    assert!(result.is_err());
}
