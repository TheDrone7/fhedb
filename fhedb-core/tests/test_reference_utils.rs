use fhedb_core::prelude::{
    Database, FieldDefinition, FieldType, ReferenceChecker, Schema, SchemaReferenceValidator,
};
use tempfile::TempDir;

fn schema(fields: Vec<(&str, FieldType)>) -> Schema {
    Schema {
        fields: fields
            .into_iter()
            .map(|(n, t)| (n.to_string(), FieldDefinition::new(t)))
            .collect(),
    }
}

fn setup_db(collections: Vec<(&str, Vec<(&str, FieldType)>)>) -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());
    for (name, fields) in collections {
        db.create_collection(name, schema(fields)).unwrap();
    }
    (temp_dir, db)
}

#[test]
fn contains_reference_direct() {
    assert!(FieldType::Reference("users".to_string()).contains_reference());
}

#[test]
fn contains_reference_primitives() {
    assert!(!FieldType::Int.contains_reference());
    assert!(!FieldType::Float.contains_reference());
    assert!(!FieldType::Boolean.contains_reference());
    assert!(!FieldType::String.contains_reference());
    assert!(!FieldType::IdString.contains_reference());
    assert!(!FieldType::IdInt.contains_reference());
}

#[test]
fn contains_reference_nullable() {
    let with_ref = FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())));
    let without_ref = FieldType::Nullable(Box::new(FieldType::String));
    assert!(with_ref.contains_reference());
    assert!(!without_ref.contains_reference());
}

#[test]
fn contains_reference_array() {
    let with_ref = FieldType::Array(Box::new(FieldType::Reference("users".to_string())));
    let without_ref = FieldType::Array(Box::new(FieldType::Int));
    assert!(with_ref.contains_reference());
    assert!(!without_ref.contains_reference());
}

#[test]
fn contains_reference_nested() {
    let nested_ref = FieldType::Array(Box::new(FieldType::Nullable(Box::new(
        FieldType::Reference("users".to_string()),
    ))));
    let nested_no_ref =
        FieldType::Nullable(Box::new(FieldType::Array(Box::new(FieldType::String))));
    assert!(nested_ref.contains_reference());
    assert!(!nested_no_ref.contains_reference());
}

#[test]
fn references_collection_match() {
    let ft = FieldType::Reference("users".to_string());
    assert!(ft.references_collection("users"));
    assert!(!ft.references_collection("posts"));
}

#[test]
fn references_collection_primitives() {
    assert!(!FieldType::Int.references_collection("users"));
    assert!(!FieldType::String.references_collection("users"));
}

#[test]
fn references_collection_nested() {
    let nullable_ref = FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())));
    let array_ref = FieldType::Array(Box::new(FieldType::Reference("posts".to_string())));
    assert!(nullable_ref.references_collection("users"));
    assert!(!nullable_ref.references_collection("posts"));
    assert!(array_ref.references_collection("posts"));
    assert!(!array_ref.references_collection("users"));
}

#[test]
fn find_invalid_reference_valid() {
    let (_temp, db) = setup_db(vec![("users", vec![])]);
    let ft = FieldType::Reference("users".to_string());
    assert!(ft.find_invalid_reference(&db, None).is_none());
}

#[test]
fn find_invalid_reference_invalid() {
    let (_temp, db) = setup_db(vec![("users", vec![])]);
    let ft = FieldType::Reference("nonexistent".to_string());
    let result = ft.find_invalid_reference(&db, None);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "nonexistent");
}

#[test]
fn find_invalid_reference_self_reference() {
    let (_temp, db) = setup_db(vec![]);
    let ft = FieldType::Reference("new_collection".to_string());
    assert!(
        ft.find_invalid_reference(&db, Some("new_collection"))
            .is_none()
    );
    assert!(ft.find_invalid_reference(&db, Some("other")).is_some());
}

#[test]
fn find_invalid_reference_primitives() {
    let (_temp, db) = setup_db(vec![]);
    assert!(FieldType::Int.find_invalid_reference(&db, None).is_none());
    assert!(
        FieldType::String
            .find_invalid_reference(&db, None)
            .is_none()
    );
}

#[test]
fn find_invalid_reference_nested() {
    let (_temp, db) = setup_db(vec![("users", vec![])]);
    let nullable_valid = FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())));
    let nullable_invalid = FieldType::Nullable(Box::new(FieldType::Reference("bad".to_string())));
    let array_valid = FieldType::Array(Box::new(FieldType::Reference("users".to_string())));
    let array_invalid = FieldType::Array(Box::new(FieldType::Reference("bad".to_string())));
    assert!(nullable_valid.find_invalid_reference(&db, None).is_none());
    assert!(nullable_invalid.find_invalid_reference(&db, None).is_some());
    assert!(array_valid.find_invalid_reference(&db, None).is_none());
    assert!(array_invalid.find_invalid_reference(&db, None).is_some());
}

#[test]
fn validate_references_valid() {
    let (_temp, db) = setup_db(vec![("users", vec![])]);
    let s = schema(vec![
        ("name", FieldType::String),
        ("author", FieldType::Reference("users".to_string())),
    ]);
    assert!(s.validate_references(&db, None).is_ok());
}

#[test]
fn validate_references_invalid() {
    let (_temp, db) = setup_db(vec![("users", vec![])]);
    let s = schema(vec![(
        "author",
        FieldType::Reference("nonexistent".to_string()),
    )]);
    let result = s.validate_references(&db, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("nonexistent"));
}

#[test]
fn validate_references_self_reference() {
    let (_temp, db) = setup_db(vec![]);
    let s = schema(vec![("parent", FieldType::Reference("nodes".to_string()))]);
    assert!(s.validate_references(&db, Some("nodes")).is_ok());
}

#[test]
fn validate_references_empty() {
    let (_temp, db) = setup_db(vec![]);
    let s = schema(vec![]);
    assert!(s.validate_references(&db, None).is_ok());
}

#[test]
fn find_referencing_collections_basic() {
    let (_temp, db) = setup_db(vec![
        ("users", vec![("name", FieldType::String)]),
        (
            "posts",
            vec![("author", FieldType::Reference("users".to_string()))],
        ),
        ("tags", vec![("name", FieldType::String)]),
    ]);
    let result = db.find_referencing_collections("users");
    assert_eq!(result.len(), 1);
    assert!(result.contains(&"posts".to_string()));
    assert!(!result.contains(&"tags".to_string()));
}

#[test]
fn find_referencing_collections_none() {
    let (_temp, db) = setup_db(vec![
        ("users", vec![("name", FieldType::String)]),
        ("posts", vec![("title", FieldType::String)]),
    ]);
    assert!(db.find_referencing_collections("users").is_empty());
}

#[test]
fn find_referencing_collections_excludes_self() {
    let (_temp, db) = setup_db(vec![(
        "nodes",
        vec![("parent", FieldType::Reference("nodes".to_string()))],
    )]);
    assert!(db.find_referencing_collections("nodes").is_empty());
}

#[test]
fn find_referencing_collections_multiple() {
    let (_temp, db) = setup_db(vec![
        ("users", vec![("name", FieldType::String)]),
        (
            "posts",
            vec![("author", FieldType::Reference("users".to_string()))],
        ),
        (
            "comments",
            vec![("author", FieldType::Reference("users".to_string()))],
        ),
    ]);
    let result = db.find_referencing_collections("users");
    assert_eq!(result.len(), 2);
    assert!(result.contains(&"posts".to_string()));
    assert!(result.contains(&"comments".to_string()));
}

#[test]
fn find_referencing_collections_nested() {
    let (_temp, db) = setup_db(vec![
        ("users", vec![("name", FieldType::String)]),
        (
            "posts",
            vec![(
                "authors",
                FieldType::Array(Box::new(FieldType::Reference("users".to_string()))),
            )],
        ),
    ]);
    let result = db.find_referencing_collections("users");
    assert_eq!(result.len(), 1);
    assert!(result.contains(&"posts".to_string()));
}
