mod base;
pub(crate) mod collection;
mod contextual;

use crate::{handlers::base::execute_base_query, state::ServerState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use fhedb_query::prelude::parse_database_query;

pub async fn handle_base(State(state): State<ServerState>, body: String) -> (StatusCode, String) {
    let ast_result = parse_database_query(&body);
    if let Err(errors) = ast_result {
        let err_strings: Vec<String> = errors.iter().map(|e| e.format(&body)).collect();
        let err_response = format!("{}", err_strings.join("\n"));
        return (StatusCode::BAD_REQUEST, err_response);
    }
    let ast = ast_result.unwrap();
    match execute_base_query(ast, &state) {
        Ok(message) => (StatusCode::OK, message),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub async fn handle_db(Path(db_name): Path<String>) -> String {
    format!("Database: {}", db_name)
}
