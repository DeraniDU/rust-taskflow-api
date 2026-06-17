use std::sync::{Arc, Mutex};

use crate::models::task::{Task, TaskStatus};

pub type SharedTasks = Arc<Mutex<Vec<Task>>>;

#[derive(Clone)]
pub struct AppState {
    pub tasks: SharedTasks,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(seed_tasks())),
        }
    }
}

fn seed_tasks() -> Vec<Task> {
    vec![
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
    ]
}
