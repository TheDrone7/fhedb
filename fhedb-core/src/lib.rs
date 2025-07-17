pub mod db;

/// Re-exports commonly used types for easy access.
pub mod prelude {
    pub use crate::db::schema::{FieldType, Schema};
}
