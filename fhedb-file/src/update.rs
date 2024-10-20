//! This module defines the FileUpdate trait.
//!
//! It also contains the implementations for structures defined in the fhedb-core crate.

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::*;
use std::io::prelude::*;

/// The FileUpdate trait defines the function for updating files.
pub trait FileUpdate {
    /// Update a file, replacing appropriate contents with current structure.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be updated.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    fn update_file(&self, path: &str) -> Result<()>;
}

/// Implementing FileUpdate trait for DbMetadata.
impl FileUpdate for DbMetadata {
    /// Update a file, replacing appropriate contents with current metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be updated.
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
    /// let mut metadata = DbMetadata::new("test".to_owned());
    /// metadata.create_file("test.fhedb");
    ///
    ///
    /// metadata.name = "new_name".to_owned();
    /// let result = metadata.update_file("test.fhedb");
    ///
    /// assert!(result.is_ok());
    ///
    /// // Clean up
    /// std::fs::remove_file("test.fhedb").unwrap();
    /// ```
    fn update_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .map_err(|_| Error::new("Could not open file", path.to_str().unwrap_or("")))?;

        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)
            .map_err(|_| Error::new("Could not read metadata size", path.to_str().unwrap_or("")))?;

        let size = u32::from_le_bytes(buffer) as u64;

        file.seek(std::io::SeekFrom::Start(size as u64))
            .map_err(|_| Error::new("Could not seek file", path.to_str().unwrap_or("")))?;

        let mut remaining = Vec::new();
        file.read_to_end(&mut remaining)
            .map_err(|_| Error::new("Could not read file", path.to_str().unwrap_or("")))?;

        let dbm = self.to_bytes();

        file.seek(std::io::SeekFrom::Start(0))
            .map_err(|_| Error::new("Could not seek file", path.to_str().unwrap_or("")))?;

        file.write_all(&dbm)
            .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))?;

        file.write_all(&remaining)
            .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))?;

        Ok(())
    }
}
