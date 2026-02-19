//! # Collection Data
//!
//! Provides schema modification and data consistency operations for collections.

use crate::{
    collection::{Collection, Operation},
    document::DocId,
    schema::{FieldDefinition, FieldType, IdType, SchemaOps},
};

/// Schema modification and data consistency operations.
impl Collection {
    /// Checks if the schema contains a field with the given name.
    ///
    /// ## Arguments
    ///
    /// * `field` - The name of the field to check.
    ///
    /// ## Returns
    ///
    /// `true` if the field exists in the schema, `false` otherwise.
    pub fn has_field(&self, field: &str) -> bool {
        self.schema.fields.contains_key(field)
    }

    /// Validates a BSON document against this collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`bson::Document`] to validate.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the document matches the schema,
    /// or [`Err`]\([`Vec<String>`]) with validation errors.
    pub fn validate_document(&self, doc: &bson::Document) -> Result<(), Vec<String>> {
        self.schema.validate_document(doc)
    }

    /// Adds a new field to the collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to add.
    /// * `field_definition` - The [`FieldDefinition`] specifying the type and default value.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the field was successfully added,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn add_field(
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
            FieldType::IdString | FieldType::IdInt
        ) {
            return Err(format!(
                "Cannot add ID field '{}' because the schema already has an ID field '{}'",
                field_name, self.id_field
            ));
        }

        let is_nullable = matches!(field_definition.field_type, FieldType::Nullable(_));
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

        if field_definition.default_value.is_some()
            && let Err(e) = self.apply_defaults_to_existing(&field_name, &field_definition)
        {
            self.schema.fields.remove(&field_name);
            return Err(e);
        }

        Ok(())
    }

    /// Removes a field from the collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to remove.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the field was successfully removed,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn remove_field(&mut self, field_name: &str) -> Result<(), String> {
        if !self.schema.fields.contains_key(field_name) {
            return Err(format!(
                "Field '{}' does not exist in the schema",
                field_name
            ));
        }
        let is_id_field = field_name == self.id_field;

        self.schema.fields.remove(field_name);

        if is_id_field {
            let new_id_definition = FieldDefinition::new(FieldType::IdInt);
            self.schema
                .fields
                .insert("id".to_string(), new_id_definition);

            self.id_field = "id".to_string();
            self.id_type = IdType::Int;
            self.inserts = 0;

            self.add_ids_to_all_documents(field_name, "id")?;
        } else {
            self.cleanup_removed_field(field_name)?;
        }

        Ok(())
    }

    /// Modifies an existing field's definition in the collection's schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the field to modify.
    /// * `new_definition` - The new [`FieldDefinition`] for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the field was successfully modified,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn modify_field(
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

        let is_nullable = matches!(new_definition.field_type, FieldType::Nullable(_));
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
            FieldType::IdString | FieldType::IdInt
        );

        let new_is_id = matches!(
            new_definition.field_type,
            FieldType::IdString | FieldType::IdInt
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
                FieldType::IdString => IdType::String,
                FieldType::IdInt => IdType::Int,
                _ => unreachable!(),
            };

            self.add_ids_to_all_documents(field_name, field_name)?;
        } else if original_is_id && !new_is_id {
            let new_id_definition = FieldDefinition::new(FieldType::IdInt);
            self.schema
                .fields
                .insert("id".to_string(), new_id_definition);

            self.id_field = "id".to_string();
            self.id_type = IdType::Int;

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

    /// Renames a field in the collection's schema.
    /// Preserves the field's definition and data.
    ///
    /// ## Arguments
    ///
    /// * `old_name` - The current name of the field.
    /// * `new_name` - The new name for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if the field was successfully renamed,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn rename_field(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
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

    /// Applies default values to existing documents when a new field with a default is added.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the newly added field.
    /// * `field_definition` - The [`FieldDefinition`] containing the default value.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<DocId>`]) with the IDs of updated documents,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn apply_defaults_to_existing(
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

    /// Removes field data from existing documents when a field is removed from the schema.
    ///
    /// ## Arguments
    ///
    /// * `field_name` - The name of the removed field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<DocId>`]) with the IDs of updated documents,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn cleanup_removed_field(&mut self, field_name: &str) -> Result<Vec<DocId>, String> {
        let mut updated_document_ids = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();

        for doc_id in document_ids {
            if let Some(document) = self.get_document(doc_id.clone())
                && document.data.contains_key(field_name)
            {
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

        Ok(updated_document_ids)
    }

    /// Renames a field in all existing documents when a field is renamed in the schema.
    ///
    /// ## Arguments
    ///
    /// * `old_field_name` - The current name of the field.
    /// * `new_field_name` - The new name for the field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<DocId>`]) with the IDs of updated documents,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn rename_field_in_documents(
        &mut self,
        old_field_name: &str,
        new_field_name: &str,
    ) -> Result<Vec<DocId>, String> {
        let mut updated_document_ids = Vec::new();
        let document_ids: Vec<DocId> = self.document_indices.keys().cloned().collect();

        for doc_id in document_ids {
            if let Some(document) = self.get_document(doc_id.clone())
                && let Some(field_value) = document.data.get(old_field_name)
            {
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

        Ok(updated_document_ids)
    }

    /// Returns the names of all fields in the schema.
    pub fn list_fields(&self) -> Vec<String> {
        self.schema.fields.keys().cloned().collect()
    }

    /// Re-assigns IDs to all documents in the collection.
    ///
    /// ## Arguments
    ///
    /// * `old_field_name` - The name of the current ID field to remove.
    /// * `new_field_name` - The name of the new ID field.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<DocId>`]) with the new document IDs,
    /// or [`Err`]\([`String`]) with an error message.
    pub fn add_ids_to_all_documents(
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
