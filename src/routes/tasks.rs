use axum::{Json, Router, routing::get};

use crate::models::task::{Task, TaskStatus};

pub fn task_routes() -> Router {
    Router::new().route("/tasks", get(get_tasks))
}

async fn get_tasks() -> Json<Vec<Task>> {
    let tasks = vec![
        Task::new(
            1,
            "Learn Rust branches",
            "Create a feature branch and understand Git workflow",
            TaskStatus::Pending,
        ),
        Task::new(
            2,
            "Create Pull Request",
            "Push feature branch and open a PR on GitHub",
            TaskStatus::Pending,
        ),
        Task::new(
            3,
            "Fix CI pipeline",
            "Make sure cargo fmt, clippy, test, and build pass",
            TaskStatus::Completed,
        ),
    ];

    Json(tasks)
}
