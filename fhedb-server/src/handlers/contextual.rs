//! # Contextual Query Handlers
//!
//! This module routes contextual queries (queries within a database context)
//! to the appropriate sub-handlers for collection and document operations.

use fhedb_query::prelude::ContextualQuery;

use crate::{handlers::collection::execute_collection_query, state::ServerState};

/// Executes a contextual query within a specific database.
///
/// ## Arguments
///
/// * `db_name` - The name of the database to operate on.
/// * `query` - The [`ContextualQuery`] to execute.
/// * `state` - The [`ServerState`] containing database references.
///
/// ## Returns
///
/// Returns [`Ok`]\([`serde_json::Value`]) on success, or [`Err`]\([`String`]) on failure.
pub(crate) fn execute_contextual_query(
    db_name: String,
    query: ContextualQuery,
    state: &ServerState,
) -> Result<serde_json::Value, String> {
    match query {
        ContextualQuery::Collection(collection_query) => {
            execute_collection_query(db_name, collection_query, state)
        }
        ContextualQuery::Document(_) => Err("Not implemented".to_string()),
    }
}
