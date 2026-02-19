//! # Database
//!
//! Provides the core [`Database`] type and its collection management operations.

pub mod file;

use crate::{collection::Collection, schema::Schema};
use std::{collections::HashMap, path::PathBuf};

/// A named group of [`Collection`]s stored under a shared base path.
#[derive(Debug, Clone)]
pub struct Database {
    /// The name of the database.
    pub name: String,
    /// The base path where the database and its collections are stored.
    pub base_path: PathBuf,
    /// The collections stored in this database.
    pub(crate) collections: HashMap<String, Collection>,
}

impl Database {
    /// Creates a new [`Database`] with the given name and base path.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the database.
    /// * `base_path` - The base directory where the database will be stored.
    pub fn new(name: impl Into<String>, base_path: impl Into<PathBuf>) -> Self {
        let name = name.into();
        let temp_path = base_path.into();
        let base_path = temp_path.join(&name);

        Self {
            name,
            base_path,
            collections: HashMap::new(),
        }
    }

    /// Returns the full path for the database directory.
    pub fn path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Creates a new collection in the database.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the collection.
    /// * `schema` - The [`Schema`] describing document structure.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Collection`]) if created successfully,
    /// or [`Err`]\([`String`]) if the name is taken or creation failed.
    pub fn create_collection(
        &mut self,
        name: impl Into<String>,
        schema: Schema,
    ) -> Result<&Collection, String> {
        let collection_name = name.into();

        if self.collections.contains_key(&collection_name) {
            return Err(format!("Collection '{}' already exists", collection_name));
        }

        let collection = Collection::new(collection_name.clone(), schema, &self.base_path)?;

        collection
            .write_metadata()
            .map_err(|e| format!("Failed to write collection metadata: {}", e))?;

        self.collections.insert(collection_name.clone(), collection);

        Ok(self.collections.get(&collection_name).unwrap())
    }

    /// Drops a collection from the database and deletes its files.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to drop.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`String`]) with the dropped collection name,
    /// or [`Err`]\([`String`]) if not found or deletion failed.
    pub fn drop_collection(&mut self, collection_name: &str) -> Result<String, String> {
        if let Some(collection) = self.collections.remove(collection_name) {
            if let Err(e) = collection.delete_collection_files() {
                self.collections.insert(collection_name.to_string(), collection);
                return Err(format!("Failed to delete collection files: {}", e));
            }

            Ok(collection_name.to_string())
        } else {
            Err(format!("Collection '{}' not found", collection_name))
        }
    }

    /// Checks if a collection exists in the database.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to check.
    pub fn has_collection(&self, collection_name: &str) -> bool {
        self.collections.contains_key(collection_name)
    }

    /// Returns all collection names in the database.
    pub fn collection_names(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Returns the number of collections in the database.
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }

    /// Retrieves a collection by name.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Collection`]) if found, or [`None`] if not found.
    pub fn get_collection(&self, collection_name: &str) -> Option<&Collection> {
        self.collections.get(collection_name)
    }

    /// Retrieves a mutable reference to a collection by name.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`]\([`Collection`]) if found, or [`None`] if not found.
    pub fn get_collection_mut(&mut self, collection_name: &str) -> Option<&mut Collection> {
        self.collections.get_mut(collection_name)
    }

    /// Removes all collections from the in-memory database.
    pub fn clear_collections(&mut self) {
        self.collections.clear();
    }
}
