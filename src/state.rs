use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub api_key: String,
}

impl AppState {
    pub fn new(db: SqlitePool, api_key: impl Into<String>) -> Self {
        Self {
            db,
            api_key: api_key.into(),
        }
    }
}
