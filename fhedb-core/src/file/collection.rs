use crate::db::{collection::Collection, schema::Schema};
use crate::file::types::{LogEntry, Operation};
use bson::{Bson, Document as BsonDocument};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Seek, Write};
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
    /// Returns [`Ok(usize)`](Result::Ok) with the file offset where the entry was written,
    /// or [`Err`]\([`io::Error`]) if the operation failed.
    fn append_to_log(&self, operation: &Operation, document: &BsonDocument) -> io::Result<usize>;

    /// Reads all log entries from the collection's logfile.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec`]\([`LogEntry`]) if the log entries were read successfully,
    /// or [`Err`]\([`io::Error`]) if the log entries could not be read.
    fn read_log_entries(&self) -> io::Result<Vec<LogEntry>>;

    /// Reads a single log entry from the collection's logfile at the specified offset.
    ///
    /// ## Arguments
    ///
    /// * `offset` - The byte offset in the logfile where the entry begins.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`LogEntry`]) if the log entry was read successfully,
    /// or [`Err`]\([`io::Error`]) if the entry could not be read or the offset is invalid.
    fn read_log_entry_at_offset(&self, offset: usize) -> io::Result<LogEntry>;

    /// Compacts the logfile by applying all operations and creating a new logfile
    /// with only the final state of each document.
    ///
    /// This method reads all log entries, applies them in order to reconstruct
    /// the current state of documents, then writes a new compacted logfile
    /// containing only the final state of each document as INSERT operations.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the logfile was compacted successfully,
    /// or [`Err`]\([`io::Error`]) if the compaction failed.
    fn compact_logfile(&self) -> io::Result<()>;

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

    fn append_to_log(&self, operation: &Operation, document: &BsonDocument) -> io::Result<usize> {
        self.ensure_collection_dir()?;

        let logfile_path = self.logfile_path();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
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

        // Get the offset where the entry was written
        let offset = file.stream_position()? as usize - bson_bytes.len() - 1;

        Ok(offset)
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

    fn read_log_entry_at_offset(&self, offset: usize) -> io::Result<LogEntry> {
        let logfile_path = self.logfile_path();

        if !logfile_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Logfile does not exist",
            ));
        }

        let contents = fs::read(&logfile_path)?;
        let offset = offset as usize;

        if offset >= contents.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Offset is beyond end of file",
            ));
        }

        // Check if we have enough bytes for the BSON length header
        if offset + 4 >= contents.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes for BSON length header",
            ));
        }

        let length = u32::from_le_bytes([
            contents[offset],
            contents[offset + 1],
            contents[offset + 2],
            contents[offset + 3],
        ]) as usize;

        if offset + length > contents.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "BSON entry extends beyond end of file",
            ));
        }

        let entry_bytes = &contents[offset..offset + length];
        let log_doc: BsonDocument = bson::from_slice(entry_bytes).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse BSON: {}", e),
            )
        })?;

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

        Ok(LogEntry {
            timestamp,
            operation,
            document,
        })
    }

    fn compact_logfile(&self) -> io::Result<()> {
        let logfile_path = self.logfile_path();

        // Read all log entries
        let entries = self.read_log_entries()?;

        // If no entries, nothing to compact
        if entries.is_empty() {
            return Ok(());
        }

        // Apply all operations to reconstruct current state
        let mut current_state: HashMap<String, BsonDocument> = HashMap::new();

        for log_entry in entries {
            let document = log_entry.document;
            let operation = log_entry.operation;

            // Extract document ID for tracking
            let id_field = self.id_field.clone();
            let doc_id = match document.get(id_field) {
                Some(bson::Bson::String(s)) => s.clone(),
                Some(bson::Bson::Int32(i)) => i.to_string(),
                Some(bson::Bson::Int64(i)) => i.to_string(),
                _ => continue, // Skip documents without valid ID
            };

            match operation {
                Operation::Insert => {
                    current_state.insert(doc_id, document);
                }
                Operation::Delete => {
                    current_state.remove(&doc_id);
                }
                Operation::Update => {
                    current_state.insert(doc_id, document);
                }
            }
        }

        // Create a temporary file for the compacted log
        let temp_path = logfile_path.with_extension("tmp");
        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&temp_path)?;

        // Write compacted entries (only final state as INSERT operations)
        for (_, document) in current_state {
            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut log_entry = BsonDocument::new();
            log_entry.insert("timestamp", Bson::String(timestamp));
            log_entry.insert(
                "operation",
                Bson::String(Operation::Insert.as_str().to_string()),
            );
            log_entry.insert("document", Bson::Document(document));

            let bson_bytes = bson::to_vec(&log_entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            temp_file.write_all(&bson_bytes)?;
            writeln!(temp_file)?;
        }

        // Atomically replace the old logfile with the compacted one
        fs::rename(temp_path, logfile_path)?;

        Ok(())
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
