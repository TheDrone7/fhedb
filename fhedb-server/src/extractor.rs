//! # Query Extractors
//!
//! Axum extractors for parsing incoming query requests into structured types.

use axum::{
    RequestExt,
    body::Body,
    extract::{FromRequest, Path, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fhedb_query::prelude::{ParserError, parse_contextual_query, parse_database_query};
use fhedb_types::{CollectionQuery, ContextualQuery, DatabaseQuery, DocumentQuery};
use log::debug;

/// Represents a parsed query from the request body.
pub enum ParsedQuery {
    /// A database-level query (CREATE/DROP/LIST DATABASE).
    Base(DatabaseQuery),
    /// A contextual query within a database (collection/document operations).
    Context(ContextualQuery),
}

impl<S> FromRequest<S> for ParsedQuery
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(mut req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let path = req
            .extract_parts::<Option<Path<String>>>()
            .await
            .map(|path_param| path_param.map_or(String::new(), |Path(p)| p))
            .map_err(IntoResponse::into_response)?;

        let query = req
            .extract::<String, _>()
            .await
            .map_err(IntoResponse::into_response)?;

        if path.trim().is_empty() {
            match parse_database_query(&query) {
                Ok(ast) => {
                    let parsed = ParsedQuery::Base(ast);
                    log_query(&parsed, "");
                    Ok(parsed)
                }
                Err(err) => Err(handle_errs(&query, err)),
            }
        } else {
            match parse_contextual_query(&query) {
                Ok(ast) => {
                    let parsed = ParsedQuery::Context(ast);
                    log_query(&parsed, &path);
                    Ok(parsed)
                }
                Err(err) => Err(handle_errs(&query, err)),
            }
        }
    }
}

/// Formats parser errors into an HTTP response.
///
/// ## Arguments
///
/// * `query` - The original query string that failed to parse.
/// * `errs` - The list of [`ParserError`]s to format.
fn handle_errs(query: &str, errs: Vec<ParserError>) -> Response {
    let errors = errs
        .iter()
        .map(|e| e.format(query))
        .collect::<Vec<_>>()
        .join("\n\n");
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(errors))
        .unwrap()
}

/// Logs the parsed query type for debugging purposes.
///
/// ## Arguments
///
/// * `query` - The [`ParsedQuery`] to log.
/// * `path` - The request path for context.
fn log_query(query: &ParsedQuery, path: &str) {
    let query_type = match query {
        ParsedQuery::Base(ast) => match ast {
            DatabaseQuery::Create { .. } => "Create database",
            DatabaseQuery::Drop { .. } => "Drop database",
            DatabaseQuery::List => "List database",
        },
        ParsedQuery::Context(ast) => match ast {
            ContextualQuery::Collection(coll) => match coll {
                CollectionQuery::Create { .. } => "Create collection",
                CollectionQuery::Drop { .. } => "Drop collection",
                CollectionQuery::List => "List collections",
                CollectionQuery::GetSchema { .. } => "Get collection schema",
                CollectionQuery::Modify { .. } => "Modify collection",
            },
            ContextualQuery::Document(doc) => match doc {
                DocumentQuery::Insert { .. } => "Insert document",
                DocumentQuery::Delete { .. } => "Delete document",
                DocumentQuery::Get { .. } => "Get/List documents",
                DocumentQuery::Update { .. } => "Update document",
            },
        },
    };

    debug!("Parsed {} query at path '{}'", query_type, path);
}
