pub mod config;
pub mod handlers;
pub mod logger;

pub mod prelude {
    pub use crate::config::core::CoreConfig;
    pub use crate::handlers::*;
    pub use crate::logger::setup_logger;
}
