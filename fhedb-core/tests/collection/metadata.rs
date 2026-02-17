use bson::doc;
use fhedb_core::prelude::*;
use std::fs;
use tempfile::tempdir;

use super::super::common::{make_complex_schema, make_int_schema, make_string_schema};

#[test]
fn write_creates_file() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    assert!(collection.write_metadata().is_ok());

    let metadata_path = collection.metadata_path();
    assert!(metadata_path.exists());
    assert!(metadata_path.is_file());
}

#[test]
fn read_round_trip_string_id() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let original_collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    assert!(original_collection.write_metadata().is_ok());

    let read_collection = Collection::read_metadata(
        original_collection.base_path().clone().parent().unwrap(),
        &original_collection.name,
    )
    .unwrap();

    assert_eq!(read_collection.name, original_collection.name);
    assert_eq!(
        read_collection.schema().fields,
        original_collection.schema().fields
    );
    assert_eq!(read_collection.inserts(), original_collection.inserts());
    assert_eq!(read_collection.base_path(), original_collection.base_path());
}

#[test]
fn read_round_trip_int_id() {
    let schema = make_int_schema();
    let temp_dir = tempdir().unwrap();
    let original_collection = Collection::new("employees", schema, temp_dir.path()).unwrap();
    assert!(original_collection.write_metadata().is_ok());

    let read_collection = Collection::read_metadata(
        original_collection.base_path().clone().parent().unwrap(),
        &original_collection.name,
    )
    .unwrap();

    assert_eq!(read_collection.name, original_collection.name);
    assert_eq!(
        read_collection.schema().fields,
        original_collection.schema().fields
    );
    assert_eq!(read_collection.inserts(), original_collection.inserts());
    assert_eq!(read_collection.base_path(), original_collection.base_path());
}

#[test]
fn read_with_inserts() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let doc1 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64,
        "active": true,
        "scores": [85.5, 92.0, 78.5],
        "department": "engineering"
    };
    let doc2 = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Bob",
        "age": 25i64,
        "active": false,
        "scores": [90.0, 88.5, 95.0],
        "department": "marketing"
    };

    collection.add_document(doc1).unwrap();
    collection.add_document(doc2).unwrap();

    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();

    assert_eq!(read_collection.inserts(), 2);
    assert_eq!(read_collection.inserts(), collection.inserts());
}

#[test]
fn read_file_not_found() {
    let temp_dir = tempdir().unwrap();

    let result = Collection::read_metadata(temp_dir.path(), "nonexistent_collection");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(error.to_string().contains("Metadata file not found"));
}

#[test]
fn metadata_paths() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();

    let expected_metadata_path = collection.base_path().join("metadata.bin");
    assert_eq!(collection.metadata_path(), expected_metadata_path);

    let expected_logfile_path = collection.base_path().join("logfile.log");
    assert_eq!(collection.logfile_path(), expected_logfile_path);
}

#[test]
fn write_metadata_overwrites_existing() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let mut collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    assert!(collection.write_metadata().is_ok());

    let doc = doc! {
        "id": uuid::Uuid::new_v4().to_string(),
        "name": "Alice",
        "age": 30i64,
        "active": true,
        "scores": [85.5, 92.0],
        "department": "engineering"
    };
    collection.add_document(doc).unwrap();
    assert!(collection.write_metadata().is_ok());

    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();
    assert_eq!(read_collection.inserts(), 1);
    assert_eq!(read_collection.inserts(), collection.inserts());
}

#[test]
fn preserves_complex_schema() {
    let schema = make_complex_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("complex_users", schema, temp_dir.path()).unwrap();
    assert!(collection.write_metadata().is_ok());

    let read_collection = Collection::read_metadata(
        collection.base_path().clone().parent().unwrap(),
        &collection.name,
    )
    .unwrap();
    assert_eq!(
        read_collection.schema().fields.len(),
        collection.schema().fields.len()
    );

    for (key, value) in collection.schema().fields.iter() {
        assert_eq!(read_collection.schema().fields.get(key), Some(value));
    }
}

#[test]
fn metadata_file_size() {
    let schema = make_string_schema();
    let temp_dir = tempdir().unwrap();
    let collection = Collection::new("users", schema, temp_dir.path()).unwrap();
    assert!(collection.write_metadata().is_ok());

    let metadata_path = collection.metadata_path();
    let metadata = fs::read(&metadata_path).unwrap();
    assert!(!metadata.is_empty());

    let parsed: bson::Document = bson::Document::from_reader(&mut metadata.as_slice()).unwrap();
    assert!(parsed.contains_key("name"));
    assert!(parsed.contains_key("inserts"));
    assert!(parsed.contains_key("schema"));
}

#[test]
fn multiple_collections_metadata_isolation() {
    let schema1 = make_string_schema();
    let schema2 = make_int_schema();
    let temp_dir = tempdir().unwrap();

    let collection1 = Collection::new("users", schema1, temp_dir.path()).unwrap();
    let collection2 = Collection::new("employees", schema2, temp_dir.path()).unwrap();

    assert!(collection1.write_metadata().is_ok());
    assert!(collection2.write_metadata().is_ok());

    assert!(collection1.metadata_path().exists());
    assert!(collection2.metadata_path().exists());

    let read_collection1 = Collection::read_metadata(
        collection1.base_path().clone().parent().unwrap(),
        &collection1.name,
    )
    .unwrap();
    let read_collection2 = Collection::read_metadata(
        collection2.base_path().clone().parent().unwrap(),
        &collection2.name,
    )
    .unwrap();

    assert_eq!(read_collection1.name, "users");
    assert_eq!(read_collection2.name, "employees");
    assert_ne!(
        read_collection1.schema().fields,
        read_collection2.schema().fields
    );
}
