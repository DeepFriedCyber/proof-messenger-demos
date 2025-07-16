use proof_messenger_relay::{database::Database, create_app_with_rate_limiting};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:/app/db/messages.db".to_string());
    
    info!("Connecting to database: {}", database_url);
    
    // Debug: Check current directory and permissions
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    info!("Current directory: {:?}", current_dir);
    
    // Try to create a test file to verify write permissions
    match std::fs::File::create("test_permissions.txt") {
        Ok(_) => info!("Successfully created test file"),
        Err(e) => info!("Failed to create test file: {}", e),
    }
    
    // Try to create a test file in /app/db to verify write permissions
    match std::fs::File::create("/app/db/test_permissions.txt") {
        Ok(_) => info!("Successfully created test file in /app/db"),
        Err(e) => info!("Failed to create test file in /app/db: {}", e),
    }
    
    // Try to create the database file explicitly if it doesn't exist
    let db_path = database_url.strip_prefix("sqlite://").unwrap_or(&database_url);
    info!("Database path: {}", db_path);
    
    // Ensure the directory exists
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        if !parent.exists() {
            info!("Database directory does not exist, attempting to create it");
            match std::fs::create_dir_all(parent) {
                Ok(_) => info!("Successfully created database directory"),
                Err(e) => info!("Failed to create database directory: {}", e),
            }
        }
    }
    
    if !std::path::Path::new(db_path).exists() {
        info!("Database file does not exist, attempting to create it");
        match std::fs::File::create(db_path) {
            Ok(_) => info!("Successfully created database file"),
            Err(e) => info!("Failed to create database file: {}", e),
        }
    }
    
    // Connect to database with better error handling
    let db = match Database::new(&database_url).await {
        Ok(db) => {
            info!("Successfully connected to database");
            db
        },
        Err(e) => {
            info!("Database connection error: {:?}", e);
            panic!("Failed to connect to database: {:?}", e);
        }
    };
    
    info!("Running database migrations...");
    match db.migrate().await {
        Ok(_) => info!("Database migrations completed successfully"),
        Err(e) => {
            info!("Database migration error: {:?}", e);
            panic!("Failed to run database migrations: {:?}", e);
        }
    };
    
    let db = Arc::new(db);

    let app = create_app_with_rate_limiting(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    info!("🚀 Relay server starting...");
    info!("📡 Listening on 0.0.0.0:8080");
    info!("💾 Database initialized and ready");
    info!("✅ Server ready to accept connections");
    
    axum::serve(listener, app).await.unwrap();
}