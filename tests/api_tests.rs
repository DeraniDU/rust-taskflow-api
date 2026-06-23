use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use rust_taskflow_api::{app::create_app, database::sqlite::connect_database, state::AppState};
use serde_json::{Value, json};
use tower::util::ServiceExt;

fn test_database_url() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time error")
        .as_nanos();

    let path = std::env::temp_dir().join(format!("taskflow_test_{timestamp}.db"));

    format!("sqlite://{}", path.to_string_lossy())
}

async fn test_app() -> axum::Router {
    let database_url = test_database_url();
    let db = connect_database(&database_url)
        .await
        .expect("Failed to connect test database");

    let app_state = AppState::new(db);

    create_app(app_state)
}

#[tokio::test]
async fn get_tasks_returns_success() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_tasks_creates_task() {
    let app = test_app().await;

    let request_body = json!({
        "title": "Integration test task",
        "description": "Created from API integration test"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::CREATED);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let body: Value = serde_json::from_slice(&bytes).expect("Failed to parse response body");

    assert_eq!(body["title"], "Integration test task");
    assert_eq!(body["description"], "Created from API integration test");
    assert_eq!(body["status"], "pending");
}

#[tokio::test]
async fn post_tasks_rejects_empty_title() {
    let app = test_app().await;

    let request_body = json!({
        "title": "",
        "description": "This should fail"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
