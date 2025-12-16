//! # Query Types
//!
//! Type definitions for query operations and conditions.

use std::collections::HashMap;

/// Represents comparison operators for document field conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryOperator {
    /// Equality operator (=) - exact match.
    Equal,
    /// Inequality operator (!=) - not equal.
    NotEqual,
    /// Greater than operator (>) - numeric/string comparison.
    GreaterThan,
    /// Greater than or equal operator (>=) - numeric/string comparison.
    GreaterThanOrEqual,
    /// Less than or equal operator (<=) - numeric/string comparison.
    LessThanOrEqual,
    /// Less than operator (<) - numeric/string comparison.
    LessThan,
    /// Similarity operator (==) - pattern/substring matching.
    Similar,
}

/// Represents a condition on a document field for filtering/querying.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldCondition {
    /// The name of the field to apply the condition to.
    pub field_name: String,
    /// The comparison operator to use.
    pub operator: QueryOperator,
    /// The value to compare the field against.
    pub value: String,
}

/// Represents the parsed content of a document query.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDocContent {
    /// Field assignments (field: value).
    pub assignments: HashMap<String, String>,
    /// Field conditions (field operator value).
    pub conditions: Vec<FieldCondition>,
    /// Field selectors (which fields to return).
    pub selectors: Vec<FieldSelector>,
}

/// Represents which fields to return in a query response.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldSelector {
    /// Return specific named fields only.
    Field(String),
    /// Return all fields (*) - shallow, no reference resolution.
    AllFields,
    /// Return all fields (**) - deep, with recursive reference resolution.
    AllFieldsRecursive,
    /// Nested sub-document field (with reference).
    SubDocument {
        /// The name of the sub-document field.
        field_name: String,
        /// The parsed content for the sub-document.
        content: ParsedDocContent,
    },
}
