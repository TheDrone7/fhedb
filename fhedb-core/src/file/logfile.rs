use bson::{Bson, Document as BsonDocument};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// File operations for maintaining collections in the database.
///
/// This module provides functionality for creating collection directories
/// and maintaining append-only logfiles for each collection.
#[derive(Debug)]
pub struct FileOps {
    /// The base path for all database operations.
    base_path: PathBuf,
}

impl FileOps {
    /// Creates a new [`FileOps`] instance with the specified base path.
    ///
    /// ## Arguments
    ///
    /// * `base_path` - The base directory path for all database operations.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`FileOps`]) if the base path is valid, or [`Err`]\([`std::io::Error`]) otherwise.
    pub fn new<P: AsRef<Path>>(base_path: P) -> std::io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Ensure the base directory exists
        std::fs::create_dir_all(&base_path)?;

        Ok(Self { base_path })
    }

    /// Gets the path to a collection's directory.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection.
    ///
    /// ## Returns
    ///
    /// The path to the collection's directory.
    pub fn collection_dir(&self, collection_name: &str) -> PathBuf {
        self.base_path.join(collection_name)
    }

    /// Gets the path to a collection's logfile.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection.
    ///
    /// ## Returns
    ///
    /// The path to the collection's logfile.
    pub fn logfile_path(&self, collection_name: &str) -> PathBuf {
        self.collection_dir(collection_name).join("documents.log")
    }

    /// Ensures a collection's directory exists.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the directory was created or already exists,
    /// or [`Err`]\([`io::Error`]) if creation failed.
    pub fn ensure_collection_dir(&self, collection_name: &str) -> io::Result<()> {
        let dir_path = self.collection_dir(collection_name);
        fs::create_dir_all(dir_path)
    }

    /// Appends a document operation to the collection's logfile.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `operation` - The operation to log (e.g., "INSERT", "DELETE").
    /// * `document` - The BSON document to log.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the operation was logged successfully,
    /// or [`Err`]\([`io::Error`]) if writing failed.
    pub fn append_to_log(
        &self,
        collection_name: &str,
        operation: &str,
        document: &BsonDocument,
    ) -> io::Result<()> {
        // Ensure the collection directory exists
        self.ensure_collection_dir(collection_name)?;

        let logfile_path = self.logfile_path(collection_name);

        // Open the logfile in append mode, create if it doesn't exist
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile_path)?;

        // Create a log entry with timestamp
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut log_entry = BsonDocument::new();
        log_entry.insert("timestamp", Bson::String(timestamp));
        log_entry.insert("operation", Bson::String(operation.to_string()));
        log_entry.insert("document", Bson::Document(document.clone()));

        // Serialize to BSON and write to file
        let bson_bytes =
            bson::to_vec(&log_entry).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        file.write_all(&bson_bytes)?;
        writeln!(file)?; // Add newline for readability

        Ok(())
    }

    /// Reads all log entries from a collection's logfile.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<LogEntry>`]) containing all log entries,
    /// or [`Err`]\([`io::Error`]) if reading failed.
    pub fn read_log_entries(&self, collection_name: &str) -> io::Result<Vec<LogEntry>> {
        let logfile_path = self.logfile_path(collection_name);

        if !logfile_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read(&logfile_path)?;
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset < content.len() {
            // Check if we have enough bytes for a BSON document (minimum 5 bytes for length + type)
            if offset + 4 >= content.len() {
                break;
            }

            // Read the BSON document length (first 4 bytes, little-endian)
            let doc_len = u32::from_le_bytes([
                content[offset],
                content[offset + 1],
                content[offset + 2],
                content[offset + 3],
            ]) as usize;

            // Check if we have enough bytes for the full document
            if offset + doc_len > content.len() {
                break;
            }

            // Try to deserialize the BSON document
            match bson::from_slice::<BsonDocument>(&content[offset..offset + doc_len]) {
                Ok(log_doc) => {
                    // Extract the log entry fields
                    let timestamp = log_doc
                        .get_str("timestamp")
                        .unwrap_or("unknown")
                        .to_string();
                    let operation = log_doc
                        .get_str("operation")
                        .unwrap_or("unknown")
                        .to_string();
                    let document = log_doc
                        .get_document("document")
                        .cloned()
                        .unwrap_or_default();

                    entries.push(LogEntry {
                        timestamp,
                        operation,
                        document,
                    });

                    // Move offset past this BSON document
                    offset += doc_len;

                    // Skip newline if present
                    if offset < content.len() && content[offset] == b'\n' {
                        offset += 1;
                    }
                }
                Err(_) => {
                    // If we can't parse the BSON document, skip to the next newline
                    let newline_pos = content[offset..].iter().position(|&b| b == b'\n');
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
}

/// A log entry representing a database operation.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The timestamp when the operation occurred.
    pub timestamp: String,
    /// The type of operation (e.g., "INSERT", "DELETE").
    pub operation: String,
    /// The BSON document associated with the operation.
    pub document: BsonDocument,
}

impl LogEntry {
    /// Creates a new log entry.
    ///
    /// ## Arguments
    ///
    /// * `operation` - The type of operation.
    /// * `document` - The BSON document associated with the operation.
    ///
    /// ## Returns
    ///
    /// A new [`LogEntry`] with the current timestamp.
    pub fn new(operation: String, document: BsonDocument) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation,
            document,
        }
    }
}
