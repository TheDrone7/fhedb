pub mod config;
pub mod handlers;
pub mod logger;
pub mod middleware;
pub mod state;

pub mod prelude {
    pub use crate::config::core::CoreConfig;
    pub use crate::handlers::*;
    pub use crate::logger::setup_logger;
    pub use crate::middleware::check_database;
    pub use crate::state::ServerState;
}
