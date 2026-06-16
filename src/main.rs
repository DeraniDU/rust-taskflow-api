use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::net::SocketAddr;

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Rust TaskFlow API is running"
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/health", get(health_check));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
