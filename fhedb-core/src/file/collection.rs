use crate::db::{collection::Collection, schema::Schema};
use crate::file::types::{LogEntry, Operation};
use bson::{Bson, Document as BsonDocument};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

/// Trait for file operations on collections.
///
/// This trait provides functionality for creating collection directories
/// and maintaining append-only logfiles for each collection.
pub trait CollectionFileOps {
    /// Gets the path to the collection's logfile.
    fn logfile_path(&self) -> PathBuf;

    /// Gets the path to the collection's metadata file.
    fn metadata_path(&self) -> PathBuf;

    /// Ensures the collection's directory exists.
    fn ensure_collection_dir(&self) -> io::Result<()>;

    /// Appends a document operation to the collection's logfile.
    ///
    /// ## Arguments
    ///
    /// * `operation` - The operation to append to the logfile.
    /// * `document` - The document to append to the logfile.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the operation was appended to the logfile,
    /// or [`Err`]\([`io::Error`]) if the operation failed.
    fn append_to_log(&self, operation: &Operation, document: &BsonDocument) -> io::Result<()>;

    /// Reads all log entries from the collection's logfile.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec`]\([`LogEntry`]) if the log entries were read successfully,
    /// or [`Err`]\([`io::Error`]) if the log entries could not be read.
    fn read_log_entries(&self) -> io::Result<Vec<LogEntry>>;

    /// Writes the collection's metadata to the metadata file.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the metadata was written successfully,
    /// or [`Err`]\([`io::Error`]) if the metadata could not be written.
    fn write_metadata(&self) -> io::Result<()>;

    /// Reads the collection's metadata from the metadata file.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if the metadata was read successfully,
    /// or [`Err`]\([`io::Error`]) if the metadata could not be read.
    fn read_metadata(collection_dir: impl Into<PathBuf>) -> io::Result<Collection>;
}

// Implementation for Collection should be added in collection.rs:
impl CollectionFileOps for Collection {
    fn logfile_path(&self) -> PathBuf {
        self.base_path.join("logfile.log")
    }

    fn metadata_path(&self) -> PathBuf {
        self.base_path.join("metadata.bin")
    }

    fn ensure_collection_dir(&self) -> io::Result<()> {
        fs::create_dir_all(self.base_path.clone())
    }

    fn append_to_log(&self, operation: &Operation, document: &BsonDocument) -> io::Result<()> {
        self.ensure_collection_dir()?;

        let logfile_path = self.logfile_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile_path)?;

        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut log_entry = BsonDocument::new();
        log_entry.insert("timestamp", Bson::String(timestamp));
        log_entry.insert("operation", Bson::String(operation.as_str().to_string()));
        log_entry.insert("document", Bson::Document(document.clone()));

        let bson_bytes =
            bson::to_vec(&log_entry).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(&bson_bytes)?;
        writeln!(file)?; // Add a newline to separate entries

        Ok(())
    }

    fn read_log_entries(&self) -> io::Result<Vec<LogEntry>> {
        let logfile_path = self.logfile_path();

        if !logfile_path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read(&logfile_path)?;
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset < contents.len() {
            if offset + 4 >= contents.len() {
                break;
            }

            let length = u32::from_le_bytes([
                contents[offset],
                contents[offset + 1],
                contents[offset + 2],
                contents[offset + 3],
            ]) as usize;

            if offset + length > contents.len() {
                break;
            }

            let entry_bytes = &contents[offset..offset + length];
            match bson::from_slice::<BsonDocument>(entry_bytes) {
                Ok(log_doc) => {
                    let timestamp = log_doc
                        .get_str("timestamp")
                        .unwrap_or("unknown")
                        .to_string();
                    let operation_str = log_doc.get_str("operation").unwrap_or("unknown");
                    let operation = Operation::from_str(operation_str).unwrap_or(Operation::Insert);
                    let document = log_doc
                        .get_document("document")
                        .cloned()
                        .unwrap_or_default();

                    entries.push(LogEntry {
                        timestamp,
                        operation,
                        document,
                    });

                    offset += length;

                    if offset < contents.len() && contents[offset] == b'\n' {
                        offset += 1;
                    }
                }
                Err(_) => {
                    let newline_pos = contents[offset..].iter().position(|&b| b == b'\n');
                    if let Some(pos) = newline_pos {
                        offset += pos + 1;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(entries)
    }

    fn write_metadata(&self) -> io::Result<()> {
        self.ensure_collection_dir()?;

        let metadata_path = self.metadata_path();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(metadata_path)?;

        let mut metadata = BsonDocument::new();
        metadata.insert("name", Bson::String(self.name.clone()));
        metadata.insert("inserts", Bson::Int64(self.inserts as i64));
        metadata.insert("schema", Bson::Document(self.schema.clone().into()));

        let bson_bytes =
            bson::to_vec(&metadata).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(&bson_bytes)?;
        Ok(())
    }

    fn read_metadata(collection_dir: impl Into<PathBuf>) -> io::Result<Collection> {
        let collection_dir: PathBuf = collection_dir.into();
        let base_path = collection_dir.parent().unwrap();
        let metadata_path = collection_dir.join("metadata.bin");

        if !metadata_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Metadata file not found: {}", metadata_path.display()),
            ));
        }

        let contents = fs::read(&metadata_path)?;
        let metadata: BsonDocument = bson::from_slice(&contents).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid BSON: {}", e))
        })?;
        let name = metadata.get_str("name").unwrap_or("unknown");
        let inserts = metadata.get_i64("inserts").unwrap_or(0) as u64;
        let schema = Schema::from(metadata.get_document("schema").cloned().unwrap_or_default());

        let mut collection = Collection::new(name, schema, base_path).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid schema: {}", e))
        })?;
        collection.inserts = inserts;
        Ok(collection)
    }
}
