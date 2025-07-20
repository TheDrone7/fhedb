use bson::{Bson, Document as BsonDocument};
use fhedb_core::file::logfile::{FileOps, LogEntry, Operation};
use tempfile::TempDir;

#[test]
fn test_file_ops_creation() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Test collection directory path
    let collection_dir = file_ops.collection_dir("test_collection");
    assert_eq!(collection_dir, base_path.join("test_collection"));

    // Test logfile path
    let logfile_path = file_ops.logfile_path("test_collection");
    assert_eq!(
        logfile_path,
        base_path.join("test_collection").join("documents.log")
    );
}

#[test]
fn test_append_to_log() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Create a test document
    let mut doc = BsonDocument::new();
    doc.insert("name", Bson::String("Alice".to_string()));
    doc.insert("age", Bson::Int32(25));

    // Append to log
    file_ops
        .append_to_log("test_collection", &Operation::Insert, &doc)
        .unwrap();

    // Verify the collection directory was created
    let collection_dir = base_path.join("test_collection");
    assert!(collection_dir.exists());
    assert!(collection_dir.is_dir());

    // Verify the logfile was created
    let logfile_path = collection_dir.join("documents.log");
    assert!(logfile_path.exists());

    // Check that the file has content
    let content = std::fs::read(&logfile_path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn test_read_log_entries() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Create test documents
    let mut doc1 = BsonDocument::new();
    doc1.insert("name", Bson::String("Alice".to_string()));
    doc1.insert("age", Bson::Int32(25));

    let mut doc2 = BsonDocument::new();
    doc2.insert("name", Bson::String("Bob".to_string()));
    doc2.insert("age", Bson::Int32(30));

    // Append multiple entries to log
    file_ops
        .append_to_log("test_collection", &Operation::Insert, &doc1)
        .unwrap();
    file_ops
        .append_to_log("test_collection", &Operation::Insert, &doc2)
        .unwrap();
    file_ops
        .append_to_log("test_collection", &Operation::Delete, &doc1)
        .unwrap();

    // Read back the log entries
    let entries = file_ops.read_log_entries("test_collection").unwrap();

    // Verify we got 3 entries
    assert_eq!(entries.len(), 3);

    // Verify the first entry
    let first_entry = &entries[0];
    assert_eq!(first_entry.operation, Operation::Insert);
    assert_eq!(first_entry.document.get_str("name").unwrap(), "Alice");
    assert_eq!(first_entry.document.get_i32("age").unwrap(), 25);
    assert!(!first_entry.timestamp.is_empty());

    // Verify the second entry
    let second_entry = &entries[1];
    assert_eq!(second_entry.operation, Operation::Insert);
    assert_eq!(second_entry.document.get_str("name").unwrap(), "Bob");
    assert_eq!(second_entry.document.get_i32("age").unwrap(), 30);

    // Verify the third entry
    let third_entry = &entries[2];
    assert_eq!(third_entry.operation, Operation::Delete);
    assert_eq!(third_entry.document.get_str("name").unwrap(), "Alice");
}

#[test]
fn test_read_empty_logfile() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Try to read from a non-existent logfile
    let entries = file_ops.read_log_entries("nonexistent_collection").unwrap();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_log_entry_creation() {
    let mut doc = BsonDocument::new();
    doc.insert("name", Bson::String("Test".to_string()));

    let log_entry = LogEntry::new(Operation::Insert, doc.clone());

    assert_eq!(log_entry.operation, Operation::Insert);
    assert_eq!(log_entry.document.get_str("name").unwrap(), "Test");
    assert!(!log_entry.timestamp.is_empty());

    // Verify timestamp is in RFC3339 format
    let _parsed = chrono::DateTime::parse_from_rfc3339(&log_entry.timestamp).unwrap();
}

#[test]
fn test_multiple_collections() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Create documents for different collections
    let mut user_doc = BsonDocument::new();
    user_doc.insert("name", Bson::String("Alice".to_string()));

    let mut product_doc = BsonDocument::new();
    product_doc.insert("title", Bson::String("Widget".to_string()));
    product_doc.insert("price", Bson::Double(19.99));

    // Write to different collections
    file_ops
        .append_to_log("users", &Operation::Insert, &user_doc)
        .unwrap();
    file_ops
        .append_to_log("products", &Operation::Insert, &product_doc)
        .unwrap();

    // Verify separate directories were created
    assert!(base_path.join("users").exists());
    assert!(base_path.join("products").exists());

    // Verify separate logfiles were created
    assert!(base_path.join("users").join("documents.log").exists());
    assert!(base_path.join("products").join("documents.log").exists());

    // Read back entries from each collection
    let user_entries = file_ops.read_log_entries("users").unwrap();
    let product_entries = file_ops.read_log_entries("products").unwrap();

    assert_eq!(user_entries.len(), 1);
    assert_eq!(product_entries.len(), 1);

    assert_eq!(user_entries[0].document.get_str("name").unwrap(), "Alice");
    assert_eq!(
        product_entries[0].document.get_str("title").unwrap(),
        "Widget"
    );
}

#[test]
fn test_complex_document_logging() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let file_ops = FileOps::new(base_path).unwrap();

    // Create a complex document with various BSON types
    let mut complex_doc = BsonDocument::new();
    complex_doc.insert("string_field", Bson::String("hello".to_string()));
    complex_doc.insert("int_field", Bson::Int32(42));
    complex_doc.insert("double_field", Bson::Double(3.14));
    complex_doc.insert("bool_field", Bson::Boolean(true));
    complex_doc.insert(
        "array_field",
        Bson::Array(vec![
            Bson::String("item1".to_string()),
            Bson::Int32(123),
            Bson::Boolean(false),
        ]),
    );

    // Create a nested document
    let mut nested_doc = BsonDocument::new();
    nested_doc.insert("nested_key", Bson::String("nested_value".to_string()));
    complex_doc.insert("nested_field", Bson::Document(nested_doc));

    // Write to log
    file_ops
        .append_to_log("complex_collection", &Operation::Insert, &complex_doc)
        .unwrap();

    // Read back
    let entries = file_ops.read_log_entries("complex_collection").unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    let doc = &entry.document;

    // Verify all fields are preserved
    assert_eq!(doc.get_str("string_field").unwrap(), "hello");
    assert_eq!(doc.get_i32("int_field").unwrap(), 42);
    assert_eq!(doc.get_f64("double_field").unwrap(), 3.14);
    assert_eq!(doc.get_bool("bool_field").unwrap(), true);

    // Verify array
    let array = doc.get_array("array_field").unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array[0].as_str().unwrap(), "item1");
    assert_eq!(array[1].as_i32().unwrap(), 123);
    assert_eq!(array[2].as_bool().unwrap(), false);

    // Verify nested document
    let nested = doc.get_document("nested_field").unwrap();
    assert_eq!(nested.get_str("nested_key").unwrap(), "nested_value");
}
