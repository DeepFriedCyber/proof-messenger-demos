// proof-messenger-relay/tests/integration/database_config_test.rs
use proof_messenger_relay::database::DatabaseConfig;
use sqlx::SqlitePool;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod database_config_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection_with_valid_path() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_messages.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Act
        let pool = SqlitePool::connect(&db_url).await;

        // Assert
        assert!(pool.is_ok(), "Database connection should succeed with valid path");

        if let Ok(pool) = pool {
            pool.close().await;
        }
    }

    #[tokio::test]
    async fn test_database_connection_with_missing_directory() {
        // Arrange
        let db_path = "/nonexistent/directory/messages.db";
        let db_url = format!("sqlite:{}", db_path);

        // Act
        let pool = SqlitePool::connect(&db_url).await;

        // Assert
        assert!(pool.is_err(), "Database connection should fail with missing directory");
    }

    #[tokio::test]
    async fn test_database_migrations_idempotent() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("migration_test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Act - Run migrations twice
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        let result1 = sqlx::migrate!("./migrations").run(&pool).await;
        let result2 = sqlx::migrate!("./migrations").run(&pool).await;

        // Assert
        assert!(result1.is_ok(), "First migration should succeed");
        assert!(result2.is_ok(), "Second migration should be idempotent");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_database_permissions_in_container() {
        // This test specifically addresses the container permission issue
        // Arrange
        let db_path = "/app/db/messages.db";

        // Act - Check if directory exists and is writable
        let parent_dir = Path::new(db_path).parent().unwrap();
        let dir_exists = parent_dir.exists();
        let dir_writable = parent_dir.metadata()
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false);

        // Assert
        assert!(dir_exists, "Database directory should exist in container");
        assert!(dir_writable, "Database directory should be writable");
    }

    #[test]
    fn test_database_config_environment_variables() {
        // Test that the database configuration correctly reads environment variables
        // Arrange
        std::env::set_var("DATABASE_URL", "sqlite:/app/db/test.db");

        // Act
        let config = DatabaseConfig::from_env();

        // Assert
        assert_eq!(config.database_url, "sqlite:/app/db/test.db");

        // Cleanup
        std::env::remove_var("DATABASE_URL");
    }
}

// Additional utility for database testing
pub struct DatabaseTestHelper {
    pub pool: SqlitePool,
    pub temp_dir: TempDir,
}

impl DatabaseTestHelper {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        Self { pool, temp_dir }
    }
    
    pub async fn cleanup(self) {
        self.pool.close().await;
        drop(self.temp_dir); // Automatically cleans up temp directory
    }
}