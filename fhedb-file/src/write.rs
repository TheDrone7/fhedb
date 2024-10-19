use std::fs::File;
use std::io::prelude::*;

use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::{Database, bson};

pub trait FileWrite {
    fn to_file(&self, path: &str) -> Result<()>;
}

impl FileWrite for Database {
    fn to_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        let mut file = File::create(path)
            .map_err(|_| Error::new("Could not create file", path.to_str().unwrap_or("")))?;
        if let Ok(db) = bson::to_vec(self) {
            file.write_all(&db)
                .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))
        } else {
            Err(Error::new("Could not serialize database", ""))
        }
    }
}
