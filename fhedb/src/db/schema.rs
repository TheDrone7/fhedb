use std::collections::HashMap;

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
