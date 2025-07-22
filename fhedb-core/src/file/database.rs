use crate::db::{collection::Collection, database::Database};
use crate::file::collection::CollectionFileOps;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Trait for file operations on databases.
///
/// This trait provides functionality for loading databases and collections from disk.
pub trait DatabaseFileOps {
    /// Loads a database from existing files on disk.
    ///
    /// This method creates a new database instance and loads all collections
    /// from subdirectories found in the database's directory. Each subdirectory
    /// is treated as a collection and loaded using the collection's `from_files` method.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the database.
    /// * `base_path` - The base path where the database is stored.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Database`]) if the database was loaded successfully,
    /// or [`Err`]\([`std::io::Error`]) if the database could not be loaded.
    fn from_files(
        name: impl Into<String>,
        base_path: impl Into<PathBuf>,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

impl DatabaseFileOps for Database {
    fn from_files(
        name: impl Into<String>,
        base_path: impl Into<PathBuf>,
    ) -> Result<Self, std::io::Error> {
        // Create a new database object with the given parameters
        let mut database = Self::new(name, base_path);

        // Check if the database directory exists
        if !database.base_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Database directory not found: {}",
                    database.base_path.display()
                ),
            ));
        }

        // Read all entries in the database directory
        let entries = fs::read_dir(&database.base_path)?;

        // For each directory under the database path, load it as a collection
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process directories (skip files)
            if path.is_dir() {
                if let Some(collection_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Try to load the collection from files using the trait method
                    let collection = Collection::from_files(&database.base_path, collection_name)?;
                    database
                        .collections
                        .insert(collection_name.to_string(), collection);
                }
            }
        }

        Ok(database)
    }
}
