use std::net::{IpAddr, SocketAddr};

use rust_taskflow_api::{app::create_app, database::sqlite::connect_database, state::AppState};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://taskflow.db".to_string());

    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "dev-secret-key".to_string());

    let db = connect_database(&database_url)
        .await
        .expect("Failed to connect database");

    let app_state = AppState::new(db, api_key);
    let app = create_app(app_state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(3000);

    let ip: IpAddr = host.parse().expect("Invalid HOST value");
    let addr = SocketAddr::new(ip, port);

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
