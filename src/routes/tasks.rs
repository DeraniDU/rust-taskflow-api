use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use sqlx::{Row, sqlite::SqliteRow};

use crate::{
    errors::ApiError,
    models::task::{CreateTaskRequest, Task, TaskStatus, UpdateTaskRequest},
    state::AppState,
};

pub fn task_routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
}

async fn get_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, ApiError> {
    let rows = sqlx::query("SELECT id, title, description, status FROM tasks ORDER BY id ASC")
        .fetch_all(&state.db)
        .await?;

    let tasks = rows.into_iter().map(row_to_task).collect();

    Ok(Json(tasks))
}

async fn get_task(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Task>, ApiError> {
    let row = sqlx::query("SELECT id, title, description, status FROM tasks WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(&state.db)
        .await?;

    match row {
        Some(row) => Ok(Json(row_to_task(row))),
        None => Err(ApiError::NotFound("Task not found".to_string())),
    }
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();

    if title.is_empty() {
        return Err(ApiError::BadRequest("Task title is required".to_string()));
    }

    let result = sqlx::query("INSERT INTO tasks (title, description, status) VALUES (?, ?, ?)")
        .bind(&title)
        .bind(&description)
        .bind("pending")
        .execute(&state.db)
        .await?;

    let new_task = Task::new(
        result.last_insert_rowid() as u32,
        title,
        description,
        TaskStatus::Pending,
    );

    Ok((StatusCode::CREATED, Json(new_task)))
}

async fn update_task(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();

    if title.is_empty() {
        return Err(ApiError::BadRequest("Task title is required".to_string()));
    }

    let status_text = task_status_to_database(&payload.status);

    let result =
        sqlx::query("UPDATE tasks SET title = ?, description = ?, status = ? WHERE id = ?")
            .bind(&title)
            .bind(&description)
            .bind(status_text)
            .bind(id as i64)
            .execute(&state.db)
            .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Task not found".to_string()));
    }

    let updated_task = Task::new(id, title, description, payload.status);

    Ok(Json(updated_task))
}

async fn delete_task(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Task not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn row_to_task(row: SqliteRow) -> Task {
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
}

pub fn task_status_from_database(status: &str) -> TaskStatus {
    match status {
        "completed" => TaskStatus::Completed,
        _ => TaskStatus::Pending,
    }
}

fn task_status_to_database(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Completed => "completed",
    }
}
