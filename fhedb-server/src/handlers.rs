use axum::extract::Path;

pub async fn handle_base() -> () {
    ()
}

pub async fn handle_db(Path(db_name): Path<String>) -> String {
    format!("Database: {}", db_name)
}
