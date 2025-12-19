//! # Query Module
//!
//! Provides query execution utilities for document operations.

mod comparison;
mod value;

pub use comparison::{compare_bson, evaluate_condition};
pub use value::{parse_bson_value, unescape};
