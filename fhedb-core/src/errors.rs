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

    #[error("Unable to create index on field '{0}': {1}")]
    FieldNotIndexable(String, String),

    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        if let Error::Io(self_io) = self
            && let Error::Io(other_io) = other
        {
            return self_io.kind() == other_io.kind()
                && self_io.to_string() == other_io.to_string();
        }
        self == other
    }
}

pub type Result<T> = result::Result<T, Error>;
