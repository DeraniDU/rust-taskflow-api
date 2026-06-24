use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskTimestamps {
    pub created_at: String,
    pub updated_at: String,
}

impl Task {
    pub fn new(
        id: u32,
        title: impl Into<String>,
        description: impl Into<String>,
        status: TaskStatus,
        priority: TaskPriority,
        due_date: Option<String>,
        timestamps: TaskTimestamps,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            status,
            priority,
            due_date,
            created_at: timestamps.created_at,
            updated_at: timestamps.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_new_task() {
        let task = Task::new(
            1,
            "Test task",
            "Testing task creation",
            TaskStatus::Pending,
            TaskPriority::Medium,
            None,
            TaskTimestamps {
                created_at: "2026-06-24 10:00:00".to_string(),
                updated_at: "2026-06-24 10:00:00".to_string(),
            },
        );

        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, "Testing task creation");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert_eq!(task.due_date, None);
    }
}
