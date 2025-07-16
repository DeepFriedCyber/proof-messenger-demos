// Performance tests for the relay server
// These tests measure the performance of key operations

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use proof_messenger_relay::{Message, database::Database};
use serde_json::json;
use std::{sync::Arc, time::Instant};
use tower::ServiceExt;
use proof_messenger_protocol::key::generate_keypair_with_seed;

// Helper function to create a test app with database
async fn create_test_app() -> Router {
    let db = Database::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    let db = Arc::new(db);

    // Create a simplified router for performance testing
    axum::Router::new()
        .route("/relay", axum::routing::post(relay_handler))
        .route("/health", axum::routing::get(health_handler))
        .with_state(db)
}

// Simplified relay handler for performance testing
async fn relay_handler(
    axum::extract::State(db): axum::extract::State<Arc<Database>>,
    axum::extract::Json(payload): axum::extract::Json<Message>,
) -> impl axum::response::IntoResponse {
    // Process message
    let stored_message = proof_messenger_relay::database::StoredMessage::from(payload);
    match db.store_message(stored_message).await {
        Ok(message_id) => {
            (StatusCode::OK, axum::Json(json!({
                "status": "success",
                "message_id": message_id
            })))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(json!({
                "status": "error",
                "message": e.to_string()
            })))
        }
    }
}

// Health check handler for performance testing
async fn health_handler(
    axum::extract::State(db): axum::extract::State<Arc<Database>>,
) -> impl axum::response::IntoResponse {
    match db.health_check().await {
        Ok(_) => {
            (StatusCode::OK, axum::Json(json!({
                "status": "healthy",
                "database": "connected",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            (StatusCode::SERVICE_UNAVAILABLE, axum::Json(json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string()
            })))
        }
    }
}

// Helper function to create a valid test message
fn create_test_message(keypair_seed: u64, context: &[u8], body: &str) -> Message {
    let keypair = generate_keypair_with_seed(keypair_seed);
    let signature = keypair.sign(context);
    
    Message {
        sender: hex::encode(keypair.public.to_bytes()),
        context: hex::encode(context),
        body: body.to_string(),
        proof: hex::encode(signature.to_bytes()),
    }
}

#[tokio::test]
async fn test_health_check_performance() {
    // ARRANGE: Create test app
    let app = create_test_app().await;
    
    // ACT: Measure health check response time
    let start = Instant::now();
    
    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let duration = start.elapsed();
    
    // ASSERT: Health check should be fast
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration.as_millis() < 100, "Health check took too long: {:?}", duration);
    
    println!("Health check response time: {:?}", duration);
}

#[tokio::test]
async fn test_message_relay_performance() {
    // ARRANGE: Create test app and message
    let app = create_test_app().await;
    let context = b"performance test context";
    let message = create_test_message(42, context, "Performance test message");
    
    // ACT: Measure relay response time
    let start = Instant::now();
    
    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/relay")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&message).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let duration = start.elapsed();
    
    // ASSERT: Message relay should be reasonably fast
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration.as_millis() < 200, "Message relay took too long: {:?}", duration);
    
    println!("Message relay response time: {:?}", duration);
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    // ARRANGE: Create test app
    let app = create_test_app().await;
    
    // ACT: Make multiple concurrent requests
    let start = Instant::now();
    
    let mut handles = vec![];
    for _ in 0..10 {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            app_clone
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
        }));
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    let duration = start.elapsed();
    
    // ASSERT: Concurrent requests should be handled efficiently
    assert!(duration.as_millis() < 500, "Concurrent health checks took too long: {:?}", duration);
    
    println!("10 concurrent health checks response time: {:?}", duration);
}

#[tokio::test]
async fn test_message_throughput() {
    // ARRANGE: Create test app
    let app = create_test_app().await;
    let num_messages = 50;
    
    // ACT: Send multiple messages in sequence
    let start = Instant::now();
    
    for i in 0..num_messages {
        let context = format!("throughput test context {}", i);
        let message = create_test_message(100 + i as u64, context.as_bytes(), &format!("Throughput test message {}", i));
        
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/relay")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&message).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    let duration = start.elapsed();
    let throughput = num_messages as f64 / duration.as_secs_f64();
    
    // ASSERT: Throughput should be reasonable
    println!("Message throughput: {:.2} messages/second", throughput);
    assert!(throughput > 10.0, "Message throughput too low: {:.2} messages/second", throughput);
}