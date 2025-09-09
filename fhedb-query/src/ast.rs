//! Abstract Syntax Tree (AST) definitions for FHEDB queries.
//!
//! This module defines the core data structures that represent the parsed
//! form of FHEDB query language statements.

/// Represents a query in the FHEDB query language.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// A database-level operation query such as creating or dropping a database.
    DatabaseQuery(DatabaseQuery),
    /// A contextual query that operates within a specific database context.
    ContextualQuery(ContextualQuery),
}

/// Represents queries that operate at the database level.
///
/// Database queries are operations that work at the database level,
/// such as creating new databases or dropping existing ones.
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
}

/// Represents queries that operate within a specific database context.
///
/// Contextual queries are operations that work on collections, documents,
/// or other entities within an existing database.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextualQuery {
    /// A collection operation query.
    Collection(CollectionQuery),
    /// A document operation query.
    Document(DocumentQuery),
}

/// Represents queries on collections within a database,
/// such as creating collections, dropping collections, or modifying collection schemas.
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionQuery {
    // TODO: Add collection operations
}

/// Represents queries on documents within a database's collections,
/// such as inserting, updating, deleting, or querying documents.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentQuery {
    // TODO: Add document operations
}
