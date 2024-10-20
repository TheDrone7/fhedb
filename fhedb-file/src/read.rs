//! This module defines the FileRead trait.
//!
//! It also contains the implementations for structures defined in the fhedb-core crate.

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::DbMetadata;

/// The FileRead trait defines the function for reading files.
pub trait FileRead<T> {
    /// Read a file and return.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be read.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    /// File content parsed into the type T if successful.
    /// Otherwise an error.
    fn from_file(path: &str) -> Result<T>;
}

/// Implementing FileRead trait for DbMetadata.
impl FileRead<DbMetadata> for DbMetadata {
    /// Read a file and return the database metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be read.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    /// Database metadata if successful.
    ///
    /// # Example
    ///
    /// ```
    /// use fhedb_file::prelude::*;
    /// use fhedb_core::prelude::*;
    ///
    /// let metadata = DbMetadata::new("test".to_owned());
    /// metadata.create_file("test.fhedb").unwrap();
    ///
    /// let result = DbMetadata::from_file("test.fhedb");
    ///
    /// assert!(result.is_ok());
    ///
    /// // Clean up
    /// std::fs::remove_file("test.fhedb").unwrap();
    /// ```
    fn from_file(path: &str) -> Result<Self> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(Error::new(
                "File does not exist",
                path.to_str().unwrap_or(""),
            ));
        }

        if let Ok(db) = std::fs::read(path) {
            if db.len() < 4 {
                return Err(Error::new("File is empty", path.to_str().unwrap_or("")));
            }

            let size = u32::from_le_bytes([db[0], db[1], db[2], db[3]]) as usize;

            let db = Self::from(&db[0..size]);
            if let Ok(db) = db {
                Ok(db)
            } else {
                Err(Error::new(
                    "Could not parse file",
                    path.to_str().unwrap_or(""),
                ))
            }
        } else {
            Err(Error::new(
                "Could not read file",
                path.to_str().unwrap_or(""),
            ))
        }
    }
}
