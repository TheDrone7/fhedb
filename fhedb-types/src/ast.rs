//! # AST Types
//!
//! Abstract Syntax Tree type definitions for FHEDB queries.

use std::collections::HashMap;

use crate::{
    query::{FieldCondition, FieldSelector},
    schema::{FieldDefinition, Schema},
};

/// Represents queries that operate at the database level.
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseQuery {
    /// Create a new database with the specified name.
    Create {
        /// The name of the database to create.
        name: String,
        /// Whether to drop an existing database with the same name before creating.
        drop_if_exists: bool,
    },
    /// Drops an existing database with the specified name.
    Drop {
        /// The name of the database to drop.
        name: String,
    },
    /// Lists all databases.
    List,
}

/// Represents queries that operate within a specific database context.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextualQuery {
    /// A collection operation query.
    Collection(CollectionQuery),
    /// A document operation query.
    Document(DocumentQuery),
}

/// Represents a modification operation on a collection field.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldModification {
    /// Drop the field from the collection schema.
    Drop,
    /// Set the field to a new definition (add or modify).
    Set(FieldDefinition),
}

/// Represents queries on collections within a database.
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionQuery {
    /// Create a new collection with the specified name and schema.
    Create {
        /// The name of the collection to create.
        name: String,
        /// Whether to drop an existing collection with the same name before creating.
        drop_if_exists: bool,
        /// The schema definition for the collection.
        schema: Schema,
    },
    /// Drops an existing collection with the specified name.
    Drop {
        /// The name of the collection to drop.
        name: String,
    },
    /// Modifies an existing collection's schema.
    Modify {
        /// The name of the collection to modify.
        name: String,
        /// A map of field names to their modification operations.
        modifications: HashMap<String, FieldModification>,
    },
    /// Lists all collections in the database.
    List,
    /// Gets the schema of a specific collection.
    GetSchema {
        /// The name of the collection to get the schema for.
        name: String,
    },
}

/// Represents queries on documents within a database's collections.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentQuery {
    /// Insert a new document into a collection.
    Insert {
        /// The name of the collection to insert into.
        collection_name: String,
        /// The document data as field-value pairs.
        fields: HashMap<String, String>,
    },
    /// Update an existing document in a collection.
    Update {
        /// The name of the collection to update in.
        collection_name: String,
        /// The conditions to identify which document(s) to update.
        conditions: Vec<FieldCondition>,
        /// The field updates to apply.
        updates: HashMap<String, String>,
        /// The additional fields to return in the response.
        selectors: Vec<FieldSelector>,
    },
    /// Delete document(s) from a collection.
    Delete {
        /// The name of the collection to delete from.
        collection_name: String,
        /// The conditions to identify which document(s) to delete.
        conditions: Vec<FieldCondition>,
        /// The additional fields to return in the response.
        selectors: Vec<FieldSelector>,
    },
    /// Get/query document(s) from a collection.
    Get {
        /// The name of the collection to query.
        collection_name: String,
        /// The conditions to filter documents (empty means get all).
        conditions: Vec<FieldCondition>,
        /// The fields to return in the response.
        selectors: Vec<FieldSelector>,
    },
}
