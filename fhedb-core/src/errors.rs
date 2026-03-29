use std::{io, result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Validation failed: {}", .0.join(", "))]
    Validation(Vec<String>),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Document already exists: {0}")]
    DocumentAlreadyExists(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionAlreadyExists(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

pub type Result<T> = result::Result<T, Error>;
