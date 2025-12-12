mod base;
pub(crate) mod collection;
mod contextual;

use crate::{extractor::ParsedQuery, handlers::base::execute_base_query, state::ServerState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn handle_base(
    State(state): State<ServerState>,
    body: ParsedQuery,
) -> (StatusCode, String) {
    let query = match body {
        ParsedQuery::Base(db) => db,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Expected database query, this should never happen.".to_string(),
            );
        }
    };
    match execute_base_query(query, &state) {
        Ok(message) => (StatusCode::OK, message),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub async fn handle_db(
    Path(db_name): Path<String>,
    State(state): State<ServerState>,
    body: ParsedQuery,
) -> impl IntoResponse {
    let query = match body {
        ParsedQuery::Context(contextual) => contextual,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Expected contextual query, this should never happen.".to_string(),
            );
        }
    };
    match contextual::execute_contextual_query(db_name, query, &state) {
        Ok(message) => (StatusCode::OK, message),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}
