use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tower_http::cors::{Any, CorsLayer};

use crate::{routes::tasks::task_routes, state::AppState};

pub fn create_app(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .merge(task_routes())
        .with_state(app_state)
        .layer(cors)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Rust TaskFlow API is running"
    }))
}
