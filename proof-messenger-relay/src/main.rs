use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use ed25519_dalek::{PublicKey, Signature};
use proof_messenger_protocol::proof::{verify_proof_result, ProofError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument, warn};

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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidSignature(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidPublicKey(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidContext(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::VerificationFailed => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::ProcessingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

// TDD Step 2: Implement the decoupled, testable business logic
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

// The Axum handler is now just a thin wrapper around the testable logic
#[instrument(skip_all)]
async fn relay_handler(
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received message for relay");
    
    // Delegate to the unit-tested function
    process_and_verify_message(&payload)?;
    
    let success_response = Json(serde_json::json!({
        "status": "success",
        "message": "Message verified and relayed successfully"
    }));
    
    Ok((StatusCode::OK, success_response))
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/relay", post(relay_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    info!("🚀 Relay server starting...");
    info!("📡 Listening on 0.0.0.0:8080");
    info!("✅ Server ready to accept connections");
    
    axum::serve(listener, app).await.unwrap();
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

    // Additional comprehensive tests for decoupled architecture
    
    #[test]
    fn process_and_verify_message_cross_keypair_verification_always_fails() {
        // Test that messages signed with one keypair never verify with another
        let context = b"cross verification test";
        let message1 = create_test_message(42, context, "Message 1");
        let message2 = create_test_message(43, context, "Message 2");
        
        // Try to verify message1's signature with message2's public key
        let cross_message = Message {
            sender: message2.sender,
            context: message1.context,
            body: message1.body,
            proof: message1.proof,
        };
        
        let result = process_and_verify_message(&cross_message);
        assert!(matches!(result, Err(AppError::VerificationFailed)));
    }

    #[test]
    fn process_and_verify_message_deterministic_behavior() {
        // Test that the same message always produces the same result
        let context = b"deterministic test";
        let message = create_test_message(42, context, "Deterministic message");
        
        // Verify multiple times
        for _ in 0..10 {
            let result = process_and_verify_message(&message);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn process_and_verify_message_handles_unicode_content() {
        // Test with Unicode content in the body
        let context = b"unicode test context";
        let message = create_test_message(42, context, "Unicode: 🚀 🔐 ✅ 测试");
        
        let result = process_and_verify_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn process_and_verify_message_performance_test() {
        // Test that verification is reasonably fast
        let context = b"performance test";
        let message = create_test_message(42, context, "Performance test message");
        
        let start = std::time::Instant::now();
        
        // Run 100 verifications
        for _ in 0..100 {
            let result = process_and_verify_message(&message);
            assert!(result.is_ok());
        }
        
        let duration = start.elapsed();
        
        // Should complete 100 verifications in under 1 second
        assert!(duration.as_secs() < 1, "Verification too slow: {:?}", duration);
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

    #[test]
    fn process_and_verify_message_edge_case_hex_encoding() {
        // Test edge cases in hex encoding/decoding
        let context = b"hex edge case test";
        let mut message = create_test_message(42, context, "Hex edge case");
        
        // Test uppercase hex (should work)
        message.proof = message.proof.to_uppercase();
        assert!(process_and_verify_message(&message).is_ok());
        
        // Test mixed case hex (should work)
        let mut mixed_case = String::new();
        for (i, c) in message.proof.chars().enumerate() {
            if i % 2 == 0 {
                mixed_case.push(c.to_uppercase().next().unwrap());
            } else {
                mixed_case.push(c.to_lowercase().next().unwrap());
            }
        }
        message.proof = mixed_case;
        assert!(process_and_verify_message(&message).is_ok());
    }
}

// Property-based tests for comprehensive verification
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proof_messenger_protocol::key::generate_keypair_with_seed;
    use ed25519_dalek::Signer;

    proptest! {
        /// Property: Valid messages always verify successfully
        #[test]
        fn prop_valid_messages_always_verify(
            keypair_seed in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..1000),
            body in ".*"
        ) {
            let keypair = generate_keypair_with_seed(keypair_seed);
            let signature = keypair.sign(&context);
            
            let message = Message {
                sender: hex::encode(keypair.public.to_bytes()),
                context: hex::encode(&context),
                body,
                proof: hex::encode(signature.to_bytes()),
            };
            
            let result = process_and_verify_message(&message);
            prop_assert!(result.is_ok());
        }

        /// Property: Messages with wrong keys always fail
        #[test]
        fn prop_wrong_keys_always_fail(
            signing_seed in any::<u64>(),
            verifying_seed in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..1000),
            body in ".*"
        ) {
            prop_assume!(signing_seed != verifying_seed);
            
            let signing_keypair = generate_keypair_with_seed(signing_seed);
            let verifying_keypair = generate_keypair_with_seed(verifying_seed);
            let signature = signing_keypair.sign(&context);
            
            let message = Message {
                sender: hex::encode(verifying_keypair.public.to_bytes()), // Wrong key!
                context: hex::encode(&context),
                body,
                proof: hex::encode(signature.to_bytes()),
            };
            
            let result = process_and_verify_message(&message);
            prop_assert!(matches!(result, Err(AppError::VerificationFailed)));
        }

        /// Property: Tampered contexts always fail
        #[test]
        fn prop_tampered_contexts_always_fail(
            keypair_seed in any::<u64>(),
            original_context in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_index in any::<usize>(),
            tamper_value in any::<u8>(),
            body in ".*"
        ) {
            let keypair = generate_keypair_with_seed(keypair_seed);
            let signature = keypair.sign(&original_context);
            
            // Create tampered context
            let mut tampered_context = original_context.clone();
            if !tampered_context.is_empty() {
                let index = tamper_index % tampered_context.len();
                tampered_context[index] = tamper_value;
                
                // Only test if we actually changed something
                if tampered_context != original_context {
                    let message = Message {
                        sender: hex::encode(keypair.public.to_bytes()),
                        context: hex::encode(&tampered_context), // Tampered!
                        body,
                        proof: hex::encode(signature.to_bytes()),
                    };
                    
                    let result = process_and_verify_message(&message);
                    prop_assert!(matches!(result, Err(AppError::VerificationFailed)));
                }
            }
        }

        /// Property: Invalid hex always produces appropriate errors
        #[test]
        fn prop_invalid_hex_produces_errors(
            keypair_seed in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..100),
            body in ".*",
            invalid_hex in "[^0-9a-fA-F]+"
        ) {
            let keypair = generate_keypair_with_seed(keypair_seed);
            let signature = keypair.sign(&context);
            
            // Test invalid hex in sender
            let message1 = Message {
                sender: invalid_hex.clone(),
                context: hex::encode(&context),
                body: body.clone(),
                proof: hex::encode(signature.to_bytes()),
            };
            
            let result1 = process_and_verify_message(&message1);
            prop_assert!(matches!(result1, Err(AppError::InvalidPublicKey(_))));
            
            // Test invalid hex in context
            let message2 = Message {
                sender: hex::encode(keypair.public.to_bytes()),
                context: invalid_hex.clone(),
                body: body.clone(),
                proof: hex::encode(signature.to_bytes()),
            };
            
            let result2 = process_and_verify_message(&message2);
            prop_assert!(matches!(result2, Err(AppError::InvalidContext(_))));
            
            // Test invalid hex in proof
            let message3 = Message {
                sender: hex::encode(keypair.public.to_bytes()),
                context: hex::encode(&context),
                body,
                proof: invalid_hex,
            };
            
            let result3 = process_and_verify_message(&message3);
            prop_assert!(matches!(result3, Err(AppError::InvalidSignature(_))));
        }

        /// Property: Performance is consistent across different input sizes
        #[test]
        fn prop_performance_scales_reasonably(
            keypair_seed in any::<u64>(),
            context_size in 0usize..10000,
            body_size in 0usize..1000
        ) {
            let keypair = generate_keypair_with_seed(keypair_seed);
            let context = vec![0xAA; context_size];
            let body = "A".repeat(body_size);
            let signature = keypair.sign(&context);
            
            let message = Message {
                sender: hex::encode(keypair.public.to_bytes()),
                context: hex::encode(&context),
                body,
                proof: hex::encode(signature.to_bytes()),
            };
            
            let start = std::time::Instant::now();
            let result = process_and_verify_message(&message);
            let duration = start.elapsed();
            
            // Should always succeed for valid messages
            prop_assert!(result.is_ok());
            
            // Should complete in reasonable time (less than 100ms even for large inputs)
            prop_assert!(duration.as_millis() < 100, "Verification took too long: {:?}", duration);
        }
    }
}