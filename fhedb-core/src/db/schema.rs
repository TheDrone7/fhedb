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
    /// A nullable value that can be null or of the specified type.
    Nullable(Box<FieldType>),
    /// A document identifier that must be a string.
    IdString,
    /// A document identifier that must be a u64 integer.
    IdInt,
}

/// Represents a field definition in a document schema.
///
/// This struct contains both the type of the field and its default value.
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
    /// * `field_type` - The type of the field.
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
    /// * `field_type` - The type of the field.
    /// * `default_value` - The default value for the field.
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
    /// * `field_type` - The type of the field.
    /// * `default_value` - The optional default value for the field.
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
///
/// The schema maps field names to their field definitions (type and default value).
#[derive(Debug, Clone)]
pub struct Schema {
    /// A map from field names to their corresponding field definitions.
    pub fields: HashMap<String, FieldDefinition>,
}

impl Schema {
    /// Creates a new empty schema.
    ///
    /// ## Returns
    ///
    /// A new [`Schema`] with no fields defined.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

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
        for (field, field_def) in &self.fields {
            match doc.get(field) {
                Some(value) => {
                    if let Err(e) = validate_bson_type(value, &field_def.field_type) {
                        errors.push(format!("Field '{}': {}", field, e));
                    }
                }
                None => match &field_def.field_type {
                    FieldType::IdString | FieldType::IdInt => {
                        continue;
                    }
                    FieldType::Nullable(_) => {
                        continue;
                    }
                    _ => {
                        errors.push(format!("Missing field: '{}'.", field));
                    }
                },
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
    /// If none is found, adds a new field named "id" with type IdInt.
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

    /// Applies default values to a document for any missing fields that have defaults.
    ///
    /// ## Arguments
    ///
    /// * `doc` - A mutable reference to the BSON document to update.
    ///
    /// ## Returns
    ///
    /// The number of fields that had default values applied.
    pub fn apply_defaults(&self, doc: &mut Document) -> usize {
        let mut applied_count = 0;

        for (field_name, field_def) in &self.fields {
            if doc.contains_key(field_name) {
                continue;
            }

            match &field_def.field_type {
                FieldType::IdString | FieldType::IdInt => continue,
                FieldType::Nullable(_) => continue,
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

impl From<Document> for Schema {
    /// Creates a [`Schema`] from a BSON document.
    ///
    /// The BSON document should contain field names as keys and field definition
    /// representations as values. Each field definition can be either:
    /// - A simple type string (for required fields without defaults)
    /// - A document with "type" and "default" keys
    ///
    /// ## Arguments
    ///
    /// * `doc` - The BSON document to convert from.
    ///
    /// ## Returns
    ///
    /// A new [`Schema`] with the parsed fields.
    fn from(doc: Document) -> Self {
        let mut schema = Schema::new();

        for (field_name, field_definition_value) in doc {
            if let Some(field_def) = parse_field_definition(&field_definition_value) {
                schema.fields.insert(field_name, field_def);
            }
        }

        schema
    }
}

impl From<Schema> for Document {
    /// Converts a [`Schema`] to a BSON document.
    ///
    /// ## Arguments
    ///
    /// * `schema` - The schema to convert.
    ///
    /// ## Returns
    ///
    /// A BSON document representing the schema.
    fn from(schema: Schema) -> Self {
        let mut doc = Document::new();

        for (field_name, field_def) in schema.fields {
            doc.insert(field_name, field_definition_to_bson(field_def));
        }

        doc
    }
}

/// Parses a BSON value into a [`FieldDefinition`].
///
/// ## Arguments
///
/// * `value` - The BSON value to parse.
///
/// ## Returns
///
/// Returns [`Some`]\([`FieldDefinition`]) if the value represents a valid field definition,
/// or [`None`] if the value is not recognized.
fn parse_field_definition(value: &Bson) -> Option<FieldDefinition> {
    match value {
        Bson::String(_) => {
            if let Some(field_type) = parse_field_type(value) {
                Some(FieldDefinition {
                    field_type,
                    default_value: None,
                })
            } else {
                None
            }
        }
        Bson::Document(doc) => {
            if doc.contains_key("type") {
                let field_type = parse_field_type(doc.get("type")?)?;
                let default_value = doc.get("default").cloned();
                Some(FieldDefinition {
                    field_type,
                    default_value,
                })
            } else {
                if let Some(field_type) = parse_field_type(value) {
                    Some(FieldDefinition {
                        field_type,
                        default_value: None,
                    })
                } else {
                    None
                }
            }
        }
        _ => None,
    }
}

/// Parses a BSON value into a [`FieldType`].
///
/// ## Arguments
///
/// * `value` - The BSON value to parse.
///
/// ## Returns
///
/// Returns [`Some`]\([`FieldType`]) if the value represents a valid field type,
/// or [`None`] if the value is not recognized.
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
            if doc.contains_key("array") {
                if let Some(bson) = doc.get("array") {
                    let inner_field_type = match parse_field_type(bson) {
                        Some(field_type) => field_type,
                        None => return None,
                    };
                    Some(FieldType::Array(Box::new(inner_field_type)))
                } else {
                    None
                }
            } else if doc.contains_key("reference") {
                if let Some(Bson::String(collection_name)) = doc.get("reference") {
                    Some(FieldType::Reference(collection_name.clone()))
                } else {
                    None
                }
            } else if doc.contains_key("nullable") {
                if let Some(bson) = doc.get("nullable") {
                    let inner_field_type = match parse_field_type(bson) {
                        Some(field_type) => field_type,
                        None => return None,
                    };
                    Some(FieldType::Nullable(Box::new(inner_field_type)))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Converts a [`FieldType`] to a BSON value.
///
/// ## Arguments
///
/// * `field_type` - The field type to convert.
///
/// ## Returns
///
/// A BSON value representing the field type.
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

/// Converts a [`FieldDefinition`] to a BSON value.
///
/// ## Arguments
///
/// * `field_def` - The field definition to convert.
///
/// ## Returns
///
/// A BSON value representing the field definition.
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
