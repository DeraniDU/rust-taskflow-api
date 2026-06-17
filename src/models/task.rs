use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
}

impl Task {
    pub fn new(
        id: u32,
        title: impl Into<String>,
        description: impl Into<String>,
        status: TaskStatus,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            status,
        }
    }
}
