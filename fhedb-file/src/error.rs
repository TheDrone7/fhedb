//! Error handling for fhedb-file crate.
//!
//! This module defines the error type and result type for the crate.

use std::fmt;

/// Result type for fhedb-file crate.
pub type Result<T> = std::result::Result<T, FheDbFileError>;

/// Error type for fhedb-file crate.
#[derive(Debug, Clone)]
pub struct FheDbFileError {
    /// Error message.
    message: String,
    /// File name that had the error
    filename: String,
}

/// Implementing Error trait for FheDbFileError.
impl FheDbFileError {
    /// Create a new FheDbFileError instance.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message.
    /// * `filename` - File name that had the error.
    ///
    /// # Returns
    ///
    /// A new FheDbFileError instance.
    ///
    /// # Example
    ///
    /// ```
    /// use fhedb_file::error::FheDbFileError;
    ///
    /// let error = FheDbFileError::new("Error message", "file.txt");
    /// ```
    pub fn new(message: &str, filename: &str) -> Self {
        Self {
            message: message.to_owned(),
            filename: filename.to_owned(),
        }
    }
}

/// Implementing Display trait for FheDbFileError.
impl fmt::Display for FheDbFileError {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File '{}' Error:\n{}", self.filename, self.message)
    }
}
