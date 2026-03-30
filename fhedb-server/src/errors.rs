use axum::{http::StatusCode, response::IntoResponse};
use fhedb_core::errors::Error as FheDbError;
use fhedb_query::error::ParserError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] FheDbError),

    #[error("Syntax Error")]
    Parser(Vec<ParserError>),

    #[error("Server Error: {0}")]
    Server(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            AppError::Core(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::Parser(errors) => (
                StatusCode::BAD_REQUEST,
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            ),
            AppError::Server(error) => (StatusCode::INTERNAL_SERVER_ERROR, error),
        };
        (status, body).into_response()
    }
}
