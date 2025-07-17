use crate::db::schema::Schema;

/// Describes a collection of documents in the database.
///
/// Each collection has a unique name and an associated [`Schema`].
#[derive(Debug, Clone)]
pub struct Collection {
    /// The name of the collection.
    pub name: String,
    /// The schema describing the structure of documents in this collection.
    pub schema: Schema,
}

impl Collection {
    /// Creates a new [`Collection`] with the given name and schema.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the collection.
    /// * `schema` - The [`Schema`] describing the structure of documents in this collection.
    ///
    /// ## Returns
    ///
    /// A new [`Collection`] instance.
    pub fn new(name: impl Into<String>, schema: Schema) -> Self {
        Self {
            name: name.into(),
            schema,
        }
    }

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
    /// * `doc` - A reference to the [`bson::Document`] to validate.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok(())`](Result::Ok) if the document matches the schema. Returns [`Err(Vec<String>)`](Result::Err) containing error messages for each field that does not conform to the schema.
    pub fn validate_document(&self, doc: &bson::Document) -> Result<(), Vec<String>> {
        self.schema.validate_document(doc)
    }
}
