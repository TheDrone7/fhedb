//! # Comparison Module
//!
//! Provides BSON comparison and condition evaluation utilities.

mod compare;
mod condition;

pub use compare::compare_bson;
pub use condition::evaluate_condition;
