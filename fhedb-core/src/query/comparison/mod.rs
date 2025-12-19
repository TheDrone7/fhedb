//! # Comparison Module
//!
//! Provides BSON comparison and condition evaluation utilities.

mod compare;
mod condition;

pub use compare::BsonComparable;
pub use condition::ConditionEvaluable;
