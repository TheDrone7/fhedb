//! # Schema
//!
//! Schema definitions and validation logic for FHEDB collections.

use bson::{Bson, Document};

pub use fhedb_types::{FieldDefinition, FieldType, IdType, Schema};

/// Extension trait for Schema with validation and serialization methods.
pub trait SchemaOps {
    /// Validates a BSON document against this schema.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`Document`] to validate.
    ///
    /// ## Returns
    ///
    /// [`Ok(())`](Result::Ok) if valid, or [`Err`]\([`Vec<String>`]) with validation errors.
    fn validate_document(&self, doc: &Document) -> Result<(), Vec<String>>;

    /// Ensures the schema has exactly one Id field.
    ///
    /// If no Id field exists, adds a default `id` field with type [`FieldType::IdInt`].
    ///
    /// ## Returns
    ///
    /// [`Ok`]\((field_name, [`IdType`])) on success, or [`Err`]\([`String`]) if multiple Id fields exist.
    fn ensure_id(&mut self) -> Result<(String, IdType), String>;

    /// Applies default values to a document for any missing fields that have defaults.
    ///
    /// ## Arguments
    ///
    /// * `doc` - The [`Document`] to apply defaults to.
    ///
    /// ## Returns
    ///
    /// The number of fields that had defaults applied.
    fn apply_defaults(&self, doc: &mut Document) -> usize;
}

impl SchemaOps for Schema {
    fn validate_document(&self, doc: &Document) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (field, field_def) in &self.fields {
            match doc.get(field) {
                Some(value) => {
                    if let Err(e) = validate_bson_type(value, &field_def.field_type) {
                        errors.push(format!("Field '{}': {}", field, e));
                    }
                }
                None => match &field_def.field_type {
                    FieldType::IdString | FieldType::IdInt | FieldType::Nullable(_) => continue,
                    _ => errors.push(format!("Missing field: '{}'.", field)),
                },
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn ensure_id(&mut self) -> Result<(String, IdType), String> {
        let id_fields: Vec<(String, IdType)> = self
            .fields
            .iter()
            .filter_map(|(field, field_def)| match &field_def.field_type {
                FieldType::IdString => Some((field.clone(), IdType::String)),
                FieldType::IdInt => Some((field.clone(), IdType::Int)),
                _ => None,
            })
            .collect();

        match id_fields.len() {
            0 => {
                self.fields.insert(
                    "id".to_string(),
                    FieldDefinition {
                        field_type: FieldType::IdInt,
                        default_value: None,
                    },
                );
                Ok(("id".to_string(), IdType::Int))
            }
            1 => Ok(id_fields[0].clone()),
            _ => {
                Err("Schema must contain at most one field with type IdString or IdInt".to_string())
            }
        }
    }

    fn apply_defaults(&self, doc: &mut Document) -> usize {
        let mut applied_count = 0;

        for (field_name, field_def) in &self.fields {
            if doc.contains_key(field_name) {
                continue;
            }

            match &field_def.field_type {
                FieldType::IdString | FieldType::IdInt | FieldType::Nullable(_) => continue,
                _ => {}
            }

            if let Some(default_value) = &field_def.default_value {
                doc.insert(field_name.clone(), default_value.clone());
                applied_count += 1;
            }
        }

        applied_count
    }
}

/// Converts a BSON document to a [`Schema`].
///
/// ## Arguments
///
/// * `doc` - The [`Document`] containing schema field definitions.
///
/// ## Returns
///
/// A new [`Schema`] with parsed field definitions.
pub fn schema_from_document(doc: Document) -> Schema {
    let mut schema = Schema::new();

    for (field_name, field_definition_value) in doc {
        if let Some(field_def) = parse_field_definition(&field_definition_value) {
            schema.fields.insert(field_name, field_def);
        }
    }

    schema
}

/// Converts a [`Schema`] to a BSON document.
///
/// ## Arguments
///
/// * `schema` - The [`Schema`] to convert.
///
/// ## Returns
///
/// A [`Document`] representing the schema.
pub fn schema_to_document(schema: Schema) -> Document {
    let mut doc = Document::new();

    for (field_name, field_def) in schema.fields {
        doc.insert(field_name, field_definition_to_bson(field_def));
    }

    doc
}

/// Parses a BSON value into a [`FieldDefinition`].
///
/// ## Arguments
///
/// * `value` - The [`Bson`] value to parse.
///
/// ## Returns
///
/// [`Some`]\([`FieldDefinition`]) if valid, or [`None`] if not recognized.
fn parse_field_definition(value: &Bson) -> Option<FieldDefinition> {
    match value {
        Bson::String(_) => parse_field_type(value).map(|field_type| FieldDefinition {
            field_type,
            default_value: None,
        }),
        Bson::Document(doc) => {
            if doc.contains_key("type") {
                let field_type = parse_field_type(doc.get("type")?)?;
                let default_value = doc.get("default").cloned();
                Some(FieldDefinition {
                    field_type,
                    default_value,
                })
            } else {
                parse_field_type(value).map(|field_type| FieldDefinition {
                    field_type,
                    default_value: None,
                })
            }
        }
        _ => None,
    }
}

/// Parses a BSON value into a [`FieldType`].
///
/// ## Arguments
///
/// * `value` - The [`Bson`] value to parse.
///
/// ## Returns
///
/// [`Some`]\([`FieldType`]) if valid, or [`None`] if not recognized.
fn parse_field_type(value: &Bson) -> Option<FieldType> {
    match value {
        Bson::String(s) => match s.as_str() {
            "int" => Some(FieldType::Int),
            "float" => Some(FieldType::Float),
            "boolean" => Some(FieldType::Boolean),
            "string" => Some(FieldType::String),
            "id_string" => Some(FieldType::IdString),
            "id_int" => Some(FieldType::IdInt),
            _ => None,
        },
        Bson::Document(doc) => {
            if let Some(bson) = doc.get("array") {
                parse_field_type(bson).map(|inner| FieldType::Array(Box::new(inner)))
            } else if let Some(Bson::String(collection_name)) = doc.get("reference") {
                Some(FieldType::Reference(collection_name.clone()))
            } else if let Some(bson) = doc.get("nullable") {
                parse_field_type(bson).map(|inner| FieldType::Nullable(Box::new(inner)))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Converts a [`FieldType`] to a [`Bson`] value.
///
/// ## Arguments
///
/// * `field_type` - The [`FieldType`] to convert.
///
/// ## Returns
///
/// A [`Bson`] value representing the field type.
fn field_type_to_bson(field_type: FieldType) -> Bson {
    match field_type {
        FieldType::Int => Bson::String("int".to_string()),
        FieldType::Float => Bson::String("float".to_string()),
        FieldType::Boolean => Bson::String("boolean".to_string()),
        FieldType::String => Bson::String("string".to_string()),
        FieldType::IdString => Bson::String("id_string".to_string()),
        FieldType::IdInt => Bson::String("id_int".to_string()),
        FieldType::Array(inner_type) => {
            let mut doc = Document::new();
            doc.insert("array", field_type_to_bson(*inner_type));
            Bson::Document(doc)
        }
        FieldType::Reference(collection_name) => {
            let mut doc = Document::new();
            doc.insert("reference", Bson::String(collection_name));
            Bson::Document(doc)
        }
        FieldType::Nullable(inner_type) => {
            let mut doc = Document::new();
            doc.insert("nullable", field_type_to_bson(*inner_type));
            Bson::Document(doc)
        }
    }
}

/// Converts a [`FieldDefinition`] to a [`Bson`] value.
///
/// ## Arguments
///
/// * `field_def` - The [`FieldDefinition`] to convert.
///
/// ## Returns
///
/// A [`Bson`] value representing the field definition.
fn field_definition_to_bson(field_def: FieldDefinition) -> Bson {
    match field_def.default_value {
        None => field_type_to_bson(field_def.field_type),
        Some(default) => {
            let mut doc = Document::new();
            doc.insert("type", field_type_to_bson(field_def.field_type));
            doc.insert("default", default);
            Bson::Document(doc)
        }
    }
}

/// Checks whether a BSON value matches the expected field type.
///
/// ## Arguments
///
/// * `value` - The [`Bson`] value to check.
/// * `field_type` - The expected [`FieldType`].
///
/// ## Returns
///
/// [`Ok(())`](Result::Ok) if the value matches, or [`Err`]\([`String`]) with an error message.
pub fn validate_bson_type(value: &Bson, field_type: &FieldType) -> Result<(), String> {
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
            Bson::String(_) | Bson::Null => Ok(()),
            _ => Err("Expected reference (string or null)".to_string()),
        },
        FieldType::Nullable(inner_type) => match value {
            Bson::Null => Ok(()),
            _ => validate_bson_type(value, inner_type),
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
