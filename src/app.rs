use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::{routes::tasks::task_routes, state::AppState};

pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .merge(task_routes())
        .with_state(app_state)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Rust TaskFlow API is running"
    }))
}
