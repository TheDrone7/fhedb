use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use log::error;

use crate::state::ServerState;

pub async fn check_database(
    Path(db_name): Path<String>,
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    let db_exists = match state.databases.try_read() {
        Ok(dbs) => dbs.contains_key(&db_name),
        Err(err) => {
            error!("Unable to check databases: {:#?}", err);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Unable to check databases: {:?}", err)))
                .unwrap();
        }
    };
    if !db_exists {
        let mut db_dir = state.data_dir.clone();
        db_dir.push(&db_name);
        match db_dir.try_exists() {
            Ok(existence) => {
                if !existence {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from(format!(
                            "Database '{}' does not exist.",
                            &db_name
                        )))
                        .unwrap();
                }
            }
            Err(err) => {
                error!("Unable to read database directory: {:#?}", err);
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!(
                        "Unable to read database directory: {:?}",
                        err
                    )))
                    .unwrap();
            }
        }
    }
    let response = next.run(request).await;
    response
}
