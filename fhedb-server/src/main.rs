use axum::{
    Router,
    routing::{get, post},
};

mod handlers;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, FHEDB!" }))
        .route("/", post(handlers::handle_base))
        .route("/{db_name}", post(handlers::handle_db));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
