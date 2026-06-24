use std::str::FromStr;

use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub async fn connect_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    create_tasks_table(&pool).await?;
    migrate_tasks_table(&pool).await?;
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
            status TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'medium',
            due_date TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn migrate_tasks_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    if !column_exists(pool, "priority").await? {
        sqlx::query("ALTER TABLE tasks ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium'")
            .execute(pool)
            .await?;
    }

    if !column_exists(pool, "due_date").await? {
        sqlx::query("ALTER TABLE tasks ADD COLUMN due_date TEXT")
            .execute(pool)
            .await?;
    }

    if !column_exists(pool, "created_at").await? {
        sqlx::query("ALTER TABLE tasks ADD COLUMN created_at TEXT")
            .execute(pool)
            .await?;
    }

    if !column_exists(pool, "updated_at").await? {
        sqlx::query("ALTER TABLE tasks ADD COLUMN updated_at TEXT")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        UPDATE tasks
        SET
            created_at = COALESCE(created_at, datetime('now')),
            updated_at = COALESCE(updated_at, datetime('now')),
            priority = COALESCE(priority, 'medium')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn column_exists(pool: &SqlitePool, column_name: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(tasks)")
        .fetch_all(pool)
        .await?;

    let exists = rows.into_iter().any(|row| {
        let name: String = row.get("name");
        name == column_name
    });

    Ok(exists)
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
            "medium",
            None::<String>,
        ),
        (
            "Save tasks permanently",
            "Store created tasks in database instead of memory",
            "pending",
            "high",
            None::<String>,
        ),
        (
            "Practice database pipeline",
            "Make sure database changes pass GitHub Actions",
            "completed",
            "low",
            None::<String>,
        ),
    ];

    for (title, description, status, priority, due_date) in seed_tasks {
        sqlx::query(
            r#"
            INSERT INTO tasks
                (title, description, status, priority, due_date, created_at, updated_at)
            VALUES
                (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
        )
        .bind(title)
        .bind(description)
        .bind(status)
        .bind(priority)
        .bind(due_date)
        .execute(pool)
        .await?;
    }

    Ok(())
}
