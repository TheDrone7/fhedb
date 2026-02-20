//! # Index
//!
//! Provides B+ tree index structures and operations.

/// The pager module - contains the pager implementation for managing index pages.
pub mod pager;

/// The node module - contains the B+ tree node structures and operations.
pub mod node;

/// The tree module - contains the B+ tree structure and operations for managing the index.
pub mod tree;
