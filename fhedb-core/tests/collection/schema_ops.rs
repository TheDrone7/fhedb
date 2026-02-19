use bson::{Bson, doc};
use fhedb_core::prelude::*;
use tempfile::TempDir;

use crate::common;

fn create_test_collection() -> (Collection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let schema = common::make_int_schema();

    let collection = Collection::new("test_collection", schema, temp_dir.path())
        .expect("Failed to create collection");

    (collection, temp_dir)
}

fn create_test_collection_with_data() -> (Collection, TempDir) {
    let (mut collection, temp_dir) = create_test_collection();

    collection
        .add_document(doc! {
            "name": "Alice",
            "age": 30
        })
        .expect("Failed to add document");

    collection
        .add_document(doc! {
            "name": "Bob",
            "age": 25
        })
        .expect("Failed to add document");

    (collection, temp_dir)
}

fn create_complex_collection() -> (Collection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let schema = common::make_complex_schema();

    let collection = Collection::new("complex_collection", schema, temp_dir.path())
        .expect("Failed to create collection");

    (collection, temp_dir)
}

fn create_collection_with_defaults() -> (Collection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let schema = common::make_schema_with_defaults();

    let collection = Collection::new("defaults_collection", schema, temp_dir.path())
        .expect("Failed to create collection");

    (collection, temp_dir)
}

#[test]
fn has_field() {
    let (collection, _temp_dir) = create_test_collection();

    assert!(collection.has_field("name"));
    assert!(collection.has_field("age"));
    assert!(collection.has_field("id"));
    assert!(!collection.has_field("nonexistent"));
}

#[test]
fn validate_document() {
    let (collection, _temp_dir) = create_test_collection();

    let valid_doc = doc! {
        "name": "Test",
        "age": 25
    };
    assert!(collection.validate_document(&valid_doc).is_ok());

    let invalid_doc = doc! {
        "name": "Test",
        "age": "not_a_number"
    };
    assert!(collection.validate_document(&invalid_doc).is_err());
}

#[test]
fn add_field_success() {
    let (mut collection, _temp_dir) = create_test_collection();

    let field_def =
        FieldDefinition::with_default(FieldType::String, Bson::String("default".to_string()));
    let result = collection.add_field("email".to_string(), field_def);
    assert!(result.is_ok());
    assert!(collection.has_field("email"));

    let field_def = FieldDefinition::new(FieldType::Float);
    let result = collection.add_field("salary".to_string(), field_def);
    assert!(result.is_ok());
    assert!(collection.has_field("salary"));
}

#[test]
fn add_field_already_exists() {
    let (mut collection, _temp_dir) = create_test_collection();

    let field_def = FieldDefinition::new(FieldType::String);
    let result = collection.add_field("name".to_string(), field_def);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn add_field_without_default_with_existing_docs() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::new(FieldType::String);
    let result = collection.add_field("email".to_string(), field_def);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("without a default value"));
}

#[test]
fn add_field_nullable_auto_default() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::new(FieldType::Nullable(Box::new(FieldType::String)));
    let result = collection.add_field("email".to_string(), field_def);

    assert!(result.is_ok());
    assert!(collection.has_field("email"));

    let docs = collection.get_documents();
    for doc in docs {
        assert!(doc.data.contains_key("email"));
        assert_eq!(doc.data.get("email").unwrap(), &Bson::Null);
    }
}

#[test]
fn add_field_with_default_applies_to_existing_docs() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::with_default(
        FieldType::String,
        Bson::String("default@example.com".to_string()),
    );
    let result = collection.add_field("email".to_string(), field_def);

    assert!(result.is_ok());

    let docs = collection.get_documents();
    for doc in docs {
        assert!(doc.data.contains_key("email"));
        assert_eq!(
            doc.data.get("email").unwrap(),
            &Bson::String("default@example.com".to_string())
        );
    }
}

#[test]
fn remove_field_success() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let result = collection.remove_field("age");

    assert!(result.is_ok());
    assert!(!collection.has_field("age"));

    let docs = collection.get_documents();
    for doc in docs {
        assert!(!doc.data.contains_key("age"));
        assert!(doc.data.contains_key("name"));
    }
}

#[test]
fn remove_field_not_exists() {
    let (mut collection, _temp_dir) = create_test_collection();

    let result = collection.remove_field("nonexistent");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn modify_field_success() {
    let (mut collection, _temp_dir) = create_test_collection();

    let new_def =
        FieldDefinition::with_default(FieldType::String, Bson::String("Unknown".to_string()));
    let result = collection.modify_field("name", new_def);

    assert!(result.is_ok());
    assert!(collection.has_field("name"));
}

#[test]
fn modify_field_not_exists() {
    let (mut collection, _temp_dir) = create_test_collection();

    let new_def = FieldDefinition::new(FieldType::String);
    let result = collection.modify_field("nonexistent", new_def);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn rename_field_success() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let result = collection.rename_field("name", "full_name".to_string());

    assert!(result.is_ok());
    assert!(!collection.has_field("name"));
    assert!(collection.has_field("full_name"));

    let docs = collection.get_documents();
    for doc in docs {
        assert!(!doc.data.contains_key("name"));
        assert!(doc.data.contains_key("full_name"));
    }
}

#[test]
fn rename_field_old_not_exists() {
    let (mut collection, _temp_dir) = create_test_collection();

    let result = collection.rename_field("nonexistent", "new_name".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn rename_field_new_already_exists() {
    let (mut collection, _temp_dir) = create_test_collection();

    let result = collection.rename_field("name", "age".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn apply_defaults_to_existing_success() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::with_default(
        FieldType::String,
        Bson::String("test@example.com".to_string()),
    );
    let result = collection.apply_defaults_to_existing("email", &field_def);

    assert!(result.is_ok());
    let updated_ids = result.unwrap();
    assert_eq!(updated_ids.len(), 2);
}

#[test]
fn apply_defaults_to_existing_no_default() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::new(FieldType::String);
    let result = collection.apply_defaults_to_existing("email", &field_def);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("has no default value"));
}

#[test]
fn cleanup_removed_field() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let result = collection.cleanup_removed_field("age");

    assert!(result.is_ok());
    let updated_ids = result.unwrap();
    assert_eq!(updated_ids.len(), 2);

    let docs = collection.get_documents();
    for doc in docs {
        assert!(!doc.data.contains_key("age"));
        assert!(doc.data.contains_key("name"));
    }
}

#[test]
fn rename_field_in_documents() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let result = collection.rename_field_in_documents("name", "full_name");

    assert!(result.is_ok());
    let updated_ids = result.unwrap();
    assert_eq!(updated_ids.len(), 2);

    let docs = collection.get_documents();
    for doc in docs {
        assert!(!doc.data.contains_key("name"));
        assert!(doc.data.contains_key("full_name"));
    }
}

#[test]
fn list_fields() {
    let (collection, _temp_dir) = create_test_collection();

    let fields = collection.list_fields();

    assert!(fields.contains(&"name".to_string()));
    assert!(fields.contains(&"age".to_string()));
    assert!(fields.contains(&"id".to_string()));
}

#[test]
fn add_ids_to_all_documents() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let result = collection.add_ids_to_all_documents("id", "new_id");

    assert!(result.is_ok());
    let updated_ids = result.unwrap();
    assert_eq!(updated_ids.len(), 2);

    assert_eq!(collection.inserts(), 2);

    let docs = collection.get_documents();
    for doc in docs {
        assert!(doc.data.contains_key("new_id"));
        assert!(doc.data.contains_key("name"));
        assert!(doc.data.contains_key("age"));
    }
}

#[test]
fn add_field_id_type_error() {
    let (mut collection, _temp_dir) = create_test_collection();

    let field_def = FieldDefinition::new(FieldType::IdString);
    let result = collection.add_field("another_id".to_string(), field_def);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already has an ID field"));
}

#[test]
fn remove_id_field_creates_new_default_id() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();
    let original_id_field = collection.id_field_name().to_string();

    let result = collection.remove_field(&original_id_field);

    assert!(result.is_ok());
    assert_eq!(collection.id_field_name(), "id");
    assert!(collection.has_field("id"));
    assert_eq!(collection.inserts(), 2);
}

#[test]
fn string_id_collection_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let schema = common::make_string_schema();
    let mut collection = Collection::new("string_id_collection", schema, temp_dir.path())
        .expect("Failed to create collection");

    collection
        .add_document(doc! {
            "name": "Alice",
            "age": 30
        })
        .expect("Failed to add document");

    let field_def = FieldDefinition::with_default(
        FieldType::String,
        Bson::String("default@example.com".to_string()),
    );
    let result = collection.add_field("email".to_string(), field_def);
    assert!(result.is_ok());

    let docs = collection.get_documents();
    assert_eq!(docs.len(), 1);
    assert!(docs[0].data.contains_key("email"));
}

#[test]
fn integration_add_modify_remove_field() {
    let (mut collection, _temp_dir) = create_test_collection_with_data();

    let field_def = FieldDefinition::with_default(
        FieldType::String,
        Bson::String("test@example.com".to_string()),
    );
    collection
        .add_field("email".to_string(), field_def)
        .expect("Failed to add field");

    let new_def = FieldDefinition::with_default(
        FieldType::String,
        Bson::String("updated@example.com".to_string()),
    );
    collection
        .modify_field("email", new_def)
        .expect("Failed to modify field");

    let docs = collection.get_documents();
    for doc in docs {
        assert_eq!(
            doc.data.get("email").unwrap(),
            &Bson::String("updated@example.com".to_string())
        );
    }

    collection
        .remove_field("email")
        .expect("Failed to remove field");

    let docs = collection.get_documents();
    for doc in docs {
        assert!(!doc.data.contains_key("email"));
    }
}

#[test]
fn complex_schema_operations() {
    let (mut collection, _temp_dir) = create_complex_collection();

    assert!(collection.has_field("scores"));
    assert!(collection.has_field("tags"));
    assert!(collection.has_field("department"));
    assert!(collection.has_field("nickname"));

    let result = collection.add_field(
        "phone".to_string(),
        FieldDefinition::with_default(FieldType::String, Bson::String("555-0000".to_string())),
    );
    assert!(result.is_ok());

    let result = collection.rename_field("nickname", "alias".to_string());
    assert!(result.is_ok());
    assert!(!collection.has_field("nickname"));
    assert!(collection.has_field("alias"));
}

#[test]
fn schema_with_defaults() {
    let (mut collection, _temp_dir) = create_collection_with_defaults();

    collection
        .add_document(doc! {
            "name": "Test User",
            "email": "test@example.com"
        })
        .expect("Failed to add document");

    let docs = collection.get_documents();
    assert_eq!(docs.len(), 1);
    let doc = &docs[0];

    assert_eq!(doc.data.get("age").unwrap(), &Bson::Int64(18));
    assert_eq!(doc.data.get("active").unwrap(), &Bson::Boolean(true));
    assert_eq!(
        doc.data.get("role").unwrap(),
        &Bson::String("user".to_string())
    );
    assert_eq!(doc.data.get("score").unwrap(), &Bson::Double(0.0));
}
