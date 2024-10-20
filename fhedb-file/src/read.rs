use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::DbMetadata;

pub trait FileRead {
    fn from_file(path: &str) -> Result<DbMetadata>;
}

impl FileRead for DbMetadata {
    fn from_file(path: &str) -> Result<Self> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(Error::new(
                "File does not exist",
                path.to_str().unwrap_or(""),
            ));
        }

        if let Ok(db) = std::fs::read(path) {
            if db.len() < 1 {
                return Err(Error::new("File is empty", path.to_str().unwrap_or("")));
            }

            let db = Self::from(&db[0..db[0] as usize]);
            if let Ok(db) = db {
                Ok(db)
            } else {
                Err(Error::new(
                    "Could not parse file",
                    path.to_str().unwrap_or(""),
                ))
            }
        } else {
            Err(Error::new(
                "Could not read file",
                path.to_str().unwrap_or(""),
            ))
        }
    }
}
