use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub async fn connect_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    create_tasks_table(&pool).await?;
    seed_tasks_if_empty(&pool).await?;

    Ok(pool)
}

async fn create_tasks_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_tasks_if_empty(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    let seed_tasks = [
        (
            "Learn Rust database",
            "Connect Rust API with SQLite database",
            "pending",
        ),
        (
            "Save tasks permanently",
            "Store created tasks in database instead of memory",
            "pending",
        ),
        (
            "Practice database pipeline",
            "Make sure database changes pass GitHub Actions",
            "completed",
        ),
    ];

    for (title, description, status) in seed_tasks {
        sqlx::query("INSERT INTO tasks (title, description, status) VALUES (?, ?, ?)")
            .bind(title)
            .bind(description)
            .bind(status)
            .execute(pool)
            .await?;
    }

    Ok(())
}
