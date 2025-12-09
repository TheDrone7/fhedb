use axum::{
    RequestExt,
    body::Body,
    extract::{FromRequest, Path, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fhedb_query::{
    ast::{CollectionQuery, DocumentQuery},
    error::ParserError,
    prelude::{ContextualQuery, DatabaseQuery, parse_contextual_query, parse_database_query},
};
use log::debug;

pub enum ParsedQuery {
    Base(DatabaseQuery),
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
            .map(|path_param| {
                if path_param.is_none() {
                    return "".to_string();
                } else {
                    let Path(some_path) = path_param.unwrap();
                    return some_path;
                }
            })
            .map_err(IntoResponse::into_response)?;

        let query = req
            .extract::<String, _>()
            .await
            .map_err(IntoResponse::into_response)?;

        if path.trim().is_empty() {
            match parse_database_query(&query) {
                Ok(ast) => {
                    let query = ParsedQuery::Base(ast);
                    log_query(&query, "");
                    return Ok(query);
                }
                Err(err) => return Err(handle_errs(&query, err)),
            };
        } else {
            match parse_contextual_query(&query) {
                Ok(ast) => {
                    let query = ParsedQuery::Context(ast);
                    log_query(&query, &path);
                    return Ok(query);
                }
                Err(err) => return Err(handle_errs(&query, err)),
            };
        }
    }
}

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

fn log_query(query: &ParsedQuery, path: &str) {
    let query_type = match query {
        ParsedQuery::Base(ast) => match ast {
            &DatabaseQuery::Create { .. } => "Create database",
            &DatabaseQuery::Drop { .. } => "Drop database",
            &DatabaseQuery::List => "List database",
        },
        ParsedQuery::Context(ast) => match ast {
            ContextualQuery::Collection(coll) => match coll {
                &CollectionQuery::Create { .. } => "Create collection",
                &CollectionQuery::Drop { .. } => "Drop collection",
                &CollectionQuery::List => "List collections",
                &CollectionQuery::GetSchema { .. } => "Get collection schema",
                &CollectionQuery::Modify { .. } => "Modify collection",
            },
            ContextualQuery::Document(doc) => match doc {
                &DocumentQuery::Insert { .. } => "Insert document",
                &DocumentQuery::Delete { .. } => "Delete document",
                &DocumentQuery::Get { .. } => "Get/List documents",
                &DocumentQuery::Update { .. } => "Update document",
            },
        },
    };

    debug!("Parsed {} query at path '{}'", query_type, path);
}
