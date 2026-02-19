//! # Fhedb Core
//!
//! This crate provides the core functionality for the Fhedb database.

/// The database module - contains the core database structures.
pub mod database;

/// The file module - contains the file operations for the database.
pub mod collection;

pub mod document;
pub mod reference_utils;
pub mod schema;

/// The query module - contains query execution utilities.
pub mod query;

// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::collection::{
        Collection,
        file::{LogEntry, Operation},
    };
    pub use crate::database::Database;
    pub use crate::document::{DocId, Document};
    pub use crate::query::{
        BsonComparable, ConditionEvaluable, DocumentPreparable, FieldSelectable, Unescapable,
        ValueParseable,
    };
    pub use crate::reference_utils::{ReferenceChecker, SchemaReferenceValidator};
    pub use crate::schema::{
        FieldDefinition, FieldType, IdType, Schema, SchemaOps, schema_from_document,
        schema_to_document, validate_bson_type,
    };
}
