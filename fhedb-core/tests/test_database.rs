use bson::doc;
use fhedb_core::prelude::*;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{make_int_schema, make_simple_schema, make_string_schema};

#[test]
fn test_database_creation_and_basic_properties() {
    let temp_dir = tempdir().unwrap();
    let db = Database::new("test_db", temp_dir.path());
    let expected_path = temp_dir.path().join("test_db");

    // Test creation and path properties
    assert_eq!(db.name, "test_db");
    assert_eq!(db.base_path, expected_path);
    assert_eq!(db.path(), &expected_path);
    assert_eq!(db.collection_count(), 0);
    assert_eq!(db.collection_names(), Vec::<String>::new());
}

#[test]
fn test_database_collection_management() {
    let temp_dir = tempdir().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());
    let schema1 = make_string_schema();
    let schema2 = make_int_schema();

    // Test collection creation and duplicate handling
    let result1 = db.create_collection("users", schema1);
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().name, "users");
    assert_eq!(db.collection_count(), 1);
    assert!(db.has_collection("users"));

    // Test duplicate collection creation
    let result2 = db.create_collection("users", schema2);
    assert!(result2.is_err());
    assert!(
        result2
            .unwrap_err()
            .contains("Collection 'users' already exists")
    );

    // Test multiple collections
    let simple_schema = make_simple_schema();
    db.create_collection("posts", simple_schema).unwrap();

    assert_eq!(db.collection_count(), 2);
    let mut names = db.collection_names();
    names.sort();
    assert_eq!(names, vec!["posts", "users"]);

    // Test collection retrieval
    assert!(db.get_collection("users").is_some());
    assert!(db.get_collection_mut("users").is_some());
    assert!(db.get_collection("nonexistent").is_none());
}

#[test]
fn test_database_collection_drop_operations() {
    let temp_dir = tempdir().unwrap();
    let mut db = Database::new("test_db", temp_dir.path());

    // Create collections
    db.create_collection("users", make_string_schema()).unwrap();
    db.create_collection("posts", make_int_schema()).unwrap();
    assert_eq!(db.collection_count(), 2);

    // Test successful drop
    let result = db.drop_collection("users");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "users");
    assert_eq!(db.collection_count(), 1);
    assert!(!db.has_collection("users"));
    assert!(db.has_collection("posts")); // Other collection preserved

    // Test drop non-existent collection
    let result = db.drop_collection("nonexistent");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Collection 'nonexistent' not found")
    );

    // Test clear all collections (does not drop collections)
    db.clear_collections();
    assert_eq!(db.collection_count(), 0);
    assert_eq!(db.collection_names(), Vec::<String>::new());
}

#[test]
fn test_database_file_operations() {
    let temp_dir = tempdir().unwrap();

    // Test loading from non-existent directory
    let nonexistent_path = temp_dir.path().join("nonexistent");
    let result = Database::from_files("test_db", &nonexistent_path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);

    // Test loading from empty directory
    let db_path = temp_dir.path().join("test_db");
    fs::create_dir_all(&db_path).unwrap();
    let result = Database::from_files("test_db", temp_dir.path());
    assert!(result.is_ok());
    let db = result.unwrap();
    assert_eq!(db.name, "test_db");
    assert_eq!(db.collection_count(), 0);

    // Test roundtrip: create, save, and load
    let mut original_db = Database::new("test_db2", temp_dir.path());
    original_db
        .create_collection("users", make_string_schema())
        .unwrap();
    original_db
        .create_collection("products", make_int_schema())
        .unwrap();

    // Add test data
    let users_collection = original_db.get_collection_mut("users").unwrap();
    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "John Doe",
        "age": 30i64
    };
    users_collection.add_document(doc).unwrap();

    // Load from files and verify
    let loaded_db = Database::from_files("test_db2", temp_dir.path()).unwrap();
    assert_eq!(loaded_db.name, "test_db2");
    assert_eq!(loaded_db.collection_count(), 2);
    assert!(loaded_db.has_collection("users"));
    assert!(loaded_db.has_collection("products"));

    // Test that files are ignored, only directories are loaded as collections
    fs::write(loaded_db.path().join("some_file.txt"), "test content").unwrap();
    let reloaded_db = Database::from_files("test_db2", temp_dir.path()).unwrap();
    assert_eq!(reloaded_db.collection_count(), 2); // File should be ignored
}
