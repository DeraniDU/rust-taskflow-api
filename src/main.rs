use std::net::SocketAddr;

use rust_taskflow_api::{app::create_app, database::sqlite::connect_database, state::AppState};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://taskflow.db".to_string());

    let db = connect_database(&database_url)
        .await
        .expect("Failed to connect database");

    let app_state = AppState::new(db);
    let app = create_app(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
