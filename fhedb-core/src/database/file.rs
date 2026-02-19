//! # Database File Operations
//!
//! Provides file I/O operations for loading databases from disk.

use crate::{collection::Collection, database::Database};
use std::{fs, io, path::PathBuf};

/// File I/O operations for database persistence.
impl Database {
    /// Loads a [`Database`] from existing files on disk.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the database.
    /// * `base_path` - The base path where the database is stored.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Database`]) if loaded successfully,
    /// or [`Err`]\([`io::Error`]) if the load failed.
    pub fn from_files(
        name: impl Into<String>,
        base_path: impl Into<PathBuf>,
    ) -> Result<Self, std::io::Error> {
        let mut database = Self::new(name, base_path);

        if !database.base_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Database directory not found: {}",
                    database.base_path.display()
                ),
            ));
        }

        let entries = fs::read_dir(&database.base_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && let Some(collection_name) = path.file_name().and_then(|n| n.to_str())
            {
                let collection = Collection::from_files(&database.base_path, collection_name)?;
                database
                    .collections
                    .insert(collection_name.to_string(), collection);
            }
        }

        Ok(database)
    }
}
