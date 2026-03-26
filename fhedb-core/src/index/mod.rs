//! # Index
//!
//! Provides B+ tree index structures and operations.

/// The pager module - contains the pager implementation for managing index pages.
pub mod pager;

/// The node module - contains the B+ tree node structures and operations.
pub mod node;

/// The tree module - contains the B+ tree structure and operations for managing the index.
pub mod tree;

use crate::document::DocId;
use std::{io, path::PathBuf};
use tree::BPlusTree;

/// A B+ tree backed index for a single field in a collection.
#[derive(Debug)]
pub struct CollectionIndex {
    /// The name of the indexed field.
    field_name: String,
    /// The base directory path for collection storage.
    base_path: PathBuf,
    /// The B+ tree structure for this index.
    tree: BPlusTree,
}

impl CollectionIndex {
    /// Creates a new [`CollectionIndex`] for the specified field.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field being indexed.
    /// * `base_path` - The base directory path for collection storage.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`CollectionIndex`]) if created successfully,
    /// or [`Err`]\([`io::Error`]) if the index could not be created.
    pub fn new(field_name: impl Into<String>, base_path: impl Into<PathBuf>) -> io::Result<Self> {
        let field_name = field_name.into();
        let base_path = base_path.into();
        let path = base_path.join(format!("{}.idx", &field_name));
        let pager = pager::Pager::new(path)?;
        let tree = BPlusTree::open(pager)?;
        Ok(Self {
            field_name,
            base_path,
            tree,
        })
    }

    /// Clears the index by removing the backing file and reinitializing.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the reset failed.
    pub fn clear(&mut self) -> io::Result<()> {
        let path = self.base_path.join(format!("{}.idx", &self.field_name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let pager = pager::Pager::new(path)?;
        self.tree = BPlusTree::open(pager)?;
        Ok(())
    }

    /// Checks if the index contains the specified document ID.
    ///
    /// ## Arguments
    ///
    /// * `id` - The document ID to check for in the index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`bool`]) indicating whether the ID exists,
    /// or [`Err`]\([`io::Error`]) if the lookup failed.
    pub fn contains_id(&self, id: &DocId) -> io::Result<bool> {
        let result = self.tree.get(&id.to_bytes())?;
        Ok(result.is_some())
    }

    /// Inserts a new document ID and its offset into the index.
    ///
    /// ## Arguments
    ///
    /// * `id` - The document ID to insert into the index.
    /// * `offset` - The offset in the log file for the respective entry.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the insertion failed.
    pub fn insert(&mut self, id: &DocId, offset: u64) -> io::Result<()> {
        self.tree.insert(&id.to_bytes(), &offset.to_le_bytes())
    }

    /// Retrieves the offset for a given document ID from the index.
    ///
    /// ## Arguments
    ///
    /// * `id` - The document ID to look up in the index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Some`]\([`u64`])) with the offset if found,
    /// [`Ok`]\([`None`]) if the ID does not exist,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn get(&self, id: &DocId) -> io::Result<Option<u64>> {
        let result = self.tree.get(&id.to_bytes())?;
        Ok(result.map(u64::from_le_bytes))
    }

    /// Removes a document ID and its offset from the index.
    ///
    /// ## Arguments
    ///
    /// * `id` - The document ID to remove from the index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Some`]\([`u64`])) with the removed offset if found,
    /// [`Ok`]\([`None`]) if the ID did not exist,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn remove(&mut self, id: &DocId) -> io::Result<Option<u64>> {
        let offset = self.tree.get(&id.to_bytes())?;
        self.tree.delete(&id.to_bytes())?;
        Ok(offset.map(u64::from_le_bytes))
    }

    /// Updates the offset for a given document ID in the index.
    ///
    /// ## Arguments
    ///
    /// * `id` - The document ID to update in the index.
    /// * `new_offset` - The new offset in the log file for the ID.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the key was not found or the update failed.
    pub fn update(&mut self, id: &DocId, new_offset: u64) -> io::Result<()> {
        self.tree.update(&id.to_bytes(), &new_offset.to_le_bytes())
    }

    /// Returns iterator over all document IDs currently stored in the index.
    pub fn all_ids(&self) -> io::Result<impl Iterator<Item = io::Result<DocId>> + '_> {
        let scan = self.tree.scan(None, None)?;
        Ok(scan.map(|result| {
            let (key_bytes, _) = result?;
            Ok(DocId::from_bytes(&key_bytes))
        }))
    }

    /// Returns iterator over document IDs and their offsets currently stored in the index.
    pub fn all_entries(&self) -> io::Result<impl Iterator<Item = io::Result<(DocId, u64)>> + '_> {
        let scan = self.tree.scan(None, None)?;
        Ok(scan.map(|result| {
            let (key_bytes, value_bytes) = result?;
            let id = DocId::from_bytes(&key_bytes);
            let offset = u64::from_le_bytes(value_bytes);

            Ok((id, offset))
        }))
    }

    pub fn next_id(&self, after: Option<&DocId>) -> io::Result<Option<DocId>> {
        let start_bytes = after.map(|id| id.to_bytes());
        let scan = self.tree.scan(start_bytes.as_deref(), None)?;
        for result in scan {
            let (key_bytes, _) = result?;
            let id = DocId::from_bytes(&key_bytes);
            if Some(&id) != after {
                return Ok(Some(id));
            }
        }

        Ok(None)
    }

    /// Checks if the index is empty (contains no entries).
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`bool`]) indicating whether the index is empty,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn is_empty(&self) -> io::Result<bool> {
        let mut result = self.tree.scan(None, None)?;
        Ok(result.next().is_none())
    }

    /// Returns the number of entries currently stored in the index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`usize`]) with the entry count,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn len(&self) -> io::Result<usize> {
        let result = self.tree.scan(None, None)?;
        Ok(result.count())
    }

    /// Returns the field name that this index is built on.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}
