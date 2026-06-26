# Rust TaskFlow API

![Rust CI Pipeline](https://github.com/DeraniDU/rust-taskflow-api/actions/workflows/rust-ci.yml/badge.svg)

Rust TaskFlow API is a task management REST API built with Rust.
This project was created to practice intermediate Rust backend development, REST API design, SQLite database integration, testing, Docker support, GitHub Actions CI/CD, version control workflow, and automation-ready API design for n8n.

---

## Project Purpose

The purpose of this project is to build a professional intermediate-level Rust backend application.

This project demonstrates:

* Rust project structure
* Structs and enums
* Ownership and borrowing
* Async programming
* REST API development
* JSON request and response handling
* SQLite database integration
* SQLx database queries
* Custom API error handling
* API key authentication
* Unit testing
* Integration testing
* Docker support
* GitHub Actions CI/CD pipeline
* Git branching and Pull Request workflow
* n8n-ready automation API design

---

## Tech Stack

| Technology     | Purpose                                |
| -------------- | -------------------------------------- |
| Rust           | Main programming language              |
| Axum           | Web framework for REST API             |
| Tokio          | Async runtime                          |
| Serde          | JSON serialization and deserialization |
| SQLx           | Database access                        |
| SQLite         | Local database                         |
| Docker         | Containerization                       |
| GitHub Actions | CI/CD pipeline                         |
| Git            | Version control                        |
| n8n            | Future workflow automation integration |

---

## Features

* Health check endpoint
* Create task
* View all tasks
* View one task by ID
* Update task
* Delete task
* Filter tasks by status, priority, and due date
* SQLite database persistence
* Task priority support
* Due date support
* Created and updated timestamps
* API key authentication
* Custom error responses
* Unit tests
* Integration tests
* Docker image build support
* Automated GitHub Actions pipeline

---

## Project Structure

```text
rust-taskflow-api/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app.rs
│   ├── auth.rs
│   ├── state.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── task.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── tasks.rs
│   ├── database/
│   │   ├── mod.rs
│   │   └── sqlite.rs
│   └── errors/
│       └── mod.rs
├── tests/
│   └── api_tests.rs
├── .github/
│   └── workflows/
│       └── rust-ci.yml
├── Dockerfile
├── .dockerignore
├── .env.example
├── Cargo.toml
├── Cargo.lock
└── README.md
```

---

## How the Application Works

Request flow:

```text
User request
    ↓
Axum Router
    ↓
Route function
    ↓
API key validation
    ↓
AppState
    ↓
SQLite database
    ↓
Task model
    ↓
JSON response
```

Example:

```text
GET /tasks
    ↓
routes/tasks.rs
    ↓
auth.rs checks x-api-key
    ↓
state.rs provides database connection
    ↓
SQLite SELECT query runs
    ↓
Database rows become Task structs
    ↓
API returns JSON response
```

---

## API Endpoints

| Method | Endpoint      | Description         | API Key Required |
| ------ | ------------- | ------------------- | ---------------- |
| GET    | `/health`     | Check API status    | No               |
| GET    | `/tasks`      | Get all tasks       | Yes              |
| GET    | `/tasks/{id}` | Get one task by ID  | Yes              |
| POST   | `/tasks`      | Create a new task   | Yes              |
| PUT    | `/tasks/{id}` | Update a task by ID | Yes              |
| DELETE | `/tasks/{id}` | Delete a task by ID | Yes              |

---

## API Key Authentication

All task endpoints are protected using an API key.

Protected requests must include this header:

```text
x-api-key: dev-secret-key
```

The default local API key is:

```text
dev-secret-key
```

The API key can be changed using the environment variable:

```text
API_KEY=your-secret-api-key
```

Do not commit real secret API keys to GitHub.

---

## Environment Variables

Create a `.env` file locally if needed.

Example:

```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite://taskflow.db
API_KEY=dev-secret-key
```

This project also includes:

```text
.env.example
```

Use `.env.example` as a guide. Do not commit `.env`.

---

## Run Project Locally

Install Rust first.

Then run:

```bash
cargo run
```

The server will start at:

```text
http://127.0.0.1:3000
```

---

## API Request Examples

### Health Check

This endpoint does not require an API key.

```bash
curl http://127.0.0.1:3000/health
```

Example response:

```json
{
  "status": "ok",
  "message": "Rust TaskFlow API is running"
}
```

---

### Get All Tasks

```bash
curl http://127.0.0.1:3000/tasks \
  -H "x-api-key: dev-secret-key"
```

---

### Get One Task By ID

```bash
curl http://127.0.0.1:3000/tasks/1 \
  -H "x-api-key: dev-secret-key"
```

---

### Create Task

```bash
curl -X POST http://127.0.0.1:3000/tasks \
  -H "Content-Type: application/json" \
  -H "x-api-key: dev-secret-key" \
  -d '{"title":"Learn Rust API","description":"Practice Axum and SQLite","priority":"high","due_date":"2026-07-01"}'
```

Example response:

```json
{
  "id": 1,
  "title": "Learn Rust API",
  "description": "Practice Axum and SQLite",
  "status": "pending",
  "priority": "high",
  "due_date": "2026-07-01",
  "created_at": "2026-06-26 10:30:00",
  "updated_at": "2026-06-26 10:30:00"
}
```

---

### Update Task

```bash
curl -X PUT http://127.0.0.1:3000/tasks/1 \
  -H "Content-Type: application/json" \
  -H "x-api-key: dev-secret-key" \
  -d '{"title":"Updated task","description":"Updated description","status":"completed","priority":"medium","due_date":"2026-07-05"}'
```

---

### Delete Task

```bash
curl -X DELETE http://127.0.0.1:3000/tasks/1 \
  -H "x-api-key: dev-secret-key"
```

Successful delete returns:

```text
204 No Content
```

---

## Task Fields

| Field         | Description                               |
| ------------- | ----------------------------------------- |
| `id`          | Unique task ID                            |
| `title`       | Task title                                |
| `description` | Task description                          |
| `status`      | Task status: `pending` or `completed`     |
| `priority`    | Task priority: `low`, `medium`, or `high` |
| `due_date`    | Optional due date                         |
| `created_at`  | Date and time the task was created        |
| `updated_at`  | Date and time the task was last updated   |

---

## Task Filtering

The `GET /tasks` endpoint supports query filters.

| Filter           | Example                               |
| ---------------- | ------------------------------------- |
| `status`         | `/tasks?status=pending`               |
| `priority`       | `/tasks?priority=high`                |
| `due_date`       | `/tasks?due_date=2026-07-01`          |
| Combined filters | `/tasks?status=pending&priority=high` |

### Filter by Status

```bash
curl "http://127.0.0.1:3000/tasks?status=pending" \
  -H "x-api-key: dev-secret-key"
```

### Filter by Priority

```bash
curl "http://127.0.0.1:3000/tasks?priority=high" \
  -H "x-api-key: dev-secret-key"
```

### Filter by Due Date

```bash
curl "http://127.0.0.1:3000/tasks?due_date=2026-07-01" \
  -H "x-api-key: dev-secret-key"
```

### Combined Filter

```bash
curl "http://127.0.0.1:3000/tasks?status=pending&priority=high" \
  -H "x-api-key: dev-secret-key"
```

---

## Error Response Examples

### Missing API Key

```json
{
  "error": "Invalid or missing API key"
}
```

Status code:

```text
401 Unauthorized
```

### Empty Task Title

```json
{
  "error": "Task title is required"
}
```

Status code:

```text
400 Bad Request
```

### Task Not Found

```json
{
  "error": "Task not found"
}
```

Status code:

```text
404 Not Found
```

---

## Database

This project uses SQLite.

Default database file:

```text
taskflow.db
```

The database is created automatically when the application starts.

The tasks table includes:

```sql
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
```

---

## View SQLite Database

Open SQLite:

```bash
sqlite3 taskflow.db
```

Show tables:

```sql
.tables
```

Show task data:

```sql
SELECT * FROM tasks;
```

Better view:

```sql
.headers on
.mode column
SELECT * FROM tasks;
```

Exit:

```sql
.exit
```

---

## Run Checks

Before committing code, run:

```bash
cargo fmt
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Purpose:

| Command                       | Purpose                         |
| ----------------------------- | ------------------------------- |
| `cargo fmt`                   | Format Rust code                |
| `cargo fmt --check`           | Check formatting                |
| `cargo clippy -- -D warnings` | Check code quality              |
| `cargo test`                  | Run tests                       |
| `cargo build --release`       | Build optimized release version |

---

## Testing

This project includes:

* Unit tests
* API integration tests

Run tests:

```bash
cargo test
```

Current tests cover:

* Health check
* API key protection
* Get all tasks
* Get task by ID
* Create task
* Update task
* Delete task
* Not found responses
* Validation errors
* Task filtering

---

## Docker Support

This project includes Docker support.

Build Docker image:

```bash
docker build -t rust-taskflow-api .
```

Run Docker container:

```bash
docker run -p 3000:3000 rust-taskflow-api
```

Docker is optional for local development.
The project can run normally using:

```bash
cargo run
```

If Docker is not installed locally, GitHub Actions can still test the Docker build in the CI pipeline.

---

## CI/CD Pipeline

This project uses GitHub Actions.

Workflow file:

```text
.github/workflows/rust-ci.yml
```

The pipeline runs automatically on:

* Push to `main`
* Pull Request to `main`

Pipeline checks:

```text
1. Code formatting
2. Clippy linting
3. Tests
4. Release build
5. Docker image build
```

This helps ensure that every Pull Request is checked before merging.

---

## Version Control Workflow

This project follows a feature branch workflow.

Start from main:

```bash
git checkout main
git pull origin main
```

Create a feature branch:

```bash
git checkout -b feature/example-feature
```

After completing changes:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Commit:

```bash
git add .
git commit -m "Add example feature"
```

Push branch:

```bash
git push -u origin feature/example-feature
```

Then create a Pull Request on GitHub.

Workflow:

```text
main branch
    ↓
feature branch
    ↓
commit changes
    ↓
push branch
    ↓
create Pull Request
    ↓
GitHub Actions pipeline runs
    ↓
merge after checks pass
```

---

## n8n Automation Use Case

This API is designed to support future n8n automation.

Example n8n workflow:

```text
Schedule Trigger
    ↓
HTTP Request: GET /tasks?status=pending&priority=high
    ↓
Filter due tasks
    ↓
Send email or notification
```

n8n can call this API using the HTTP Request node.

Example n8n request:

```text
Method: GET
URL: http://YOUR_API_URL/tasks?status=pending&priority=high
Header:
x-api-key: your-api-key
```

Possible n8n automations:

* Daily pending task reminder
* High-priority task alert
* Google Form submission creates task
* Gmail message creates task
* Weekly task summary email
* Completed task report

---

## Learning Outcomes

Through this project, I practiced:

* Rust backend development
* Axum routing
* JSON APIs
* SQLite database integration
* SQLx queries
* Custom error handling
* API key authentication
* Unit testing
* Integration testing
* Docker containerization
* GitHub Actions CI/CD
* Git branching and Pull Requests
* API design for workflow automation

---

## Future Improvements

Possible next features:

* Pagination
* Search tasks by title
* Sort tasks by due date
* User authentication with JWT
* Role-based access control
* Frontend using React
* n8n workflow documentation
* Cloud deployment
* OpenAPI/Swagger documentation
* Task categories or tags

---

## Author

Created as an intermediate Rust learning project.
