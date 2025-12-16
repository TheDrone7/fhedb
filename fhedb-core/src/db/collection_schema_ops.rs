use crate::db::collection::Collection;
use crate::db::document::DocId;
use crate::db::schema::{FieldDefinition, SchemaOps};
use crate::file::{collection::CollectionFileOps, types::Operation};

/// A trait for modifying collection schemas.
///
/// This trait provides methods to add and remove columns (fields) from a collection's schema.
/// Implementations should handle the persistence of schema changes and ensure data consistency.
pub trait CollectionSchemaOps {
    /// Checks if the schema contains a field with the given name.
    ///
    /// ## Arguments
    ///
    /// * `field` - The name of the field to check.
    ///
    /// ## Returns
    ///
    /// `true` if the field exists in the schema, `false` otherwise.
    fn has_field(&self, field: &str) -> bool;

    /// Validates a BSON document against this collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - A reference to the [`bson::Document`] to validate.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the document matches the schema. Returns [`Err(Vec<String>)`](Result::Err) containing error messages for each field that does not conform to the schema.
    fn validate_document(&self, doc: &bson::Document) -> Result<(), Vec<String>>;

    /// Adds a new field to the collection's schema.
    ///
    /// This method adds a new field definition to the collection. If the field already exists,
    /// it should return an error. The implementation should handle updating the schema and
    /// ensuring existing documents remain valid.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to add.
    /// * `field_definition` - The [`FieldDefinition`] specifying the type and default value.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`] if the field was successfully added,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn add_field(
        &mut self,
        field_name: String,
        field_definition: FieldDefinition,
    ) -> Result<(), String>;

    /// Removes a field from the collection's schema.
    ///
    /// This method removes a field definition from the collection. If the field doesn't exist,
    /// it should return an error. The implementation should handle updating the schema and
    /// managing existing document data that contains the removed field.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to remove.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`] if the field was successfully removed,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn remove_field(&mut self, field_name: &str) -> Result<(), String>;

    /// Modifies an existing field's definition in the collection's schema.
    ///
    /// This method updates the definition of an existing field. If the field doesn't exist,
    /// it should return an error. The implementation should validate that the new definition
    /// is compatible with existing data or handle data migration appropriately.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to modify.
    /// * `new_definition` - The new [`FieldDefinition`] for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`] if the field was successfully modified,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn modify_field(
        &mut self,
        field_name: &str,
        new_definition: FieldDefinition,
    ) -> Result<(), String>;

    /// Renames a field in the collection's schema.
    ///
    /// This method changes the name of an existing field while preserving its definition
    /// and data. If the source field doesn't exist or the target field name already exists,
    /// it should return an error.
    ///
    /// ## Arguments
    ///
    /// * `old_name` - The current name of the field.
    /// * `new_name` - The new name for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`] if the field was successfully renamed,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn rename_field(&mut self, old_name: &str, new_name: String) -> Result<(), String>;

    /// Applies default values to existing documents when a new field with a default is added.
    ///
    /// This method updates all existing documents in the collection to include the new field
    /// with its default value. It should be called after successfully adding a field with
    /// a default value to the schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the newly added field.
    /// * `field_definition` - The [`FieldDefinition`] containing the default value.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(Vec<DocId>)`] with the IDs of documents that were updated,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn apply_defaults_to_existing(
        &mut self,
        field_name: &str,
        field_definition: &FieldDefinition,
    ) -> Result<Vec<DocId>, String>;

    /// Removes field data from existing documents when a field is removed from the schema.
    ///
    /// This method updates all existing documents in the collection to remove the specified
    /// field. It should be called after successfully removing a field from the schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the removed field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(Vec<DocId>)`] with the IDs of documents that were updated,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn cleanup_removed_field(&mut self, field_name: &str) -> Result<Vec<DocId>, String>;

    /// Renames a field in all existing documents when a field is renamed in the schema.
    ///
    /// This method updates all existing documents in the collection to rename the specified
    /// field from the old name to the new name. It should be called after successfully
    /// renaming a field in the schema.
    ///
    /// ## Arguments
    ///
    /// * `old_field_name` - The current name of the field.
    /// * `new_field_name` - The new name for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(Vec<DocId>)`] with the IDs of documents that were updated,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn rename_field_in_documents(
        &mut self,
        old_field_name: &str,
        new_field_name: &str,
    ) -> Result<Vec<DocId>, String>;

    /// Lists all fields currently defined in the collection's schema.
    ///
    /// This method returns the names of all fields currently present in the schema,
    /// which can be useful for schema introspection and validation.
    ///
    /// ## Returns
    ///
    /// A [`Vec<String>`] containing the names of all fields in the schema.
    fn list_fields(&self) -> Vec<String>;

    /// Adds IDs to all documents in the collection that don't have them.
    ///
    /// This method ensures that all documents in the collection have a proper ID field
    /// according to the collection's ID type. Documents that already have IDs are left
    /// unchanged.
    ///
    /// ## Arguments
    ///
    /// * `old_field_name` - The name of the current/old ID field.
    /// * `new_field_name` - The name of the new ID field to use.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(Vec<DocId>)`] with the IDs of documents that were updated,
    /// or [`Err(String)`] with an error message if the operation failed.
    fn add_ids_to_all_documents(
        &mut self,
        old_field_name: &str,
        new_field_name: &str,
    ) -> Result<Vec<DocId>, String>;
}

impl CollectionSchemaOps for Collection {
    fn has_field(&self, field: &str) -> bool {
        self.schema.fields.contains_key(field)
    }

    fn validate_document(&self, doc: &bson::Document) -> Result<(), Vec<String>> {
        self.schema.validate_document(doc)
    }

    fn add_field(
        &mut self,
        field_name: String,
        mut field_definition: FieldDefinition,
    ) -> Result<(), String> {
        if self.schema.fields.contains_key(&field_name) {
            return Err(format!(
                "Field '{}' already exists in the schema",
                field_name
            ));
        }

        if matches!(
            field_definition.field_type,
            crate::db::schema::FieldType::IdString | crate::db::schema::FieldType::IdInt
        ) {
            return Err(format!(
                "Cannot add ID field '{}' because the schema already has an ID field '{}'",
                field_name, self.id_field
            ));
        }

        let is_nullable = matches!(
            field_definition.field_type,
            crate::db::schema::FieldType::Nullable(_)
        );
        let has_default = field_definition.default_value.is_some();

        if is_nullable && !has_default {
            field_definition.default_value = Some(bson::Bson::Null);
        }

        if !is_nullable && !has_default && !self.document_indices.is_empty() {
            return Err(format!(
                "Cannot add non-nullable field '{}' without a default value because the collection contains {} existing documents",
                field_name,
                self.document_indices.len()
            ));
        }

        self.schema
            .fields
            .insert(field_name.clone(), field_definition.clone());

        if field_definition.default_value.is_some() {
            if let Err(e) = self.apply_defaults_to_existing(&field_name, &field_definition) {
                self.schema.fields.remove(&field_name);
                return Err(e);
            }
        }

        Ok(())
    }

    fn remove_field(&mut self, field_name: &str) -> Result<(), String> {
        if !self.schema.fields.contains_key(field_name) {
            return Err(format!(
                "Field '{}' does not exist in the schema",
                field_name
            ));
        }
        let is_id_field = field_name == self.id_field;

        self.schema.fields.remove(field_name);

        if is_id_field {
            let new_id_definition = FieldDefinition::new(crate::db::schema::FieldType::IdInt);
            self.schema
                .fields
                .insert("id".to_string(), new_id_definition);

            self.id_field = "id".to_string();
            self.id_type = crate::db::schema::IdType::Int;
            self.inserts = 0;

            self.add_ids_to_all_documents(field_name, "id")?;
        } else {
            self.cleanup_removed_field(field_name)?;
        }

        Ok(())
    }

    fn modify_field(
        &mut self,
        field_name: &str,
        mut new_definition: FieldDefinition,
    ) -> Result<(), String> {
        if !self.schema.fields.contains_key(field_name) {
            return Err(format!(
                "Field '{}' does not exist in the schema",
                field_name
            ));
        }

        let original_definition = self.schema.fields.get(field_name).unwrap().clone();

        let is_nullable = matches!(
            new_definition.field_type,
            crate::db::schema::FieldType::Nullable(_)
        );
        let has_default = new_definition.default_value.is_some();

        if is_nullable && !has_default {
            new_definition.default_value = Some(bson::Bson::Null);
        }

        if !self.document_indices.is_empty() && !is_nullable && !has_default {
            return Err(format!(
                "Cannot modify field '{}' to non-nullable without a default value because the collection contains {} existing documents",
                field_name,
                self.document_indices.len()
            ));
        }

        let original_is_id = matches!(
            original_definition.field_type,
            crate::db::schema::FieldType::IdString | crate::db::schema::FieldType::IdInt
        );

        let new_is_id = matches!(
            new_definition.field_type,
            crate::db::schema::FieldType::IdString | crate::db::schema::FieldType::IdInt
        );

        if !original_is_id && new_is_id {
            return Err(format!(
                "Cannot modify field '{}' to ID type because the schema already has an ID field '{}'",
                field_name, self.id_field
            ));
        }

        self.schema
            .fields
            .insert(field_name.to_string(), new_definition.clone());

        if original_is_id && new_is_id {
            self.id_type = match new_definition.field_type {
                crate::db::schema::FieldType::IdString => crate::db::schema::IdType::String,
                crate::db::schema::FieldType::IdInt => crate::db::schema::IdType::Int,
                _ => unreachable!(),
            };

            self.add_ids_to_all_documents(field_name, field_name)?;
        } else if original_is_id && !new_is_id {
            let new_id_definition = FieldDefinition::new(crate::db::schema::FieldType::IdInt);
            self.schema
                .fields
                .insert("id".to_string(), new_id_definition);

            self.id_field = "id".to_string();
            self.id_type = crate::db::schema::IdType::Int;

            if !self.document_indices.is_empty() && new_definition.default_value.is_some() {
                self.add_ids_to_all_documents(field_name, "id")?;
                self.apply_defaults_to_existing(field_name, &new_definition)?;
            }
        } else if !self.document_indices.is_empty() {
            self.cleanup_removed_field(field_name)?;

            if new_definition.default_value.is_some() {
                self.apply_defaults_to_existing(field_name, &new_definition)?;
            }
        }

        Ok(())
    }

    fn rename_field(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        if !self.has_field(old_name) {
            return Err(format!("Field '{}' does not exist", old_name));
        }
        if self.has_field(&new_name) {
            return Err(format!("Field '{}' already exists", new_name));
        }

        let field_definition = self.schema.fields.remove(old_name).unwrap();
        self.schema
            .fields
            .insert(new_name.clone(), field_definition);
        if old_name == self.id_field {
            self.id_field = new_name.clone();
        }

        self.rename_field_in_documents(old_name, &new_name)?;

        Ok(())
    }

    fn apply_defaults_to_existing(
        &mut self,
        field_name: &str,
        field_definition: &FieldDefinition,
    ) -> Result<Vec<DocId>, String> {
        let default_value = field_definition
            .default_value
            .as_ref()
            .ok_or_else(|| format!("Field '{}' has no default value", field_name))?;

        let mut updated_document_ids = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();

        let mut update_doc = bson::Document::new();
        update_doc.insert(field_name, default_value.clone());

        for doc_id in document_ids {
            match self.update_document(doc_id.clone(), update_doc.clone()) {
                Ok(_) => {
                    updated_document_ids.push(doc_id);
                }
                Err(errors) => {
                    return Err(format!(
                        "Failed to apply default value to document {:?}: {}",
                        doc_id,
                        errors.join(", ")
                    ));
                }
            }
        }

        Ok(updated_document_ids)
    }

    fn cleanup_removed_field(&mut self, field_name: &str) -> Result<Vec<DocId>, String> {
        let mut updated_document_ids = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();

        for doc_id in document_ids {
            if let Some(document) = self.get_document(doc_id.clone()) {
                if document.data.contains_key(field_name) {
                    let mut cleaned_doc = document.data.clone();
                    cleaned_doc.remove(field_name);

                    match self.append_to_log(&Operation::Update, &cleaned_doc) {
                        Ok(new_offset) => {
                            self.document_indices.insert(doc_id.clone(), new_offset);
                            updated_document_ids.push(doc_id);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Failed to write cleaned document {:?} to log: {}",
                                doc_id, e
                            ));
                        }
                    }
                }
            }
        }

        Ok(updated_document_ids)
    }

    fn rename_field_in_documents(
        &mut self,
        old_field_name: &str,
        new_field_name: &str,
    ) -> Result<Vec<DocId>, String> {
        let mut updated_document_ids = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();

        for doc_id in document_ids {
            if let Some(document) = self.get_document(doc_id.clone()) {
                if let Some(field_value) = document.data.get(old_field_name) {
                    let mut updated_doc = document.data.clone();
                    updated_doc.remove(old_field_name);
                    updated_doc.insert(new_field_name, field_value.clone());

                    match self.append_to_log(&Operation::Update, &updated_doc) {
                        Ok(new_offset) => {
                            self.document_indices.insert(doc_id.clone(), new_offset);
                            updated_document_ids.push(doc_id);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Failed to write renamed document {:?} to log: {}",
                                doc_id, e
                            ));
                        }
                    }
                }
            }
        }

        Ok(updated_document_ids)
    }

    fn list_fields(&self) -> Vec<String> {
        self.schema.fields.keys().cloned().collect()
    }

    fn add_ids_to_all_documents(
        &mut self,
        old_field_name: &str,
        new_field_name: &str,
    ) -> Result<Vec<DocId>, String> {
        self.inserts = 0;

        let mut documents_to_readd = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();
        for doc_id in document_ids {
            if let Some(document) = self.remove_document(doc_id) {
                documents_to_readd.push(document.data);
            }
        }

        let mut updated_document_ids = Vec::new();
        for mut doc_data in documents_to_readd {
            doc_data.remove(old_field_name);
            let new_id = self.generate_id();
            doc_data.insert(new_field_name, new_id.to_bson());

            match self.add_document(doc_data) {
                Ok(doc_id) => {
                    updated_document_ids.push(doc_id);
                }
                Err(errors) => {
                    return Err(format!(
                        "Failed to re-add document with new ID: {}",
                        errors.join(", ")
                    ));
                }
            }
        }

        Ok(updated_document_ids)
    }
}
