use axum::{
    Router,
    handler::Handler,
    middleware::{self},
    routing::{get, post},
};
use log::info;

use fhedb_server::prelude::{
    CoreConfig, ServerState, check_database, handle_base, handle_db, setup_logger,
};

#[tokio::main]
async fn main() {
    let core_config = CoreConfig::read_from_file();
    core_config.ensure_dirs();

    setup_logger(core_config.logging.level(), core_config.logging.file())
        .expect("Unable to set up logging utility.");

    let state = ServerState::new(core_config.storage.base_dir().clone());
    let layered_db_handler = handle_db.layer(middleware::from_fn_with_state(
        state.clone(),
        check_database,
    ));
    let app = Router::new()
        .route("/", get(|| async { "Hello, FHEDB!" }))
        .route("/", post(handle_base))
        .route("/{db_name}", post(layered_db_handler))
        .with_state(state);

    let address = format!(
        "{}:{}",
        core_config.server.host(),
        core_config.server.port()
    );

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    info!("Server running on http://{}", &address);
    axum::serve(listener, app).await.unwrap();
}
