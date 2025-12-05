use fhedb_core::prelude::FieldType;
use fhedb_query::prelude::{CollectionQuery, ContextualQuery, parse_contextual_query};

#[test]
fn basic() {
    let input = "CREATE COLLECTION users {id: id_int, name: string, age: int}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));

    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "users");
    assert!(!drop_if_exists);
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
}

#[test]
fn case_insensitive() {
    let input = "CrEaTe CoLlEcTiOn MyCollection {id: iD_stRiNg, title: string}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "MyCollection");
    assert!(!drop_if_exists);
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdString);
    assert_eq!(schema.fields["title"].field_type, FieldType::String);
}

#[test]
fn with_drop_if_exists() {
    let input = "CREATE COLLECTION test_collection DROP IF EXISTS {id: id_int, data: string}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "test_collection");
    assert!(drop_if_exists);
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema.fields["data"].field_type, FieldType::String);
}

#[test]
fn with_extra_whitespace() {
    let input1 = "   CREATE    COLLECTION    test_collection   {id: id_int, name: string}   ";
    let result1 = parse_contextual_query(input1);
    assert!(result1.is_ok());

    let Ok(ContextualQuery::Collection(query1)) = result1 else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query1, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name: name1,
        drop_if_exists: drop_if_exists1,
        schema: schema1,
    } = query1
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name1, "test_collection");
    assert!(!drop_if_exists1);
    assert_eq!(schema1.fields.len(), 2);
    assert_eq!(schema1.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema1.fields["name"].field_type, FieldType::String);

    let input2 =
        "   CREATE    COLLECTION    test_collection    DROP   IF   EXISTS   {id: id_string}   ";
    let result2 = parse_contextual_query(input2);
    assert!(result2.is_ok());

    let Ok(ContextualQuery::Collection(query2)) = result2 else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query2, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name: name2,
        drop_if_exists: drop_if_exists2,
        schema: schema2,
    } = query2
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name2, "test_collection");
    assert!(drop_if_exists2);
    assert_eq!(schema2.fields.len(), 1);
    assert_eq!(schema2.fields["id"].field_type, FieldType::IdString);
}

#[test]
fn complex_schema() {
    let input = "CREATE COLLECTION products {
        id: id_string,
        name: string (nullable, default = \"Unnamed\"),
        price: float,
        in_stock: boolean(default = true),
        tags: array<string>,
        category_ref: ref<categories>(nullable)
    }";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());
    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "products");
    assert!(!drop_if_exists);
    assert_eq!(schema.fields.len(), 6);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdString);
    assert_eq!(
        schema.fields["name"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["name"].default_value,
        Some(bson::Bson::String("Unnamed".to_string()))
    );
    assert_eq!(schema.fields["price"].field_type, FieldType::Float);
    assert_eq!(schema.fields["in_stock"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["in_stock"].default_value,
        Some(bson::Bson::Boolean(true))
    );
    assert_eq!(
        schema.fields["tags"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["category_ref"].field_type,
        FieldType::Nullable(Box::new(FieldType::Reference("categories".to_string())))
    );
}

#[test]
fn nested_braces_in_strings() {
    let input = "CREATE COLLECTION test {
        id: id_int,
        config: string(default = \"{\\\"key\\\": {\\\"nested\\\": \\\"value\\\"}}\"),
        template: string(default = \"Hello {name}, welcome to {place}!\")
    }";
    let result = parse_contextual_query(input);

    assert!(result.is_ok());
    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "test");
    assert!(!drop_if_exists);
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema.fields["config"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["config"].default_value,
        Some(bson::Bson::String(
            "{\"key\": {\"nested\": \"value\"}}".to_string()
        ))
    );
    assert_eq!(schema.fields["template"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["template"].default_value,
        Some(bson::Bson::String(
            "Hello {name}, welcome to {place}!".to_string()
        ))
    );
}

#[test]
fn empty_schema() {
    let input = "CREATE COLLECTION empty_collection {}";
    let result = parse_contextual_query(input);
    assert!(result.is_ok());

    let Ok(ContextualQuery::Collection(query)) = result else {
        panic!("Expected Ok result");
    };

    assert!(matches!(query, CollectionQuery::Create { .. }));
    let CollectionQuery::Create {
        name,
        drop_if_exists,
        schema,
    } = query
    else {
        panic!("Expected Create variant");
    };

    assert_eq!(name, "empty_collection");
    assert!(!drop_if_exists);
    assert_eq!(schema.fields.len(), 0);
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
    let input = "CREATE COLLECTION";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"create collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"collection name".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid create collection query")
        );
    }
}

#[test]
fn invalid_missing_schema() {
    let input = "CREATE COLLECTION test_collection";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.context.contains(&"create collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"schema".to_string()));
        assert!(
            error
                .message
                .to_lowercase()
                .contains("invalid create collection query")
        );
    }
}

#[test]
fn invalid_extra_input() {
    let input = "CREATE COLLECTION test_collection {id: id_int} EXTRA_STUFF";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.expected.contains(&"end of input".to_string()));
        assert!(error.message == "Unexpected input after query");
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "CREATE test_collection {id: id_int}";
    let result = parse_contextual_query(input);
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
                .contains("invalid create collection query")
        );
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.context.contains(&"create collection".to_string()));
        assert!(error.expected.contains(&"COLLECTION".to_string()));
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION CREATE test_collection {id: id_int}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.message.to_lowercase().contains("unknown query"));
        assert!(error.span.start == 0);
    }
}

#[test]
fn invalid_malformed_schema() {
    let input = "CREATE COLLECTION test_collection {id id_int}";
    let result = parse_contextual_query(input);
    assert!(result.is_err());

    let Err(errors) = result else {
        panic!("Expected Err result");
    };

    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.message.to_lowercase().contains("invalid field type"));
        assert!(error.context.contains(&"create collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.context.contains(&"schema".to_string()));
        assert!(error.context.contains(&"field type".to_string()));
        assert!(error.expected.contains(&":".to_string()));
    }
}

#[test]
fn invalid_missing_braces() {
    let input = "CREATE COLLECTION test_collection id: id_int";
    let result = parse_contextual_query(input);
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
                .contains("invalid create collection query")
        );
        assert!(error.context.contains(&"create collection".to_string()));
        assert!(error.context.contains(&"collection query".to_string()));
        assert!(error.expected.contains(&"schema".to_string()));
        assert!(error.expected.contains(&"DROP".to_string()));
    }
}
