//! # Schema Types
//!
//! Type definitions for document schemas in FHEDB.

use std::collections::HashMap;

use bson::Bson;

/// Represents the type of ID that can be used in a collection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdType {
    /// String-based identifiers (UUIDs or arbitrary strings).
    String,
    /// Integer-based identifiers (u64).
    Int,
}

/// Represents the type of a field in a document schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// A 64-bit integer value.
    Int,
    /// A 64-bit floating point value.
    Float,
    /// A boolean value (true or false).
    Boolean,
    /// A UTF-8 encoded string value.
    String,
    /// An array of elements, all of the specified type.
    Array(Box<FieldType>),
    /// A reference to another collection, identified by its name.
    Reference(String),
    /// A nullable value that can be null or of the specified type.
    Nullable(Box<FieldType>),
    /// A document identifier that must be a string.
    IdString,
    /// A document identifier that must be a u64 integer.
    IdInt,
}

/// Represents a field definition in a document schema.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// The type of the field.
    pub field_type: FieldType,
    /// The default value for the field. If None, the field is required.
    pub default_value: Option<Bson>,
}

impl FieldDefinition {
    /// Creates a new required field definition (no default value).
    ///
    /// ## Arguments
    ///
    /// * `field_type` - The [`FieldType`] for this field.
    ///
    /// ## Returns
    ///
    /// A new [`FieldDefinition`] with no default value.
    pub fn new(field_type: FieldType) -> Self {
        Self {
            field_type,
            default_value: None,
        }
    }

    /// Creates a new field definition with a default value.
    ///
    /// ## Arguments
    ///
    /// * `field_type` - The [`FieldType`] for this field.
    /// * `default_value` - The default [`Bson`] value.
    ///
    /// ## Returns
    ///
    /// A new [`FieldDefinition`] with the specified default value.
    pub fn with_default(field_type: FieldType, default_value: Bson) -> Self {
        Self {
            field_type,
            default_value: Some(default_value),
        }
    }

    /// Creates a new field definition with an optional default value.
    ///
    /// ## Arguments
    ///
    /// * `field_type` - The [`FieldType`] for this field.
    /// * `default_value` - The optional default [`Bson`] value.
    ///
    /// ## Returns
    ///
    /// A new [`FieldDefinition`] with the specified optional default value.
    pub fn with_optional_default(field_type: FieldType, default_value: Option<Bson>) -> Self {
        Self {
            field_type,
            default_value,
        }
    }
}

/// Describes the schema for a document.
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    /// A map from field names to their corresponding field definitions.
    pub fields: HashMap<String, FieldDefinition>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl Schema {
    /// Creates a new empty schema.
    ///
    /// ## Returns
    ///
    /// A new [`Schema`] with no fields defined.
    pub fn new() -> Self {
        Self::default()
    }
}
