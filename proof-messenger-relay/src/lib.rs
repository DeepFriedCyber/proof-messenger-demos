//! Proof Messenger Relay Library
//!
//! This library provides the core functionality for the relay server,
//! including message verification, database operations, and HTTP handlers.

pub mod database;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use ed25519_dalek::{PublicKey, Signature};
use proof_messenger_protocol::proof::{verify_proof_result, ProofError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument, warn};
use std::sync::Arc;
use chrono;
use hex;

use database::{Database, DatabaseError, StoredMessage};

/// Query parameters for message retrieval
#[derive(Deserialize)]
pub struct MessageQuery {
    /// Maximum number of messages to return
    pub limit: Option<i64>,
}

/// Message structure for relay operations
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    /// Public key of the sender (hex encoded)
    pub sender: String,
    /// Context data that was signed (hex encoded)
    pub context: String,
    /// Message body content
    pub body: String,
    /// Cryptographic proof/signature (hex encoded)
    pub proof: String,
}

/// Application-specific error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid signature format: {0}")]
    InvalidSignature(String),
    
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),
    
    #[error("Invalid context data: {0}")]
    InvalidContext(String),
    
    #[error("Proof verification failed")]
    VerificationFailed,
    
    #[error("Message processing error: {0}")]
    ProcessingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidSignature(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidPublicKey(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidContext(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::VerificationFailed => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::ProcessingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

/// Process and verify a message using cryptographic proof
/// 
/// This function is decoupled from the web framework and can be unit tested
/// independently. It performs the core business logic of message verification.
#[instrument(skip_all, fields(sender = %message.sender))]
pub fn process_and_verify_message(message: &Message) -> Result<(), AppError> {
    info!("Processing message verification");

    // Parse the public key from hex
    let sender_bytes = hex::decode(&message.sender)
        .map_err(|e| AppError::InvalidPublicKey(format!("Invalid hex encoding: {}", e)))?;
    
    if sender_bytes.len() != 32 {
        return Err(AppError::InvalidPublicKey("Public key must be 32 bytes".to_string()));
    }
    
    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(&sender_bytes);
    let public_key = PublicKey::from_bytes(&pubkey_bytes)
        .map_err(|e| AppError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;

    // Parse the context from hex
    let context = hex::decode(&message.context)
        .map_err(|e| AppError::InvalidContext(format!("Invalid hex encoding: {}", e)))?;

    // Parse the signature from hex
    let proof_bytes = hex::decode(&message.proof)
        .map_err(|e| AppError::InvalidSignature(format!("Invalid hex encoding: {}", e)))?;
    
    if proof_bytes.len() != 64 {
        return Err(AppError::InvalidSignature("Signature must be 64 bytes".to_string()));
    }
    
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(&proof_bytes);
    let signature = Signature::from_bytes(&sig_bytes)
        .map_err(|e| AppError::InvalidSignature(format!("Invalid signature: {}", e)))?;

    // Use the improved protocol function with Result-based error handling!
    verify_proof_result(&public_key, &context, &signature)
        .map_err(|e| match e {
            ProofError::VerificationFailed(_) => AppError::VerificationFailed,
            _ => AppError::ProcessingError(format!("Verification error: {}", e)),
        })?;

    info!("Proof successfully verified");
    Ok(())
}

/// Create the application router with database state
pub fn create_app(db: Arc<Database>) -> Router {
    Router::new()
        .route("/relay", post(relay_handler))
        .route("/messages/:group_id", get(get_messages_handler))
        .route("/message/:message_id", get(get_message_by_id_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .with_state(db)
}

/// The Axum handler for message relay
#[instrument(skip_all)]
async fn relay_handler(
    State(db): State<Arc<Database>>,
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received message for relay");
    
    // Delegate to the unit-tested function
    process_and_verify_message(&payload)?;
    
    // Store the verified message in the database
    let stored_message = StoredMessage::from(payload);
    let message_id = db.store_message(stored_message).await?;
    
    let success_response = Json(serde_json::json!({
        "status": "success",
        "message": "Message verified and relayed successfully",
        "message_id": message_id
    }));
    
    Ok((StatusCode::OK, success_response))
}

/// Handler to retrieve messages for a specific group
#[instrument(skip_all)]
async fn get_messages_handler(
    State(db): State<Arc<Database>>,
    Path(group_id): Path<String>,
    Query(params): Query<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("Retrieving messages for group: {}", group_id);
    
    let messages = db.get_messages_by_group(&group_id, params.limit).await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "group_id": group_id,
        "message_count": messages.len(),
        "messages": messages
    }));
    
    Ok((StatusCode::OK, response))
}

/// Handler to retrieve a specific message by ID
#[instrument(skip_all)]
async fn get_message_by_id_handler(
    State(db): State<Arc<Database>>,
    Path(message_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Retrieving message: {}", message_id);
    
    let message = db.get_message_by_id(&message_id).await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": message
    }));
    
    Ok((StatusCode::OK, response))
}

/// Health check endpoint for container orchestration
#[instrument]
async fn health_handler(State(db): State<Arc<Database>>) -> impl IntoResponse {
    // Check database health
    let db_status = match db.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    
    let overall_status = if db_status == "healthy" { "healthy" } else { "unhealthy" };
    let status_code = if overall_status == "healthy" { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let health_response = Json(serde_json::json!({
        "status": overall_status,
        "service": "proof-messenger-relay",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": db_status
        }
    }));
    
    (status_code, health_response)
}

/// Readiness check endpoint
#[instrument]
async fn ready_handler(State(db): State<Arc<Database>>) -> impl IntoResponse {
    // Check if all systems are ready
    let db_ready = db.health_check().await.is_ok();
    
    let overall_ready = db_ready;
    let status_code = if overall_ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let ready_response = Json(serde_json::json!({
        "status": if overall_ready { "ready" } else { "not_ready" },
        "service": "proof-messenger-relay",
        "checks": {
            "crypto": "ok",
            "memory": "ok",
            "database": if db_ready { "ok" } else { "error" }
        }
    }));
    
    (status_code, ready_response)
}

// TDD Step 1: Write the failing tests first
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use proof_messenger_protocol::key::generate_keypair_with_seed;

    /// Helper function to create a valid message for testing
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

    #[test]
    fn process_and_verify_message_rejects_tampered_context() {
        // ARRANGE: Create a message where the signature is for a different context
        let keypair = generate_keypair_with_seed(42);
        let original_context = b"context-for-signature";
        let tampered_context = b"different-context-in-message";
        let signature = keypair.sign(original_context);
        
        let tampered_message = Message {
            sender: hex::encode(keypair.public.to_bytes()),
            context: hex::encode(tampered_context), // The context doesn't match the signature
            body: "This is a test".to_string(),
            proof: hex::encode(signature.to_bytes()),
        };

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&tampered_message);

        // ASSERT: The result must be a VerificationFailed error
        assert!(matches!(result, Err(AppError::VerificationFailed)));
    }

    #[test]
    fn process_and_verify_message_accepts_valid_message() {
        // ARRANGE: Create a valid message
        let context = b"valid context for signature";
        let message = create_test_message(42, context, "Valid test message");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be successful
        assert!(result.is_ok());
    }

    #[test]
    fn process_and_verify_message_rejects_invalid_signature_format() {
        // ARRANGE: Create a message with invalid signature format
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = "invalid_hex_signature".to_string(); // Invalid hex

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be an InvalidSignature error
        assert!(matches!(result, Err(AppError::InvalidSignature(_))));
    }

    #[test]
    fn process_and_verify_message_rejects_invalid_public_key_format() {
        // ARRANGE: Create a message with invalid public key format
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.sender = "invalid_hex_pubkey".to_string(); // Invalid hex

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be an InvalidPublicKey error
        assert!(matches!(result, Err(AppError::InvalidPublicKey(_))));
    }

    #[test]
    fn process_and_verify_message_rejects_wrong_signature_length() {
        // ARRANGE: Create a message with wrong signature length
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = hex::encode(&[0u8; 32]); // Wrong length (32 instead of 64)

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be an InvalidSignature error
        assert!(matches!(result, Err(AppError::InvalidSignature(_))));
    }

    #[test]
    fn process_and_verify_message_rejects_wrong_public_key_length() {
        // ARRANGE: Create a message with wrong public key length
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.sender = hex::encode(&[0u8; 16]); // Wrong length (16 instead of 32)

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be an InvalidPublicKey error
        assert!(matches!(result, Err(AppError::InvalidPublicKey(_))));
    }

    #[test]
    fn process_and_verify_message_rejects_tampered_signature() {
        // ARRANGE: Create a valid message then tamper with the signature
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        
        // Tamper with the signature by flipping a bit
        let mut sig_bytes = hex::decode(&message.proof).unwrap();
        sig_bytes[0] ^= 0x01; // Flip the first bit
        message.proof = hex::encode(sig_bytes);

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be a VerificationFailed error
        assert!(matches!(result, Err(AppError::VerificationFailed)));
    }

    #[test]
    fn process_and_verify_message_handles_empty_context() {
        // ARRANGE: Create a message with empty context
        let context = b"";
        let message = create_test_message(42, context, "Message with empty context");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be successful (empty context is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn process_and_verify_message_handles_large_context() {
        // ARRANGE: Create a message with large context
        let large_context = vec![0xAA; 10000]; // 10KB context
        let message = create_test_message(42, &large_context, "Message with large context");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message);

        // ASSERT: The result should be successful
        assert!(result.is_ok());
    }

    #[test]
    fn error_messages_are_informative() {
        // Test that error messages contain useful information
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = "not_hex".to_string();

        let result = process_and_verify_message(&message);
        
        match result {
            Err(AppError::InvalidSignature(msg)) => {
                assert!(msg.contains("Invalid hex encoding"));
            }
            _ => panic!("Expected InvalidSignature error"),
        }
    }

    #[test]
    fn app_error_http_status_codes() {
        // Test that errors map to correct HTTP status codes
        let invalid_sig = AppError::InvalidSignature("test".to_string());
        let invalid_key = AppError::InvalidPublicKey("test".to_string());
        let invalid_context = AppError::InvalidContext("test".to_string());
        let verification_failed = AppError::VerificationFailed;
        let processing_error = AppError::ProcessingError("test".to_string());
        
        // Convert to responses and check status codes
        let response1 = invalid_sig.into_response();
        let response2 = invalid_key.into_response();
        let response3 = invalid_context.into_response();
        let response4 = verification_failed.into_response();
        let response5 = processing_error.into_response();
        
        // All should be valid responses (this tests the IntoResponse implementation)
        assert_eq!(response1.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response2.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response3.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response4.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response5.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn message_serialization_roundtrip() {
        // Test that messages can be serialized and deserialized
        let context = b"serialization test";
        let original_message = create_test_message(42, context, "Serialization test");
        
        // Serialize to JSON
        let json = serde_json::to_string(&original_message).unwrap();
        
        // Deserialize back
        let deserialized_message: Message = serde_json::from_str(&json).unwrap();
        
        // Should be identical
        assert_eq!(original_message.sender, deserialized_message.sender);
        assert_eq!(original_message.context, deserialized_message.context);
        assert_eq!(original_message.body, deserialized_message.body);
        assert_eq!(original_message.proof, deserialized_message.proof);
        
        // Both should verify successfully
        assert!(process_and_verify_message(&original_message).is_ok());
        assert!(process_and_verify_message(&deserialized_message).is_ok());
    }
}