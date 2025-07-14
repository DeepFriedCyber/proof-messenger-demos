use proof_messenger_relay::{database::Database, create_app_with_rate_limiting};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:messages.db".to_string());
    
    info!("Connecting to database: {}", database_url);
    let db = Database::new(&database_url).await
        .expect("Failed to connect to database");
    
    info!("Running database migrations...");
    db.migrate().await
        .expect("Failed to run database migrations");
    
    let db = Arc::new(db);

    let app = create_app_with_rate_limiting(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    info!("🚀 Relay server starting...");
    info!("📡 Listening on 0.0.0.0:8080");
    info!("💾 Database initialized and ready");
    info!("✅ Server ready to accept connections");
    
    axum::serve(listener, app).await.unwrap();
}