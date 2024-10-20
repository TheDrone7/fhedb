//! This module defines the FileDelete trait.
//!
//! It also contains the implementations for structures defined in the fhedb-core crate.

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::*;

/// The FileDelete trait defines the function for deleting files.
pub trait FileDelete {
    /// Delete a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be deleted.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    fn delete_file(&self, path: &str) -> Result<()>;
}

/// Implementing FileDelete trait for DbMetadata.
impl FileDelete for DbMetadata {
    /// Deletes a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be deleted.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    ///
    /// # Example
    ///
    /// ```
    /// use fhedb_file::prelude::*;
    /// use fhedb_core::prelude::*;
    ///
    /// let db = DbMetadata::new("test".to_owned());
    /// let result = db.delete_file("file.txt");
    /// ```
    fn delete_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|_| Error::new("Could not delete file", path.to_str().unwrap_or("")))
        } else {
            Err(Error::new(
                "File does not exist",
                path.to_str().unwrap_or(""),
            ))
        }
    }
}
