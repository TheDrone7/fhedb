use std::fmt;

pub type Result<T> = std::result::Result<T, FheDbFileError>;

#[derive(Debug, Clone)]
pub struct FheDbFileError {
    message: String,
    filename: String,
}

impl FheDbFileError {
    pub fn new(message: &str, filename: &str) -> Self {
        Self {
            message: message.to_owned(),
            filename: filename.to_owned(),
        }
    }
}

impl fmt::Display for FheDbFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File '{}' Error:\n{}", self.filename, self.message)
    }
}
