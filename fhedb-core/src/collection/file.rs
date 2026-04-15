//! # Collection File Operations
//!
//! Provides file I/O operations for collection persistence.

use crate::{
    collection::Collection,
    schema::{schema_from_document, schema_to_document},
};
use bson::{Bson, Document as BsonDocument};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

/// Represents a database operation type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Insert a new document.
    Insert,
    /// Delete an existing document.
    Delete,
    /// Update an existing document.
    Update,
}

impl Operation {
    /// Returns the string representation of the operation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::Insert => "INSERT",
            Operation::Delete => "DELETE",
            Operation::Update => "UPDATE",
        }
    }
}

/// Error returned when parsing an invalid operation string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOperationError(String);

impl fmt::Display for ParseOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized operation: {:?}", self.0)
    }
}

impl FromStr for Operation {
    type Err = ParseOperationError;

    /// Converts a string to an operation.
    ///
    /// ## Arguments
    ///
    /// * `s` - The string to convert.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Operation`]) if the string is valid, or [`Err`]\([`ParseOperationError`]) if not recognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INSERT" => Ok(Operation::Insert),
            "DELETE" => Ok(Operation::Delete),
            "UPDATE" => Ok(Operation::Update),
            _ => Err(ParseOperationError(s.to_string())),
        }
    }
}

/// A log entry representing a database operation.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The timestamp when the operation occurred.
    pub timestamp: String,
    /// The type of operation.
    pub operation: Operation,
    /// The BSON document associated with the operation.
    pub document: BsonDocument,
}

impl LogEntry {
    /// Creates a new [`LogEntry`] with the current timestamp.
    ///
    /// ## Arguments
    ///
    /// * `operation` - The type of operation.
    /// * `document` - The BSON document associated with the operation.
    ///
    /// ## Returns
    ///
    /// A new [`LogEntry`] with the current UTC timestamp.
    pub fn new(operation: Operation, document: BsonDocument) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation,
            document,
        }
    }
}

/// File I/O operations for collection persistence.
impl Collection {
    /// Gets the path to the collection's logfile.
    pub fn logfile_path(&self) -> PathBuf {
        self.base_path.join("logfile.log")
    }

    /// Gets the path to the collection's metadata file.
    pub fn metadata_path(&self) -> PathBuf {
        self.base_path.join("metadata.bin")
    }

    /// Ensures the collection's directory exists.
    pub fn ensure_collection_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.base_path)
    }

    /// Appends a document operation to the collection's logfile.
    ///
    /// ## Arguments
    ///
    /// * `operation` - The [`Operation`] to append.
    /// * `document` - The [`BsonDocument`] to append.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`u64`]) with the file offset where the entry was written,
    /// or [`Err`]\([`io::Error`]) if the write failed.
    pub fn append_to_log(&self, operation: &Operation, document: &BsonDocument) -> io::Result<u64> {
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

        let bson_bytes = log_entry
            .to_vec()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(&bson_bytes)?;
        writeln!(file)?; // Add a newline to separate entries

        let offset = file.stream_position()? - bson_bytes.len() as u64 - 1;

        Ok(offset)
    }

    /// Reads all log entries from the collection's logfile.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with an iterator yielding ([`LogEntry`], [`u64`] offset) pairs,
    /// or [`Err`]\([`io::Error`]) if the read failed.
    pub fn read_log_entries(
        &self,
    ) -> io::Result<impl Iterator<Item = io::Result<(LogEntry, u64)>> + 'static> {
        let logfile_path = self.logfile_path();
        let mut file = if logfile_path.exists() {
            Some(File::open(&logfile_path)?)
        } else {
            None
        };

        let file_len = match &file {
            Some(f) => f.metadata()?.len(),
            None => 0,
        };

        let mut offset = 0u64;

        Ok(std::iter::from_fn(move || {
            let f = file.as_mut()?;

            loop {
                if offset + 4 > file_len {
                    return None;
                }

                let mut len_buf = [0u8; 4];
                if let Err(e) = f.seek(SeekFrom::Start(offset)) {
                    return Some(Err(e));
                }
                if let Err(e) = f.read_exact(&mut len_buf) {
                    return Some(Err(e));
                }
                let length = u32::from_le_bytes(len_buf) as u64;

                if offset + length > file_len {
                    return None;
                }

                if let Err(e) = f.seek(SeekFrom::Start(offset)) {
                    return Some(Err(e));
                }
                let mut entry_buf = vec![0u8; length as usize];
                if let Err(e) = f.read_exact(&mut entry_buf) {
                    return Some(Err(e));
                }

                let entry_offset = offset;
                offset += length;

                if offset < file_len {
                    let mut newline_buf = [0u8; 1];
                    if f.read_exact(&mut newline_buf).is_ok() && newline_buf[0] == b'\n' {
                        offset += 1;
                    }
                }

                match bson::Document::from_reader(entry_buf.as_slice()) {
                    Ok(log_doc) => {
                        let timestamp = log_doc
                            .get_str("timestamp")
                            .unwrap_or("unknown")
                            .to_string();
                        let operation_str = log_doc.get_str("operation").unwrap_or("unknown");
                        let operation = operation_str
                            .parse::<Operation>()
                            .unwrap_or(Operation::Insert);
                        let document = log_doc
                            .get_document("document")
                            .cloned()
                            .unwrap_or_default();

                        return Some(Ok((
                            LogEntry {
                                timestamp,
                                operation,
                                document,
                            },
                            entry_offset,
                        )));
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }))
    }

    /// Reads a single log entry at the specified offset.
    ///
    /// ## Arguments
    ///
    /// * `base_path` - The base path for the collection that needs to be read.
    /// * `offset` - The byte offset in the logfile where the entry begins.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`LogEntry`]) if successful,
    /// or [`Err`]\([`io::Error`]) if the offset is invalid or the read failed.
    pub fn read_entry_at(base_path: &Path, offset: u64) -> io::Result<LogEntry> {
        let log_path = base_path.join("logfile.log");
        if !log_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Logfile does not exist",
            ));
        }
        let mut logs = File::open(log_path)?;

        logs.seek(SeekFrom::Start(offset))?;
        let mut len_buf = [0u8; 4];
        logs.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        logs.seek(SeekFrom::Start(offset))?;
        let mut entry_buf = vec![0u8; len];
        logs.read_exact(&mut entry_buf)?;

        let log_doc = bson::Document::from_reader(entry_buf.as_slice()).map_err(|e| {
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
        let operation = operation_str
            .parse::<Operation>()
            .unwrap_or(Operation::Insert);
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

    /// Compacts the logfile by getting the final state of each document from the primary index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the logfile was compacted successfully,
    /// or [`Err`]\([`io::Error`]) if the compaction failed.
    pub fn compact_logfile(&mut self) -> io::Result<()> {
        let logfile_path = self.logfile_path();

        if self.primary_index.is_empty() {
            return Ok(());
        }

        let temp_path = logfile_path.with_extension("tmp");
        let mut temp_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;

        for result in self.primary_index.all_entries()? {
            let (_, offset) = result?;
            let log_entry = Self::read_entry_at(&self.base_path, offset)?;
            let document = log_entry.document;
            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut log_entry = BsonDocument::new();
            log_entry.insert("timestamp", Bson::String(timestamp));
            log_entry.insert(
                "operation",
                Bson::String(Operation::Insert.as_str().to_string()),
            );
            log_entry.insert("document", Bson::Document(document));

            let bson_bytes = log_entry
                .to_vec()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            temp_file.write_all(&bson_bytes)?;
            writeln!(temp_file)?;
        }

        fs::remove_file(&logfile_path)?;
        fs::rename(temp_path, logfile_path)?;
        self.build_primary_index()?;

        Ok(())
    }

    /// Writes the collection's metadata to the metadata file.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the write failed.
    pub fn write_metadata(&self) -> io::Result<()> {
        self.ensure_collection_dir()?;

        let metadata_path = self.metadata_path();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(metadata_path)?;

        let mut metadata = BsonDocument::new();
        metadata.insert("name", Bson::String(self.name.to_string()));
        metadata.insert("inserts", Bson::Int64(self.inserts as i64));
        metadata.insert("schema", Bson::Document(schema_to_document(&self.schema)));

        let bson_bytes = metadata
            .to_vec()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(&bson_bytes)?;
        Ok(())
    }

    /// Reads the collection's metadata from the metadata file.
    ///
    /// ## Arguments
    ///
    /// * `base_path` - The base directory path where collections are stored.
    /// * `name` - The name of the collection to read.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if successful,
    /// or [`Err`]\([`io::Error`]) if the read failed.
    pub fn read_metadata(base_path: impl AsRef<Path>, name: &str) -> io::Result<Collection> {
        let collection_dir = base_path.as_ref().join(name);
        let metadata_path = collection_dir.join("metadata.bin");

        if !metadata_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Metadata file not found: {}", metadata_path.display()),
            ));
        }

        let contents = fs::read(&metadata_path)?;
        let metadata: BsonDocument = bson::Document::from_reader(&mut contents.as_slice())
            .map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid BSON: {}", e))
            })?;
        let stored_name = metadata.get_str("name").unwrap_or("unknown");
        let inserts = metadata.get_i64("inserts").unwrap_or(0) as u64;
        let schema =
            schema_from_document(metadata.get_document("schema").cloned().unwrap_or_default());

        let mut collection =
            Collection::new(stored_name, schema, base_path.as_ref()).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid schema: {}", e))
            })?;
        collection.inserts = inserts;
        Ok(collection)
    }

    /// Reads the log entries and builds the primary index from scratch.
    /// This is a cleanup utility to avoid corrupted indices.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if failed.
    pub fn build_primary_index(&mut self) -> io::Result<()> {
        self.primary_index.clear()?;

        for result in self.read_log_entries()? {
            let (log_entry, offset) = result?;
            let doc_id = self
                .get_doc_id_from_bson(&log_entry.document)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unable to find ID for log entry",
                    )
                })?;

            match log_entry.operation {
                Operation::Insert => {
                    self.primary_index.insert(&doc_id, offset)?;
                }
                Operation::Update => {
                    self.primary_index.update(&doc_id, offset)?;
                }
                Operation::Delete => {
                    self.primary_index.remove(&doc_id)?;
                }
            }
        }

        Ok(())
    }

    /// Creates a [`Collection`] from existing files on disk.
    ///
    /// ## Arguments
    ///
    /// * `base_path` - The base directory path where collections are stored.
    /// * `name` - The name of the collection to load.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if successful,
    /// or [`Err`]\([`io::Error`]) if the load failed.
    pub fn from_files(base_path: impl AsRef<Path>, name: &str) -> io::Result<Collection> {
        let mut collection = Self::read_metadata(base_path.as_ref(), name)?;
        collection.build_primary_index()?;
        collection.compact_logfile()?;

        Ok(collection)
    }

    /// Deletes the entire collection directory and all its files.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the deletion failed.
    pub fn delete_collection_files(&self) -> io::Result<()> {
        if self.base_path.exists() {
            fs::remove_dir_all(&self.base_path)?;
        }
        Ok(())
    }
}
