//! Collection index management.
//!
//! Provides index amangement utilities for a collection.

use crate::{
    collection::Collection,
    document::DocId,
    errors::{Error, Result},
    index::secondary::SecondaryIndex,
};
use bson::Bson;
use std::{fs::remove_file, io};

/// Check if a value is indexable (i.e., can be used as an index key).
///
/// ## Arguments
///
/// * `value` - The value to check for indexability.
fn is_indexable(value: &bson::Bson) -> bool {
    matches!(
        value,
        Bson::Int32(_) | Bson::Int64(_) | Bson::Double(_) | Bson::Boolean(_) | Bson::String(_)
    )
}

impl Collection {
    /// Insert entries into all secondary indices for a given document.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [bson document](bson::Document) being inserted.
    /// * `id` - The [`DocId`] for the inserted document.
    /// * `offset` - The offset at which the document is stored in the log file.
    ///
    /// ## Returns
    /// Returns [`Ok`] if all indices were updated successfully,
    /// or an [`io::Error`] if any index operation failed.
    pub(crate) fn insert_secondary_indices(
        &mut self,
        doc: &bson::Document,
        id: &DocId,
        offset: u64,
    ) -> io::Result<()> {
        for (field_name, index) in &mut self.secondary_indices {
            if let Some(value) = doc.get(field_name)
                && is_indexable(value)
            {
                index.insert(value, id, offset)?;
            }
        }

        Ok(())
    }

    /// Remove entries from all secondary indices for a given document.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [bson document](bson::Document) being removed.
    /// * `id` - The [`DocId`] for the removed document.
    pub(crate) fn remove_secondary_indices(
        &mut self,
        doc: &bson::Document,
        id: &DocId,
    ) -> io::Result<()> {
        for (field_name, index) in &mut self.secondary_indices {
            if let Some(value) = doc.get(field_name)
                && is_indexable(value)
            {
                index.remove(value, id)?;
            }
        }

        Ok(())
    }

    /// Update entries in all secondary indices for a given document.
    ///
    /// ## Arguments
    /// * `old_doc` - The [bson document](bson::Document) before the update.
    /// * `new_doc` - The [bson document](bson::Document) after the update.
    /// * `id` - The [`DocId`] for the updated document.
    /// * `new_offset` - The new offset at which the updated document is stored in.
    pub(crate) fn update_secondary_indices(
        &mut self,
        old_doc: &bson::Document,
        new_doc: &bson::Document,
        id: &DocId,
        new_offset: u64,
    ) -> io::Result<()> {
        for (field_name, index) in &mut self.secondary_indices {
            let old_val = old_doc.get(field_name).filter(|v| is_indexable(v));
            let new_val = new_doc.get(field_name).filter(|v| is_indexable(v));

            match (old_val, new_val) {
                (Some(old), Some(new)) => {
                    index.update(old, new, id, new_offset)?;
                }
                (Some(old), None) => {
                    index.remove(old, id)?;
                }
                (None, Some(new)) => {
                    index.insert(new, id, new_offset)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a secondary index on a specified field.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to index.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] if the index was created successfully,
    /// or an [`Error`] if unable to create the index.
    pub fn create_secondary_index(&mut self, field_name: &str) -> Result<()> {
        if field_name == self.id_field {
            return Err(Error::FieldNotIndexable(
                field_name.to_string(),
                "Field is ID field, already indexed.".to_string(),
            ));
        }

        if !self.schema.fields.contains_key(field_name) {
            return Err(Error::FieldNotIndexable(
                field_name.to_string(),
                "Field does not exist in schema.".to_string(),
            ));
        }

        if self.secondary_indices.contains_key(field_name) {
            return Err(Error::IndexAlreadyExists(field_name.to_string()));
        }

        let idx = SecondaryIndex::new(field_name, &self.base_path)?;
        self.secondary_indices.insert(field_name.to_string(), idx);

        self.build_secondary_index(field_name)?;

        Ok(())
    }

    /// Drop the secondary index on a specified field.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field whose index should be dropped.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`] if the index was dropped successfully,
    /// or an [`Error`] if the index does not exist or cannot be dropped.
    pub fn drop_secondary_index(&mut self, field_name: &str) -> Result<()> {
        if !self.secondary_indices.contains_key(field_name) {
            return Err(Error::IndexNotFound(field_name.to_string()));
        }

        self.secondary_indices.remove(field_name);
        let index_path = self.base_path.join(format!("{}.idx", field_name));
        if index_path.exists() {
            remove_file(&index_path)?;
        }

        Ok(())
    }
}
