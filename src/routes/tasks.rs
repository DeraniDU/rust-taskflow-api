use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use sqlx::{QueryBuilder, Row, Sqlite, sqlite::SqliteRow};

use crate::{
    auth::validate_api_key,
    errors::ApiError,
    models::task::{
        CreateTaskRequest, Task, TaskPriority, TaskQueryParams, TaskStatus, TaskTimestamps,
        UpdateTaskRequest,
    },
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

async fn get_tasks(
    headers: HeaderMap,
    Query(params): Query<TaskQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Task>>, ApiError> {
    validate_api_key(&headers, &state.api_key)?;

    let mut query_builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT id, title, description, status, priority, due_date, created_at, updated_at
        FROM tasks
        "#,
    );

    let mut has_where = false;

    if let Some(status) = params.status.as_ref() {
        push_where_or_and(&mut query_builder, &mut has_where);
        query_builder
            .push("status = ")
            .push_bind(task_status_to_database(status));
    }

    if let Some(priority) = params.priority.as_ref() {
        push_where_or_and(&mut query_builder, &mut has_where);
        query_builder
            .push("priority = ")
            .push_bind(task_priority_to_database(priority));
    }

    if let Some(due_date) = params.due_date.as_ref() {
        push_where_or_and(&mut query_builder, &mut has_where);
        query_builder.push("due_date = ").push_bind(due_date);
    }

    query_builder.push(" ORDER BY id ASC");

    let rows = query_builder.build().fetch_all(&state.db).await?;

    let tasks = rows.into_iter().map(row_to_task).collect();

    Ok(Json(tasks))
}

async fn get_task(
    Path(id): Path<u32>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Task>, ApiError> {
    validate_api_key(&headers, &state.api_key)?;

    let row = sqlx::query(
        r#"
        SELECT id, title, description, status, priority, due_date, created_at, updated_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(id as i64)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok(Json(row_to_task(row))),
        None => Err(ApiError::NotFound("Task not found".to_string())),
    }
}

async fn create_task(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    validate_api_key(&headers, &state.api_key)?;

    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();
    let priority = payload.priority.unwrap_or(TaskPriority::Medium);
    let due_date = payload.due_date;

    if title.is_empty() {
        return Err(ApiError::BadRequest("Task title is required".to_string()));
    }

    let result = sqlx::query(
        r#"
        INSERT INTO tasks
            (title, description, status, priority, due_date, created_at, updated_at)
        VALUES
            (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#,
    )
    .bind(&title)
    .bind(&description)
    .bind("pending")
    .bind(task_priority_to_database(&priority))
    .bind(&due_date)
    .execute(&state.db)
    .await?;

    let new_id = result.last_insert_rowid() as u32;

    let row = sqlx::query(
        r#"
        SELECT id, title, description, status, priority, due_date, created_at, updated_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(new_id as i64)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(row_to_task(row))))
}

async fn update_task(
    Path(id): Path<u32>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    validate_api_key(&headers, &state.api_key)?;

    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();

    if title.is_empty() {
        return Err(ApiError::BadRequest("Task title is required".to_string()));
    }

    let result = sqlx::query(
        r#"
        UPDATE tasks
        SET
            title = ?,
            description = ?,
            status = ?,
            priority = ?,
            due_date = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(&title)
    .bind(&description)
    .bind(task_status_to_database(&payload.status))
    .bind(task_priority_to_database(&payload.priority))
    .bind(&payload.due_date)
    .bind(id as i64)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Task not found".to_string()));
    }

    let row = sqlx::query(
        r#"
        SELECT id, title, description, status, priority, due_date, created_at, updated_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(id as i64)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row_to_task(row)))
}

async fn delete_task(
    Path(id): Path<u32>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    validate_api_key(&headers, &state.api_key)?;

    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Task not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn push_where_or_and(query_builder: &mut QueryBuilder<Sqlite>, has_where: &mut bool) {
    if *has_where {
        query_builder.push(" AND ");
    } else {
        query_builder.push(" WHERE ");
        *has_where = true;
    }
}

fn row_to_task(row: SqliteRow) -> Task {
    let id: i64 = row.get("id");
    let title: String = row.get("title");
    let description: String = row.get("description");
    let status: String = row.get("status");
    let priority: String = row.get("priority");
    let due_date: Option<String> = row.get("due_date");
    let created_at: Option<String> = row.get("created_at");
    let updated_at: Option<String> = row.get("updated_at");

    Task::new(
        id as u32,
        title,
        description,
        task_status_from_database(&status),
        task_priority_from_database(&priority),
        due_date,
        TaskTimestamps {
            created_at: created_at.unwrap_or_default(),
            updated_at: updated_at.unwrap_or_default(),
        },
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

pub fn task_priority_from_database(priority: &str) -> TaskPriority {
    match priority {
        "low" => TaskPriority::Low,
        "high" => TaskPriority::High,
        _ => TaskPriority::Medium,
    }
}

fn task_priority_to_database(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "low",
        TaskPriority::Medium => "medium",
        TaskPriority::High => "high",
    }
}
