//! # Request Middleware
//!
//! This module provides middleware functions for request processing,
//! such as validating database existence before handling requests.

use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use fhedb_core::prelude::Database;
use log::{debug, error};

use crate::{error as api_error, internal_error, state::ServerState};

/// Middleware that checks if a database exists before processing the request.
///
/// Validates that the requested database exists either in memory or on disk
/// before allowing the request to proceed to the handler. If the database
/// exists on disk but not in memory, it will be loaded into the server state.
///
/// ## Arguments
///
/// * `db_name` - The name of the database from the request path.
/// * `state` - The [`ServerState`] containing database references.
/// * `request` - The incoming HTTP [`Request`](axum::extract::Request).
/// * `next` - The [`Next`](axum::middleware::Next) middleware/handler in the chain.
///
/// ## Returns
///
/// Returns the [`Response`] from the next handler, or an error response if the database doesn't exist.
pub async fn check_database(
    Path(db_name): Path<String>,
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    let db_exists_in_memory = match state.databases.try_read() {
        Ok(dbs) => dbs.contains_key(&db_name),
        Err(err) => {
            error!("Unable to check databases: {:#?}", err);
            return internal_error!(format!("Unable to check databases: {:?}", err))
                .into_response();
        }
    };

    if !db_exists_in_memory {
        let mut db_dir = state.data_dir.clone();
        db_dir.push(&db_name);
        match db_dir.try_exists() {
            Ok(existence) => {
                if !existence {
                    return api_error!(
                        format!("Database '{}' does not exist.", &db_name),
                        StatusCode::NOT_FOUND
                    )
                    .into_response();
                }

                debug!("Loading database '{}' from disk into memory.", &db_name);
                match Database::from_files(&db_name, &state.data_dir) {
                    Ok(db) => {
                        let mut dbs = match state.databases.write() {
                            Ok(dbs) => dbs,
                            Err(err) => {
                                error!("Unable to acquire write lock on databases: {:#?}", err);
                                return internal_error!(format!(
                                    "Unable to acquire write lock on databases: {:?}",
                                    err
                                ))
                                .into_response();
                            }
                        };
                        dbs.insert(db_name.clone(), db);
                    }
                    Err(err) => {
                        error!(
                            "Unable to load database '{}' from disk: {:#?}",
                            &db_name, err
                        );
                        return internal_error!(format!(
                            "Unable to load database '{}' from disk: {:?}",
                            &db_name, err
                        ))
                        .into_response();
                    }
                }
            }
            Err(err) => {
                error!("Unable to read database directory: {:#?}", err);
                return internal_error!(format!("Unable to read database directory: {:?}", err))
                    .into_response();
            }
        }
    }

    next.run(request).await
}
