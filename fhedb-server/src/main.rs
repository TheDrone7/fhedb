use axum::{
    Router,
    routing::{get, post},
};
use log::info;

use fhedb_server::{
    logger::setup_logger,
    prelude::{CoreConfig, handle_base, handle_db},
};

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
        .route("/{db_name}", post(handle_db));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
