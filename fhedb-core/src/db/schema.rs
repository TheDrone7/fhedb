use bson::Bson;
use bson::Document;
use std::collections::HashMap;

/// Represents the type of ID that can be used in a collection.
///
/// This enum is used to specify whether a collection uses string or integer IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdType {
    /// String-based identifiers (UUIDs or arbitrary strings).
    String,
    /// Integer-based identifiers (u64).
    Int,
}

/// Represents the type of a field in a document schema.
///
/// This enum is used to specify the allowed types for fields in a document.
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
    /// A document identifier that must be a string.
    IdString,
    /// A document identifier that must be a u64 integer.
    IdInt,
}

/// Describes the schema for a document.
///
/// The schema maps field names to their expected types.
#[derive(Debug, Clone)]
pub struct Schema {
    /// A map from field names to their corresponding types.
    pub fields: HashMap<String, FieldType>,
}

impl Schema {
    /// Validates a BSON document against this schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - A reference to the [BSON document](bson::Document) to validate.
    ///
    /// ## Returns
    ///
    /// Returns [Ok(())](Result::Ok) if the document matches the schema.
    ///
    /// Returns [`Err`]\([`Vec<String>`]) containing error messages for each field that does not conform to the schema.
    pub fn validate_document(&self, doc: &Document) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (field, field_type) in &self.fields {
            match doc.get(field) {
                Some(value) => {
                    if let Err(e) = validate_bson_type(value, field_type) {
                        errors.push(format!("Field '{}': {}", field, e));
                    }
                }
                None => {
                    if let FieldType::IdString | FieldType::IdInt = field_type {
                        continue;
                    }
                    errors.push(format!("Missing field: '{}'.", field));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Ensures the schema has exactly one Id field.
    ///
    /// If more than one Id field is found, returns an error.
    /// If none is found, adds a new field named "id" with type IdString.
    /// If exactly one is found, does nothing.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok((String, IdType))`](Result::Ok) containing the name of the Id field and its type.
    ///
    /// Returns [`Err`]\([`String`]) containing an error message if the schema contains more than one Id field.
    pub fn ensure_id(&mut self) -> Result<(String, IdType), String> {
        let id_fields: Vec<(String, IdType)> = self
            .fields
            .iter()
            .filter_map(|(field, field_type)| match field_type {
                FieldType::IdString => Some((field.clone(), IdType::String)),
                FieldType::IdInt => Some((field.clone(), IdType::Int)),
                _ => None,
            })
            .collect();
        match id_fields.len() {
            0 => {
                self.fields.insert("id".to_string(), FieldType::IdInt);
                Ok(("id".to_string(), IdType::Int))
            }
            1 => Ok(id_fields[0].clone()),
            _ => {
                Err("Schema must contain at most one field with type IdString or IdInt".to_string())
            }
        }
    }
}

/// Checks whether a BSON value matches the expected field type.
///
/// ## Arguments
///
/// * `value` - The [BSON value](bson::Bson) to check.
/// * `field_type` - The expected [type](FieldType) of the value.
///
/// ## Returns
///
/// Returns [Ok(())](Result::Ok) if the value matches the expected type. Returns [`Err`]\([`String`]) with a description of the mismatch otherwise.
fn validate_bson_type(value: &Bson, field_type: &FieldType) -> Result<(), String> {
    match field_type {
        FieldType::Int => match value {
            Bson::Int32(_) | Bson::Int64(_) => Ok(()),
            _ => Err("Expected int".to_string()),
        },
        FieldType::Float => match value {
            Bson::Double(_) => Ok(()),
            _ => Err("Expected float".to_string()),
        },
        FieldType::Boolean => match value {
            Bson::Boolean(_) => Ok(()),
            _ => Err("Expected boolean".to_string()),
        },
        FieldType::String => match value {
            Bson::String(_) => Ok(()),
            _ => Err("Expected string".to_string()),
        },
        FieldType::Array(inner_type) => match value {
            Bson::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    if let Err(e) = validate_bson_type(v, inner_type) {
                        return Err(format!("Array element {}: {}", i, e));
                    }
                }
                Ok(())
            }
            _ => Err("Expected array".to_string()),
        },
        FieldType::Reference(_) => match value {
            Bson::String(_) => Ok(()),
            _ => Err("Expected reference (string)".to_string()),
        },
        FieldType::IdString => match value {
            Bson::String(_) => Ok(()),
            _ => Err("Expected ID as string".to_string()),
        },
        FieldType::IdInt => match value {
            Bson::Int32(_) | Bson::Int64(_) => Ok(()),
            _ => Err("Expected ID as integer".to_string()),
        },
    }
}
