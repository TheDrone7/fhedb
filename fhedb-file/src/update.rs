use crate::error::{FheDbFileError as Error, Result};
use fhedb_core::prelude::*;
use std::io::prelude::*;

pub trait FileUpdate {
    fn update_file(&self, path: &str) -> Result<()>;
}

impl FileUpdate for DbMetadata {
    fn update_file(&self, path: &str) -> Result<()> {
        let path = std::path::Path::new(path);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .map_err(|_| Error::new("Could not open file", path.to_str().unwrap_or("")))?;

        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)
            .map_err(|_| Error::new("Could not read metadata size", path.to_str().unwrap_or("")))?;

        let size = u32::from_le_bytes(buffer) as u64;

        file.seek(std::io::SeekFrom::Start(size as u64))
            .map_err(|_| Error::new("Could not seek file", path.to_str().unwrap_or("")))?;

        let mut remaining = Vec::new();
        file.read_to_end(&mut remaining)
            .map_err(|_| Error::new("Could not read file", path.to_str().unwrap_or("")))?;

        let dbm = self.to_bytes();

        file.seek(std::io::SeekFrom::Start(0))
            .map_err(|_| Error::new("Could not seek file", path.to_str().unwrap_or("")))?;

        file.write_all(&dbm)
            .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))?;

        file.write_all(&remaining)
            .map_err(|_| Error::new("Could not write to file", path.to_str().unwrap_or("")))?;

        Ok(())
    }
}
