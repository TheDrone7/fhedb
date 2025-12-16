//! # Request Handlers
//!
//! This module provides the HTTP request handlers for database and contextual operations.
//! It routes incoming requests to the appropriate sub-handlers based on [`ParsedQuery`] type.

mod base;
pub(crate) mod collection;
mod contextual;
mod document;

use crate::{
    extractor::ParsedQuery, handlers::base::execute_base_query, internal_error, state::ServerState,
    success,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

/// Handles requests to the base endpoint (database-level operations).
///
/// Processes [`DatabaseQuery`](fhedb_types::DatabaseQuery) operations such as CREATE, DROP, and LIST.
pub async fn handle_base(State(state): State<ServerState>, body: ParsedQuery) -> impl IntoResponse {
    let query = match body {
        ParsedQuery::Base(db) => db,
        _ => return internal_error!("Expected database query, this should never happen."),
    };
    match execute_base_query(query, &state) {
        Ok(data) => success!(data),
        Err(err) => internal_error!(err),
    }
}

/// Handles requests to database-specific endpoints (contextual operations).
///
/// Processes [`ContextualQuery`](fhedb_types::ContextualQuery) operations within a database context.
pub async fn handle_db(
    Path(db_name): Path<String>,
    State(state): State<ServerState>,
    body: ParsedQuery,
) -> impl IntoResponse {
    let query = match body {
        ParsedQuery::Context(contextual) => contextual,
        _ => return internal_error!("Expected contextual query, this should never happen."),
    };
    match contextual::execute_contextual_query(db_name, query, &state) {
        Ok(data) => success!(data),
        Err(err) => internal_error!(err),
    }
}
