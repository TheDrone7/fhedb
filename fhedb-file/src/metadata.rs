//! This module defines the MetadataFileIO trait.
//!
//! This trait defines various file operations for the database metadata
//! - Reading metadata from a DB file.
//! - Creating a DB file with metadata.
//! - Updating a DB file with metadata.
//!
//! It also contains the implementations of the trait for the [`DbMetadata`] structure.
//!
//! [`DbMetadata`]: fhedb_core::metadata::database::DbMetadata

use std::fs::File;
use std::io::prelude::*;

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::{bson, DbMetadata};

/// The trait that defines the file operations for the database metadata.
pub trait MetadataFileIO {
    /// Read the file with the specified path, then
    /// parse and return the database metadata.
    ///
    /// # Panics
    ///
    /// This function should panic if
    /// * The file does not exist.
    /// * The file is empty.
    /// * The file cannot be read.
    /// * The database metadata cannot be parsed (file is corrupted or invalid).
    fn read_file(path: &str) -> Result<DbMetadata>;
    /// Create a file with the specified path
    /// and write the database metadata to the file.
    ///
    /// # Panics
    ///
    /// This function should panic if
    /// * The file cannot be created. (e.g. invalid path)
    /// * The database metadata cannot be serialized.
    /// * The database metadata cannot be written to the file.
    fn create_file(&self, path: &str) -> Result<()>;
    /// Modify the file with the specified path, replacing
    /// the existing metadata with the current metadata.
    ///
    /// # Panics
    ///
    /// This function should panic if
    /// * The file cannot be opened.
    /// * The metadata size cannot be read.
    /// * The file cannot be seeked.
    /// * The file cannot be read.
    /// * The file cannot be written to.
    fn update_file(&self, path: &str) -> Result<()>;
}

impl MetadataFileIO for DbMetadata {
    /// Read a file and return the database metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be read.
    ///
    /// # Returns
    ///
    /// [`Result`] indicating success or failure.
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
    /// let result = DbMetadata::read_file("test.fhedb");
    ///
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap().name, "test");
    ///
    /// // Clean up
    /// std::fs::remove_file("test.fhedb").unwrap();
    /// ```
    fn read_file(path: &str) -> Result<DbMetadata> {
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

    /// The create_file function creates a file with the database metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path to the file.
    ///
    /// # Returns
    ///
    /// [`Result`] indicating success or failure.
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

    /// Update a file, replacing appropriate contents with current metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to be updated.
    ///
    /// # Returns
    ///
    /// [`Result`] indicating success or failure.
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
