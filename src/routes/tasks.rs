use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::json;
use sqlx::Row;

use crate::{
    models::task::{CreateTaskRequest, Task, TaskStatus},
    state::AppState,
};

type ApiError = (StatusCode, Json<serde_json::Value>);

pub fn task_routes() -> Router<AppState> {
    Router::new().route("/tasks", get(get_tasks).post(create_task))
}

async fn get_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, ApiError> {
    let rows = sqlx::query("SELECT id, title, description, status FROM tasks ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let tasks = rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let title: String = row.get("title");
            let description: String = row.get("description");
            let status: String = row.get("status");

            Task::new(
                id as u32,
                title,
                description,
                task_status_from_database(&status),
            )
        })
        .collect();

    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();

    if title.is_empty() {
        return Err(bad_request("Task title is required"));
    }

    let result = sqlx::query("INSERT INTO tasks (title, description, status) VALUES (?, ?, ?)")
        .bind(&title)
        .bind(&description)
        .bind("pending")
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    let new_task = Task::new(
        result.last_insert_rowid() as u32,
        title,
        description,
        TaskStatus::Pending,
    );

    Ok((StatusCode::CREATED, Json(new_task)))
}

fn task_status_from_database(status: &str) -> TaskStatus {
    match status {
        "completed" => TaskStatus::Completed,
        _ => TaskStatus::Pending,
    }
}

fn bad_request(message: &str) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": message
        })),
    )
}

fn internal_error(_error: sqlx::Error) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "Internal server error"
        })),
    )
}
