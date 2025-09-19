use fhedb_core::prelude::FieldType;
use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "CREATE COLLECTION users {id: id_int, name: string, age: int}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "users");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 3);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
            assert_eq!(schema.fields["name"].field_type, FieldType::String);
            assert_eq!(schema.fields["age"].field_type, FieldType::Int);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn case_insensitive() {
    let input = "CrEaTe CoLlEcTiOn MyCollection {id: iD_stRiNg, title: string}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "MyCollection");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 2);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdString);
            assert_eq!(schema.fields["title"].field_type, FieldType::String);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn with_drop_if_exists() {
    let input = "CREATE COLLECTION test_collection DROP IF EXISTS {id: id_int, data: string}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "test_collection");
            assert_eq!(drop_if_exists, true);
            assert_eq!(schema.fields.len(), 2);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
            assert_eq!(schema.fields["data"].field_type, FieldType::String);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn with_extra_whitespace() {
    let input = "   CREATE    COLLECTION    test_collection   {id: id_int, name: string}   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "test_collection");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 2);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
            assert_eq!(schema.fields["name"].field_type, FieldType::String);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }

    let input =
        "   CREATE    COLLECTION    test_collection    DROP   IF   EXISTS   {id: id_string}   ";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "test_collection");
            assert_eq!(drop_if_exists, true);
            assert_eq!(schema.fields.len(), 1);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdString);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn complex_schema() {
    let input = "CREATE COLLECTION products {
        id: id_string,
        name: string(default = \"Unnamed\"),
        price: float,
        in_stock: boolean(default = true),
        tags: array<string>,
        category_ref: ref<categories>(nullable)
    }";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "products");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 6);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdString);
            assert_eq!(schema.fields["name"].field_type, FieldType::String);
            assert_eq!(schema.fields["price"].field_type, FieldType::Float);
            assert_eq!(schema.fields["in_stock"].field_type, FieldType::Boolean);
            assert_eq!(
                schema.fields["tags"].field_type,
                FieldType::Array(Box::new(FieldType::String))
            );
            assert_eq!(
                schema.fields["category_ref"].field_type,
                FieldType::Nullable(Box::new(FieldType::Reference("categories".to_string())))
            );
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn nested_braces_in_strings() {
    let input = "CREATE COLLECTION test {
        id: id_int,
        config: string(default = \"{\\\"key\\\": {\\\"nested\\\": \\\"value\\\"}}\"),
        template: string(default = \"Hello {name}, welcome to {place}!\")
    }";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "test");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 3);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
            assert_eq!(schema.fields["config"].field_type, FieldType::String);
            assert_eq!(schema.fields["template"].field_type, FieldType::String);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn empty_schema() {
    let input = "CREATE COLLECTION empty_collection {}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "empty_collection");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 0);
        }
        _ => panic!("Expected CollectionQuery::Create, got {:?}", result),
    }
}

#[test]
fn invalid_empty() {
    let input = "";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_missing_name() {
    let input = "CREATE COLLECTION";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"CREATE COLLECTION\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_missing_schema() {
    let input = "CREATE COLLECTION test_collection";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"CREATE COLLECTION test_collection\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_extra_input() {
    let input = "CREATE COLLECTION test_collection {id: id_int} EXTRA_STUFF";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(message, "Unexpected input after collection query");
        }
    }
}

#[test]
fn invalid_no_keyword() {
    let input = "CREATE test_collection {id: id_int}";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"CREATE test_collection {id: id_int}\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_wrong_order() {
    let input = "COLLECTION CREATE test_collection {id: id_int}";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"COLLECTION CREATE test_collection {id: id_int}\", code: Tag }"
            );
        }
    }
}

#[test]
fn invalid_malformed_schema() {
    let input = "CREATE COLLECTION test_collection {id: id_int,}";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { .. } => {
            // TODO: Add specific message checks
        }
    }
}

#[test]
fn invalid_missing_braces() {
    let input = "CREATE COLLECTION test_collection id: id_int";
    let result = parse_collection_query(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::SyntaxError { message } => {
            assert_eq!(
                message,
                "Failed to parse collection query: Parsing Error: Error { input: \"CREATE COLLECTION test_collection id: id_int\", code: Tag }"
            );
        }
    }
}
