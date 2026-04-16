//! # Secondary Index
//!
//! Provides a B+ Tree backed secondary index that maps field value and document IDs
//! to data offsets. It uses a (field value | doc ID) composite key to turn every field
//! value into something unique to bypass the uniqueness requirement of B+ tree entries.

use super::{pager::Pager, tree::BPlusTree};
use crate::document::DocId;
use bson::Bson;
use std::{fs::remove_file, io, path::PathBuf};

/// A B+ Tree backed secondary index for a collection.
#[derive(Debug)]
pub struct SecondaryIndex {
    /// The name of the field being indexed.
    field_name: String,
    /// The base directory path for collection storage.
    base_path: PathBuf,
    /// The B+ tree structure for this index.
    tree: BPlusTree,
}

impl SecondaryIndex {
    /// Create a new [`SecondaryIndex`] for the specified field.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field being indexed.
    /// * `base_path` - The base directory path for collection storage.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with the created [`SecondaryIndex`] on success,
    /// or an [`io::Error`] on failure.
    pub fn new(field_name: impl Into<String>, base_path: impl Into<PathBuf>) -> io::Result<Self> {
        let field_name = field_name.into();
        let base_path = base_path.into();

        let path = base_path.join(format!("{}.idx", field_name));
        let pager = Pager::new(path)?;
        let tree = BPlusTree::open(pager)?;

        Ok(Self {
            field_name,
            base_path,
            tree,
        })
    }

    /// Insert a field value, document ID and its offset into the index.
    ///
    /// ## Arguments
    ///
    /// * `value` - The [`Bson`] field value to be indexed.
    /// * `id` - The [`DocId`] for the document containing the field value.
    /// * `offset` - The offset in the logfile where the document is stored.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] on successful insertion,
    /// or an [`io::Error`] if the insertion failed.
    pub fn insert(&mut self, value: &Bson, id: &DocId, offset: u64) -> io::Result<()> {
        let key = Self::encode_key(value, id)?;
        self.tree.insert(&key, &offset.to_le_bytes())
    }

    /// Remove an entry from the index based on the field value and document ID.
    ///
    /// ## Arguments
    ///
    /// * `value` - The [`Bson`] field value to be removed.
    /// * `id` - The [`DocId`] for the document containing the field value.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with the removed offset if the entry was found and removed,
    /// or [`Ok`] with `None` if the entry was not found,
    /// or an [`io::Error`] if the removal failed.
    pub fn remove(&mut self, value: &Bson, id: &DocId) -> io::Result<Option<u64>> {
        let key = Self::encode_key(value, id)?;
        let offset = self.tree.get(&key)?;
        self.tree.delete(&key)?;
        Ok(offset.map(u64::from_le_bytes))
    }

    /// Update an existing entry in the index.
    ///
    /// ## Arguments
    ///
    /// * `old_value` - The old [`Bson`] field value before the update.
    /// * `new_value` - The new [`Bson`] field value after the update.
    /// * `id` - The [`DocId`] for the document being updated.
    /// * `new_offset` - The new offset in the logfile where the updated document is.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] on successful update,
    /// or an [`io::Error`] if the update failed.
    pub fn update(
        &mut self,
        old_value: &Bson,
        new_value: &Bson,
        id: &DocId,
        new_offset: u64,
    ) -> io::Result<()> {
        if old_value != new_value {
            self.remove(old_value, id)?;
            self.insert(new_value, id, new_offset)
        } else {
            let key = Self::encode_key(new_value, id)?;
            self.tree.update(&key, &new_offset.to_le_bytes())
        }
    }

    /// Retrieve all entries in the index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] containing an iterator over all entries in the index, where each entry is a tuple of
    /// the [`Bson`] field value, [`DocId`], and offset. Each entry is wrapped in an [`std::io::Result`].
    pub fn all_entries(
        &self,
    ) -> io::Result<impl Iterator<Item = io::Result<(Bson, DocId, u64)>> + '_> {
        let scan = self.tree.scan(None, None)?;
        Ok(scan.map(|result| {
            let (key, off) = result?;
            let (value, doc_id) = Self::decode_key(&key)?;
            let offset = u64::from_le_bytes(off);
            Ok((value, doc_id, offset))
        }))
    }

    /// Returns the next entry in the index after the specified field value and document ID.
    ///
    /// ## Arguments
    ///
    /// * after - A tuple of an optional [`Bson`] field value and an optional [`DocId`] representing
    ///   the last seen entry.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with the next entry as a tuple of the [`Bson`] field value, [`DocId`], and
    /// offset, or `None` if there are no more entries after the specified one,
    /// or an [`io::Error`] if the operation failed.
    pub fn next_entry(
        &self,
        after: (Option<&Bson>, Option<&DocId>),
    ) -> io::Result<Option<(Bson, DocId, u64)>> {
        let start_bytes = match after {
            (Some(value), None) => Some(Self::encode_prefix(value)?),
            (Some(value), Some(id)) => Some(Self::encode_key(value, id)?),
            _ => None,
        };

        let scan = self.tree.scan(start_bytes.as_deref(), None)?;

        for result in scan {
            let (key, off) = result?;
            let (value, doc_id) = Self::decode_key(&key)?;

            let (matches_value, matches_id) = match after {
                (Some(after_value), Some(after_id)) => (value == *after_value, doc_id == *after_id),
                (Some(after_value), None) => (value == *after_value, true),
                _ => (true, true),
            };

            if !(matches_value && matches_id) {
                let offset = u64::from_le_bytes(off);
                return Ok(Some((value, doc_id, offset)));
            }
        }

        Ok(None)
    }

    /// Checks if the index contains the specified entry.
    ///
    /// ## Arguments
    ///
    /// * `value` - The indexed [`Bson`] value to search for.
    /// * `id` - The [`DocId`] to search for.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(true)`] if the entry is found, [`Ok(false)`] otherwise,
    /// or an [`io::Error`] if the operation failed.
    pub fn contains_entry(&self, value: &Bson, id: &DocId) -> io::Result<bool> {
        let key = Self::encode_key(value, id)?;
        let result = self.tree.get(&key)?;
        Ok(result.is_some())
    }

    /// Retrieves the offset for the specified entry, if it exists.
    ///
    /// ## Arguments
    ///
    /// * `value` - The indexed [`Bson`] value to search for.
    /// * `id` - The [`DocId`] to search for.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(Some(offset))`] if the entry is found, [`Ok(None)`] otherwise,
    /// or an [`io::Error`] if the operation failed.
    pub fn get(&self, value: &Bson, id: &DocId) -> io::Result<Option<u64>> {
        let key = Self::encode_key(value, id)?;
        let result = self.tree.get(&key)?;
        Ok(result.map(u64::from_le_bytes))
    }

    /// Clear the index by deleting the file backing the B+ tree and creating a new empty tree.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] on successful clearing of the index,
    /// or an [`io::Error`] if the operation failed.
    pub fn clear(&mut self) -> io::Result<()> {
        let path = self.base_path.join(format!("{}.idx", &self.field_name));
        if path.exists() {
            remove_file(&path)?;
        }

        let pager = Pager::new(path)?;
        self.tree = BPlusTree::open(pager)?;

        Ok(())
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get the number of entries currently stored in the index.
    pub fn len(&self) -> usize {
        self.tree.len() as usize
    }

    /// Get the name of the field that this index is built on.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Encodes a value into a prefix representing the field part of the index.
    ///
    /// ## Arguments
    ///
    /// * `value` - The [`Bson`] value to be encoded.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with the encoded bytes on success,
    /// or an [`io::Error`] if the value type is unsupported.
    fn encode_prefix(value: &Bson) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        match value {
            Bson::Int32(i) => {
                bytes.push(1);
                let mapped = (*i as i64) ^ (1i64 << 63);
                bytes.extend_from_slice(&(mapped as u64).to_be_bytes());
            }
            Bson::Int64(i) => {
                bytes.push(1);
                let mapped = *i ^ (1i64 << 63);
                bytes.extend_from_slice(&(mapped as u64).to_be_bytes());
            }
            Bson::Double(f) => {
                bytes.push(2);
                let bits = f.to_bits();
                let mapped = if (bits >> 63) == 1 {
                    !bits
                } else {
                    bits ^ (1u64 << 63)
                };
                bytes.extend_from_slice(&mapped.to_be_bytes());
            }
            Bson::Boolean(b) => {
                bytes.push(3);
                bytes.push(if *b { 1 } else { 0 });
            }
            Bson::String(s) => {
                let s_bytes = s.as_bytes();
                let limit = s_bytes.len().min(500);
                bytes.push(4);
                bytes.extend_from_slice(&(limit as u16).to_be_bytes());
                bytes.extend_from_slice(&s_bytes[..limit]);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported field type for secondary index",
                ));
            }
        }

        Ok(bytes)
    }

    /// Encodes a field value and a document ID into a composite key.
    ///
    /// ## Arguments
    ///
    /// * `value` - The [`Bson`] field value.
    /// * `id` - The [`DocId`] for the document.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with the encoded composite key on success,
    /// or an [`io::Error`] if the field value type is unsupported.
    fn encode_key(value: &Bson, id: &DocId) -> io::Result<Vec<u8>> {
        let mut bytes = Self::encode_prefix(value)?;
        bytes.extend_from_slice(&id.to_bytes());
        Ok(bytes)
    }

    /// Decodes a composite key into its field value and document ID components.
    ///
    /// ## Arguments
    ///
    /// * `key` - The composite key bytes to be decoded.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] with a tuple of the decoded field value and document ID on success,
    /// or an [`io::Error`] if the key is invalid or contains unsupported field types.
    fn decode_key(key: &[u8]) -> io::Result<(Bson, DocId)> {
        if key.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty key"));
        }

        let field_type = key[0];
        let (value, doc_id_start) = match field_type {
            1 => {
                if key.len() < 9 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid key length for Int32/Int64",
                    ));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&key[1..9]);
                let mapped = u64::from_be_bytes(arr);
                let original = (mapped ^ (1u64 << 63)) as i64;
                (Bson::Int64(original), 9)
            }
            2 => {
                if key.len() < 9 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid key length for Double",
                    ));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&key[1..9]);
                let mapped = u64::from_be_bytes(arr);
                let bits = if (mapped >> 63) == 1 {
                    mapped ^ (1u64 << 63)
                } else {
                    !mapped
                };
                (Bson::Double(f64::from_bits(bits)), 9)
            }
            3 => {
                if key.len() < 2 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid key length for Boolean",
                    ));
                }
                (Bson::Boolean(key[1] != 0), 2)
            }
            4 => {
                if key.len() < 3 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid key length for String prefix",
                    ));
                }
                let mut arr = [0u8; 2];
                arr.copy_from_slice(&key[1..3]);
                let len = u16::from_be_bytes(arr) as usize;

                if key.len() < 3 + len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid key length for String data",
                    ));
                }

                let s = String::from_utf8(key[3..3 + len].to_vec()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in String key")
                })?;

                (Bson::String(s), 3 + len)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unsupported field type in key",
                ));
            }
        };

        let doc_id = DocId::from_bytes(&key[doc_id_start..]);
        Ok((value, doc_id))
    }
}
