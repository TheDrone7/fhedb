//! This is the library that handles file I/O for the FHEDB project.
//!
//! It adds the ability to create, read, update and delete files for the database and metadata.

/// The create module contains the functions for creating files.
pub mod create;

/// The delete module contains the functions for deleting files.
pub mod delete;

/// The error module contains the error types for the file I/O operations.
pub mod error;

/// The read module contains the functions for reading files.
pub mod read;

/// The update module contains the functions for updating files.
pub mod update;

/// The prelude module contains the common imports for the file library.
pub mod prelude;
