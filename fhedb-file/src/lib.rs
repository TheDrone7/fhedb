//! This is the library that handles file I/O for the FHEDB project.
//!
//! It adds the ability to 
//! - create
//! - read
//! - update
//! - delete 
//! 
//! files for the various structures from [`fhedb_core`].


/// The error module contains the error types for the file I/O operations.
pub mod error;

/// Contains the file I/O operations for database metadata.
pub mod metadata;

/// The prelude module contains the common imports for the file library.
pub mod prelude;
