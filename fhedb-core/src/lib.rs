//! # Fhedb Core
//!
//! This crate provides the core functionality for the Fhedb database.

/// The database module - contains the core database structures.
pub mod database;

/// The collection module - contains the collection structures and operations.
pub mod collection;

/// The document module - contains the document and ID types.
pub mod document;

/// The reference utilities module - contains reference validation utilities.
pub mod reference_utils;

/// The schema module - contains schema definitions and validation logic.
pub mod schema;

/// The query module - contains query execution utilities.
pub mod query;

/// Commonly used types re-exported for easy access.
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
