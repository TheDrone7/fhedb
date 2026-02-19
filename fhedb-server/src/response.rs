//! # Response Types
//!
//! Unified response types for the fhedb server. Success responses return JSON, errors return plaintext.

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// An API response, either success (JSON) or error (plaintext).
#[derive(Debug)]
pub enum ApiResponse {
    /// A successful response with a JSON data payload.
    Success(serde_json::Value),
    /// An error response with a plaintext message and HTTP status code.
    Error {
        /// The error message describing what went wrong.
        message: String,
        /// The HTTP status code for the error.
        status: StatusCode,
    },
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success(data) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(data.to_string()))
                .unwrap(),
            ApiResponse::Error { message, status } => Response::builder()
                .status(status)
                .header("Content-Type", "text/plain")
                .body(Body::from(message))
                .unwrap(),
        }
    }
}

/// Creates a successful [`ApiResponse`] with the given data.
///
/// ## Arguments
///
/// * `data` - Any serializable value to include in the response.
#[macro_export]
macro_rules! success {
    ($data:expr) => {{
        let value = serde_json::to_value($data).unwrap_or(serde_json::Value::Null);
        $crate::response::ApiResponse::Success(value)
    }};
}

/// Creates an error [`ApiResponse`] with the given message and status code.
///
/// ## Arguments
///
/// * `message` - The error message to include in the response.
/// * `status` - The HTTP [`StatusCode`] for the error.
#[macro_export]
macro_rules! error {
    ($message:expr, $status:expr) => {
        $crate::response::ApiResponse::Error {
            message: $message.to_string(),
            status: $status,
        }
    };
}

/// Creates an internal server error API response with the given message.
///
/// ## Arguments
///
/// * `message` - The error message to include in the response.
#[macro_export]
macro_rules! internal_error {
    ($message:expr) => {
        $crate::response::ApiResponse::Error {
            message: $message.to_string(),
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    };
}
