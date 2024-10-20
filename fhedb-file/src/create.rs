//! This module defines the FileCreate trait.
//!
//! It also contains the implementations for structures defined in the fhedb-core crate.

use std::fs::File;
use std::io::prelude::*;

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::{bson, DbMetadata};

/// The FileCreate trait defines the function for creating files.
pub trait FileCreate {
    /// The create_file function creates a file with the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path to the file.
    ///
    /// # Returns
    ///
    /// A Result that contains a unit type or an Error.
    fn create_file(&self, path: &str) -> Result<()>;
}

/// The FileCreate trait implementation for DbMetadata.
impl FileCreate for DbMetadata {
    /// The create_file function creates a file with the database metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path to the file.
    ///
    /// # Returns
    ///
    /// A Result that contains a unit type or an Error.
    ///
    /// # Examples
    ///
    /// ```
    /// use fhedb_file::prelude::*;
    /// use fhedb_core::prelude::*;
    ///
    /// let metadata = DbMetadata::new("test".to_owned());
    /// let result = metadata.create_file("test.fhedb");
    ///
    /// assert!(result.is_ok());
    ///
    /// // Clean up
    /// std::fs::remove_file("test.fhedb").unwrap();
    /// ```
    fn create_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        let mut file = File::create(path)
            .map_err(|_| Error::new("Could not create file", path.to_str().unwrap_or("")))?;
        if let Ok(db) = bson::to_vec(self) {
            file.write_all(&db)
                .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))
        } else {
            Err(Error::new("Could not serialize database", ""))
        }
    }
}
