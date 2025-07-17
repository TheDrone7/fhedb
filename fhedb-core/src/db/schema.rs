use bson::Bson;
use bson::Document;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int,
    Float,
    Boolean,
    String,
    Array(Box<FieldType>),
    Reference(String), // Name of the collection it refers to
    Id,                // Auto-generated UUID
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub fields: HashMap<String, FieldType>,
}

impl Schema {
    /// Validates a BSON document against the schema.
    /// Returns Ok(()) if valid, or Err(Vec<String>) with error messages.
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
}

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
            Bson::String(_) => Ok(()), // Could add more checks if needed
            _ => Err("Expected reference (string)".to_string()),
        },
        FieldType::Id => match value {
            Bson::String(s) => Uuid::parse_str(s)
                .map(|_| ())
                .map_err(|_| "Expected valid UUID string".to_string()),
            _ => Err("Expected UUID as string".to_string()),
        },
    }
}
