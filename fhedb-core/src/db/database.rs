use crate::db::collection::Collection;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a database containing multiple collections.
///
/// A database is a logical grouping of collections with a shared name and base path.
/// It manages the lifecycle and organization of collections within the database.
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
    /// * `base_path` - The base path where the database will be stored.
    ///
    /// ## Returns
    ///
    /// Returns a new [`Database`] instance.
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
    ///
    /// ## Returns
    ///
    /// Returns a reference to the [`PathBuf`] representing the database's base path.
    pub fn path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Adds a collection to the database.
    ///
    /// ## Arguments
    ///
    /// * `collection` - The collection to add to the database.
    ///
    /// ## Returns
    ///
    /// Returns `true` if the collection was added, `false` if a collection with the same name already exists.
    pub fn add_collection(&mut self, collection: Collection) -> bool {
        let name = collection.name.clone();
        if !self.collections.contains_key(&name) {
            self.collections.insert(name, collection);
            true
        } else {
            false
        }
    }

    /// Removes a collection from the database.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to remove.
    ///
    /// ## Returns
    ///
    /// Returns the removed collection if it existed, or [`None`] if it wasn't found.
    pub fn remove_collection(&mut self, collection_name: &str) -> Option<Collection> {
        self.collections.remove(collection_name)
    }

    /// Checks if a collection exists in the database.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to check.
    ///
    /// ## Returns
    ///
    /// Returns `true` if the collection exists, `false` otherwise.
    pub fn has_collection(&self, collection_name: &str) -> bool {
        self.collections.contains_key(collection_name)
    }

    /// Returns a list of all collection names in the database.
    ///
    /// ## Returns
    ///
    /// Returns a vector of collection names.
    pub fn collection_names(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Returns the number of collections in the database.
    ///
    /// ## Returns
    ///
    /// Returns the count of collections as a [`usize`].
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }

    /// Retrieves a collection from the database.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns an [`Option`] containing a reference to the collection if it exists,
    /// or [`None`] if it's not found.
    pub fn get_collection(&self, collection_name: &str) -> Option<&Collection> {
        self.collections.get(collection_name)
    }

    /// Retrieves a mutable reference to a collection from the database.
    ///
    /// ## Arguments
    ///
    /// * `collection_name` - The name of the collection to retrieve.
    ///
    /// ## Returns
    ///
    /// Returns an [`Option`] containing a mutable reference to the collection if it exists,
    /// or [`None`] if it's not found.
    pub fn get_collection_mut(&mut self, collection_name: &str) -> Option<&mut Collection> {
        self.collections.get_mut(collection_name)
    }

    /// Clears all collections from the database.
    pub fn clear_collections(&mut self) {
        self.collections.clear();
    }
}
