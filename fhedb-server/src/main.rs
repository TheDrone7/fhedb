use axum::{
    Router,
    routing::{get, post},
};
use log::info;

use fhedb_server::prelude::{CoreConfig, ServerState, handle_base, handle_db, setup_logger};

#[tokio::main]
async fn main() {
    let core_config = CoreConfig::read_from_file();
    core_config.ensure_dirs();

    setup_logger(
        core_config.logging.get_level(),
        core_config.logging.get_file(),
    )
    .expect("Unable to set up logging utility.");

    let app = Router::new()
        .route("/", get(|| async { "Hello, FHEDB!" }))
        .route("/", post(handle_base))
        .route("/{db_name}", post(handle_db))
        .with_state(ServerState::new(core_config.storage.get_base_dir().clone()));

    let address = format!(
        "{}:{}",
        core_config.server.get_host(),
        core_config.server.get_port()
    );

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    info!("Server running on http://{}", &address);
    axum::serve(listener, app).await.unwrap();
}
