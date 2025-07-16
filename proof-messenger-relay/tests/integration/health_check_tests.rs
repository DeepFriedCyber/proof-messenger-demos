// proof-messenger-relay/tests/integration/health_check_test.rs
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::timeout;
use tower::ServiceExt;

#[cfg(test)]
mod health_check_tests {
    use super::*;
    use crate::integration::database_config_tests::DatabaseTestHelper;

    #[tokio::test]
    async fn test_health_check_with_healthy_database() {
        // Arrange
        let db_helper = DatabaseTestHelper::new().await;
        let app = create_test_app_with_db(db_helper.pool.clone()).await;

        // Act
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health_response: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response["status"], "healthy");
        assert_eq!(health_response["database"], "connected");
        assert!(health_response["timestamp"].is_string());

        db_helper.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check_with_disconnected_database() {
        // Arrange
        let db_helper = DatabaseTestHelper::new().await;
        let pool = db_helper.pool.clone();

        // Close the database connection to simulate failure
        pool.close().await;

        let app = create_test_app_with_db(pool).await;

        // Act
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health_response: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response["status"], "unhealthy");
        assert_eq!(health_response["database"], "disconnected");

        db_helper.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check_response_time() {
        // Arrange
        let db_helper = DatabaseTestHelper::new().await;
        let app = create_test_app_with_db(db_helper.pool.clone()).await;

        // Act
        let start = std::time::Instant::now();
        let response = timeout(
            Duration::from_secs(1),
            app.oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            ),
        ).await;
        let elapsed = start.elapsed();

        // Assert
        assert!(response.is_ok(), "Health check should complete within 1 second");
        assert!(elapsed < Duration::from_millis(500), "Health check should be fast");

        db_helper.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check_concurrent_requests() {
        // Arrange
        let db_helper = DatabaseTestHelper::new().await;
        let app = create_test_app_with_db(db_helper.pool.clone()).await;

        // Act - Make multiple concurrent requests
        let mut handles = vec![];
        for _ in 0..10 {
            let app_clone = app.clone();
            handles.push(tokio::spawn(async move {
                app_clone
                    .oneshot(
                        axum::http::Request::builder()
                            .uri("/health")
                            .body(axum::body::Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap()
            }));
        }

        // Assert
        for handle in handles {
            let response = handle.await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        db_helper.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check_detailed_response_format() {
        // Arrange
        let db_helper = DatabaseTestHelper::new().await;
        let app = create_test_app_with_db(db_helper.pool.clone()).await;

        // Act
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health_response: Value = serde_json::from_slice(&body).unwrap();

        // Verify required fields
        assert!(health_response.get("status").is_some());
        assert!(health_response.get("database").is_some());
        assert!(health_response.get("timestamp").is_some());
        assert!(health_response.get("version").is_some());

        // Verify response format
        assert!(health_response["status"].is_string());
        assert!(health_response["database"].is_string());
        assert!(health_response["timestamp"].is_string());

        db_helper.cleanup().await;
    }
}

// Helper function to create test app with database
async fn create_test_app_with_db(pool: SqlitePool) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        .route("/health", get(health_check_handler))
        .with_state(pool)
}

// Improved health check handler
async fn health_check_handler(
    axum::extract::State(pool): axum::extract::State<SqlitePool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Test database connection
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => Ok(Json(json!({
            "status": "healthy",
            "database": "connected",
            "timestamp": timestamp,
            "version": env!("CARGO_PKG_VERSION")
        }))),
        Err(e) => {
            eprintln!("Health check failed: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "database": "disconnected",
                    "timestamp": timestamp,
                    "error": e.to_string()
                }))
            ))
        }
    }
}