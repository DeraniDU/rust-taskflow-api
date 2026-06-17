use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::json;

use crate::{
    models::task::{CreateTaskRequest, Task, TaskStatus},
    state::AppState,
};

type ApiError = (StatusCode, Json<serde_json::Value>);

pub fn task_routes() -> Router<AppState> {
    Router::new().route("/tasks", get(get_tasks).post(create_task))
}

async fn get_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let tasks = state.tasks.lock().expect("Failed to lock tasks");

    Json(tasks.clone())
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Task title is required"
            })),
        ));
    }

    let mut tasks = state.tasks.lock().expect("Failed to lock tasks");

    let new_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;

    let new_task = Task::new(
        new_id,
        payload.title,
        payload.description,
        TaskStatus::Pending,
    );

    tasks.push(new_task.clone());

    Ok((StatusCode::CREATED, Json(new_task)))
}
