use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::net::SocketAddr;

mod database;
mod models;
mod routes;
mod state;

use database::sqlite::connect_database;
use routes::tasks::task_routes;
use state::AppState;

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Rust TaskFlow API is running"
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = "sqlite://taskflow.db";
    let db = connect_database(database_url)
        .await
        .expect("Failed to connect database");

    let app_state = AppState::new(db);

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(task_routes())
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
