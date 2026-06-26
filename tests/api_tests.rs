use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use rust_taskflow_api::{app::create_app, database::sqlite::connect_database, state::AppState};
use serde_json::{Value, json};
use tower::util::ServiceExt;

const TEST_API_KEY: &str = "test-api-key";

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

    let app_state = AppState::new(db, TEST_API_KEY);

    create_app(app_state)
}

async fn response_body_to_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    serde_json::from_slice(&bytes).expect("Failed to parse response body")
}

async fn create_test_task(app: axum::Router) -> Value {
    let request_body = json!({
        "title": "Test task",
        "description": "Created during integration test",
        "priority": "high",
        "due_date": "2026-07-01"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::CREATED);

    response_body_to_json(response).await
}

#[tokio::test]
async fn health_check_does_not_require_api_key() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_tasks_rejects_missing_api_key() {
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_tasks_returns_success() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_task_by_id_returns_success() {
    let app = test_app().await;

    let created_task = create_test_task(app.clone()).await;
    let task_id = created_task["id"]
        .as_u64()
        .expect("Task id should be a number");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/tasks/{task_id}"))
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_body_to_json(response).await;

    assert_eq!(body["id"], task_id);
    assert_eq!(body["title"], "Test task");
    assert_eq!(body["priority"], "high");
    assert_eq!(body["due_date"], "2026-07-01");
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn get_task_by_id_returns_not_found_for_invalid_id() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks/999999")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn post_tasks_creates_task_with_metadata() {
    let app = test_app().await;

    let request_body = json!({
        "title": "Integration test task",
        "description": "Created from API integration test",
        "priority": "high",
        "due_date": "2026-07-10"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_body_to_json(response).await;

    assert_eq!(body["title"], "Integration test task");
    assert_eq!(body["description"], "Created from API integration test");
    assert_eq!(body["status"], "pending");
    assert_eq!(body["priority"], "high");
    assert_eq!(body["due_date"], "2026-07-10");
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn post_tasks_uses_medium_priority_when_missing() {
    let app = test_app().await;

    let request_body = json!({
        "title": "Default priority task",
        "description": "Priority is not provided"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_body_to_json(response).await;

    assert_eq!(body["priority"], "medium");
    assert_eq!(body["due_date"], Value::Null);
}

#[tokio::test]
async fn post_tasks_rejects_empty_title() {
    let app = test_app().await;

    let request_body = json!({
        "title": "",
        "description": "This should fail",
        "priority": "medium",
        "due_date": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(request_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn put_tasks_updates_task_with_metadata() {
    let app = test_app().await;

    let created_task = create_test_task(app.clone()).await;
    let task_id = created_task["id"]
        .as_u64()
        .expect("Task id should be a number");

    let update_body = json!({
        "title": "Updated task title",
        "description": "Updated task description",
        "status": "completed",
        "priority": "low",
        "due_date": "2026-08-01"
    });

    let update_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/tasks/{task_id}"))
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(update_body.to_string()))
                .expect("Failed to build update request"),
        )
        .await
        .expect("Failed to call update task");

    assert_eq!(update_response.status(), StatusCode::OK);

    let updated_task = response_body_to_json(update_response).await;

    assert_eq!(updated_task["title"], "Updated task title");
    assert_eq!(updated_task["description"], "Updated task description");
    assert_eq!(updated_task["status"], "completed");
    assert_eq!(updated_task["priority"], "low");
    assert_eq!(updated_task["due_date"], "2026-08-01");
}

#[tokio::test]
async fn put_tasks_returns_not_found_for_invalid_id() {
    let app = test_app().await;

    let update_body = json!({
        "title": "Invalid update",
        "description": "This task does not exist",
        "status": "completed",
        "priority": "high",
        "due_date": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/tasks/999999")
                .header("content-type", "application/json")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::from(update_body.to_string()))
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_tasks_deletes_task() {
    let app = test_app().await;

    let created_task = create_test_task(app.clone()).await;
    let task_id = created_task["id"]
        .as_u64()
        .expect("Task id should be a number");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{task_id}"))
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build delete request"),
        )
        .await
        .expect("Failed to call delete task");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/tasks/{task_id}"))
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build get request"),
        )
        .await
        .expect("Failed to call get task");

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_tasks_returns_not_found_for_invalid_id() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/tasks/999999")
                .header("x-api-key", TEST_API_KEY)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to call app");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
