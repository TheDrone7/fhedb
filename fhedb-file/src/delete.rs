use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::*;

pub trait FileDelete {
    fn delete_file(&self, path: &str) -> Result<()>;
}

impl FileDelete for DbMetadata {
    fn delete_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|_| Error::new("Could not delete file", path.to_str().unwrap_or("")))
        } else {
            Err(Error::new(
                "File does not exist",
                path.to_str().unwrap_or(""),
            ))
        }
    }
}
