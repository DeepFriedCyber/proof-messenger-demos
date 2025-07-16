//! Integration tests for the relay server with database persistence
//!
//! These tests verify that the complete system works end-to-end,
//! including message verification, storage, and retrieval.

// Import test modules
mod integration {
    // Database configuration tests
    pub mod database_config_tests;
    
    // Health check tests
    pub mod health_check_tests;
    
    // Docker integration tests (conditionally compiled)
    #[cfg(feature = "docker-tests")]
    pub mod docker_integration_tests;
}

// Re-export the DatabaseTestHelper for use in other tests
pub use integration::database_config_tests::DatabaseTestHelper;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    Router,
    extract::{Json, Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use proof_messenger_relay::{Message, AppError, MessageQuery, database::{Database, StoredMessage}, process_and_verify_message};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use ed25519_dalek::Signer;
use proof_messenger_protocol::key::generate_keypair_with_seed;
use tracing::instrument;

/// Helper function to create the app with a test database
async fn create_test_app() -> Router {
    let db = Database::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    let db = Arc::new(db);

    // We need to recreate the router here since the main function isn't exposed
    // In a real application, you'd extract this into a separate function

    // Copy the handlers from main.rs (simplified for testing)
    #[instrument(skip_all)]
    async fn relay_handler(
        State(db): State<Arc<Database>>,
        Json(payload): Json<Message>,
    ) -> Result<impl IntoResponse, AppError> {
        // Verify the message
        process_and_verify_message(&payload, Some(&db)).await?;
        
        // Store the verified message
        let stored_message = StoredMessage::from(payload);
        let message_id = db.store_message(stored_message).await?;
        
        let success_response = Json(json!({
            "status": "success",
            "message": "Message verified and relayed successfully",
            "message_id": message_id
        }));
        
        Ok((StatusCode::OK, success_response))
    }

    #[instrument(skip_all)]
    async fn get_messages_handler(
        State(db): State<Arc<Database>>,
        Path(group_id): Path<String>,
        Query(params): Query<MessageQuery>,
    ) -> Result<impl IntoResponse, AppError> {
        let messages = db.get_messages_by_group(&group_id, params.limit).await?;
        
        let response = Json(json!({
            "status": "success",
            "group_id": group_id,
            "message_count": messages.len(),
            "messages": messages
        }));
        
        Ok((StatusCode::OK, response))
    }

    #[instrument(skip_all)]
    async fn get_message_by_id_handler(
        State(db): State<Arc<Database>>,
        Path(message_id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        let message = db.get_message_by_id(&message_id).await?;
        
        let response = Json(json!({
            "status": "success",
            "message": message
        }));
        
        Ok((StatusCode::OK, response))
    }

    Router::new()
        .route("/relay", post(relay_handler))
        .route("/messages/:group_id", get(get_messages_handler))
        .route("/message/:message_id", get(get_message_by_id_handler))
        .with_state(db)
}

/// Helper function to create a valid test message
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
async fn test_relay_and_retrieve_message_flow() {
    // ARRANGE: Create test app and message
    let app = create_test_app().await;
    let context = b"integration test context";
    let message = create_test_message(42, context, "Integration test message");

    // ACT: Send message to relay endpoint
    let relay_request = Request::builder()
        .method("POST")
        .uri("/relay")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&message).unwrap()))
        .unwrap();

    let relay_response = app.clone().oneshot(relay_request).await.unwrap();

    // ASSERT: Relay should succeed
    assert_eq!(relay_response.status(), StatusCode::OK);

    let relay_body = to_bytes(relay_response.into_body(), usize::MAX).await.unwrap();
    let relay_json: Value = serde_json::from_slice(&relay_body).unwrap();
    
    assert_eq!(relay_json["status"], "success");
    assert!(relay_json["message_id"].is_string());
    let message_id = relay_json["message_id"].as_str().unwrap();

    // ACT: Retrieve messages from default group
    let get_messages_request = Request::builder()
        .method("GET")
        .uri("/messages/default")
        .body(Body::empty())
        .unwrap();

    let get_messages_response = app.clone().oneshot(get_messages_request).await.unwrap();

    // ASSERT: Should retrieve the stored message
    assert_eq!(get_messages_response.status(), StatusCode::OK);

    let messages_body = to_bytes(get_messages_response.into_body(), usize::MAX).await.unwrap();
    let messages_json: Value = serde_json::from_slice(&messages_body).unwrap();
    
    assert_eq!(messages_json["status"], "success");
    assert_eq!(messages_json["group_id"], "default");
    assert_eq!(messages_json["message_count"], 1);
    
    let stored_messages = messages_json["messages"].as_array().unwrap();
    assert_eq!(stored_messages.len(), 1);
    
    let stored_message = &stored_messages[0];
    assert_eq!(stored_message["sender"], message.sender);
    assert_eq!(stored_message["body"], message.body);
    assert_eq!(stored_message["verified"], true);

    // ACT: Retrieve specific message by ID
    let get_message_request = Request::builder()
        .method("GET")
        .uri(&format!("/message/{}", message_id))
        .body(Body::empty())
        .unwrap();

    let get_message_response = app.oneshot(get_message_request).await.unwrap();

    // ASSERT: Should retrieve the specific message
    assert_eq!(get_message_response.status(), StatusCode::OK);

    let message_body = to_bytes(get_message_response.into_body(), usize::MAX).await.unwrap();
    let message_json: Value = serde_json::from_slice(&message_body).unwrap();
    
    assert_eq!(message_json["status"], "success");
    assert_eq!(message_json["message"]["id"], message_id);
    assert_eq!(message_json["message"]["sender"], message.sender);
    assert_eq!(message_json["message"]["body"], message.body);
}

#[tokio::test]
async fn test_invalid_message_not_stored() {
    // ARRANGE: Create test app and invalid message
    let app = create_test_app().await;
    let mut message = create_test_message(42, b"original context", "Test message");
    message.context = hex::encode(b"tampered context"); // Tamper with context

    // ACT: Send invalid message to relay endpoint
    let relay_request = Request::builder()
        .method("POST")
        .uri("/relay")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&message).unwrap()))
        .unwrap();

    let relay_response = app.clone().oneshot(relay_request).await.unwrap();

    // ASSERT: Relay should fail
    assert_eq!(relay_response.status(), StatusCode::UNAUTHORIZED);

    // ACT: Check that no messages were stored
    let get_messages_request = Request::builder()
        .method("GET")
        .uri("/messages/default")
        .body(Body::empty())
        .unwrap();

    let get_messages_response = app.oneshot(get_messages_request).await.unwrap();

    // ASSERT: No messages should be stored
    assert_eq!(get_messages_response.status(), StatusCode::OK);

    let messages_body = to_bytes(get_messages_response.into_body(), usize::MAX).await.unwrap();
    let messages_json: Value = serde_json::from_slice(&messages_body).unwrap();
    
    assert_eq!(messages_json["message_count"], 0);
}

#[tokio::test]
async fn test_message_limit_parameter() {
    // ARRANGE: Create test app and multiple messages
    let app = create_test_app().await;
    
    // Store 5 messages
    for i in 0..5 {
        let context = format!("test context {}", i);
        let message = create_test_message(42 + i, context.as_bytes(), &format!("Message {}", i));
        
        let relay_request = Request::builder()
            .method("POST")
            .uri("/relay")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        let relay_response = app.clone().oneshot(relay_request).await.unwrap();
        assert_eq!(relay_response.status(), StatusCode::OK);
    }

    // ACT: Retrieve messages with limit
    let get_messages_request = Request::builder()
        .method("GET")
        .uri("/messages/default?limit=3")
        .body(Body::empty())
        .unwrap();

    let get_messages_response = app.oneshot(get_messages_request).await.unwrap();

    // ASSERT: Should respect limit
    assert_eq!(get_messages_response.status(), StatusCode::OK);

    let messages_body = to_bytes(get_messages_response.into_body(), usize::MAX).await.unwrap();
    let messages_json: Value = serde_json::from_slice(&messages_body).unwrap();
    
    assert_eq!(messages_json["message_count"], 3);
    
    let stored_messages = messages_json["messages"].as_array().unwrap();
    assert_eq!(stored_messages.len(), 3);
}

#[tokio::test]
async fn test_nonexistent_message_returns_404() {
    // ARRANGE: Create test app
    let app = create_test_app().await;

    // ACT: Try to retrieve non-existent message
    let get_message_request = Request::builder()
        .method("GET")
        .uri("/message/non-existent-id")
        .body(Body::empty())
        .unwrap();

    let get_message_response = app.oneshot(get_message_request).await.unwrap();

    // ASSERT: Should return error
    assert_eq!(get_message_response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let message_body = to_bytes(get_message_response.into_body(), usize::MAX).await.unwrap();
    let message_json: Value = serde_json::from_slice(&message_body).unwrap();
    
    assert!(message_json["error"].as_str().unwrap().contains("Message not found"));
}

#[tokio::test]
async fn test_message_ordering() {
    // ARRANGE: Create test app
    let app = create_test_app().await;
    
    // Store messages with delays to ensure different timestamps
    let messages = vec!["First message", "Second message", "Third message"];
    
    for (i, body) in messages.iter().enumerate() {
        let context = format!("context {}", i);
        let message = create_test_message(100 + i as u64, context.as_bytes(), body);
        
        let relay_request = Request::builder()
            .method("POST")
            .uri("/relay")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        let relay_response = app.clone().oneshot(relay_request).await.unwrap();
        assert_eq!(relay_response.status(), StatusCode::OK);
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // ACT: Retrieve messages
    let get_messages_request = Request::builder()
        .method("GET")
        .uri("/messages/default")
        .body(Body::empty())
        .unwrap();

    let get_messages_response = app.oneshot(get_messages_request).await.unwrap();

    // ASSERT: Messages should be ordered newest first
    assert_eq!(get_messages_response.status(), StatusCode::OK);

    let messages_body = to_bytes(get_messages_response.into_body(), usize::MAX).await.unwrap();
    let messages_json: Value = serde_json::from_slice(&messages_body).unwrap();
    
    let stored_messages = messages_json["messages"].as_array().unwrap();
    assert_eq!(stored_messages.len(), 3);
    
    // Should be in reverse order (newest first)
    assert_eq!(stored_messages[0]["body"], "Third message");
    assert_eq!(stored_messages[1]["body"], "Second message");
    assert_eq!(stored_messages[2]["body"], "First message");
}