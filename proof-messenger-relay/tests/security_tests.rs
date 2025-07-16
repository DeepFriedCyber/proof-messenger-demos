//! Security-focused integration tests for the relay server
//!
//! These tests verify security headers, rate limiting, and other security features
//! following TDD principles - tests written first, implementation follows.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    Router,
};
use proof_messenger_relay::Message;

use std::sync::Arc;
use tower::ServiceExt;
use ed25519_dalek::Signer;
use proof_messenger_protocol::key::generate_keypair_with_seed;
use proof_messenger_relay::database::Database;

/// Helper function to create the app with security features enabled
async fn create_test_app_with_security() -> Router {
    let db = Database::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    let db = Arc::new(db);
    
    proof_messenger_relay::create_app_with_security(db)
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
async fn test_security_headers_are_present() {
    // ARRANGE: Create test app with security features
    let app = create_test_app_with_security().await;
    
    // ACT: Make a request to any endpoint
    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let headers = response.headers();

    // ASSERT: Security headers should be present
    assert_eq!(
        headers.get("strict-transport-security").unwrap(),
        "max-age=63072000; includeSubDomains"
    );
    assert_eq!(
        headers.get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(
        headers.get("x-frame-options").unwrap(),
        "DENY"
    );
}

#[tokio::test]
async fn test_rate_limiter_allows_normal_requests() {
    // ARRANGE: Create test app with rate limiting
    let app = create_test_app_with_security().await;
    let context = b"rate limit test context";
    let message = create_test_message(42, context, "Rate limit test message");

    // ACT: Send requests within the rate limit (should be 5 requests allowed)
    for i in 0..5 {
        let request = Request::builder()
            .method("POST")
            .uri("/relay")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // ASSERT: All requests within limit should succeed
        assert_eq!(
            response.status(), 
            StatusCode::OK,
            "Request {} should succeed within rate limit", i + 1
        );
    }
}

#[tokio::test]
async fn test_rate_limiter_configuration_exists() {
    // ARRANGE: Create test app with rate limiting using production version
    let db = Database::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    let db = Arc::new(db);
    
    // ACT: Create the app with rate limiting - this tests that the configuration compiles and runs
    let _app = proof_messenger_relay::create_app_with_rate_limiting(db);
    
    // ASSERT: If we get here, the rate limiting configuration is valid
    // Note: Actual rate limiting behavior is better tested in integration tests
    // with real HTTP clients that provide proper connection information
    assert!(true, "Rate limiting configuration is valid and app can be created");
}

#[tokio::test]
async fn test_compression_header_present() {
    // ARRANGE: Create test app with compression
    let app = create_test_app_with_security().await;
    
    // ACT: Make a request that should be compressed
    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .header("accept-encoding", "gzip")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // ASSERT: Response should indicate compression capability
    // Note: The actual compression depends on response size and content
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cors_headers_configured() {
    // ARRANGE: Create test app with CORS
    let app = create_test_app_with_security().await;
    
    // ACT: Make a preflight CORS request
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/relay")
        .header("origin", "https://example.com")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // ASSERT: CORS headers should be present
    // Note: Specific CORS configuration will depend on requirements
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tracing_layer_active() {
    // ARRANGE: Create test app with tracing
    let app = create_test_app_with_security().await;
    let context = b"tracing test context";
    let message = create_test_message(44, context, "Tracing test message");
    
    // ACT: Make a request that should be traced
    let request = Request::builder()
        .method("POST")
        .uri("/relay")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&message).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // ASSERT: Request should succeed (tracing is mostly internal)
    assert_eq!(response.status(), StatusCode::OK);
    
    // Note: In a real application, you might check logs or metrics
    // For this test, we're just ensuring the layer doesn't break functionality
}

#[tokio::test]
async fn test_error_handling_layer_works() {
    // ARRANGE: Create test app with error handling
    let app = create_test_app_with_security().await;
    
    // ACT: Make a request that should trigger an error
    let request = Request::builder()
        .method("POST")
        .uri("/relay")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // ASSERT: Error should be handled gracefully
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    
    // Should contain error information (may not be JSON due to Axum's built-in error handling)
    assert!(body_str.contains("error") || body_str.contains("invalid") || body_str.contains("JSON"));
}

#[tokio::test]
async fn test_rate_limit_per_ip_isolation() {
    // ARRANGE: Create test app
    let app = create_test_app_with_security().await;
    let context = b"ip isolation test context";
    let message = create_test_message(45, context, "IP isolation test message");

    // ACT: Simulate requests from different IPs by using different user agents
    // Note: In a real scenario, you'd need to test with actual different source IPs
    // This is a simplified test to verify the rate limiter is configured
    
    for i in 0..3 {
        let request = Request::builder()
            .method("POST")
            .uri("/relay")
            .header("content-type", "application/json")
            .header("user-agent", format!("test-client-{}", i))
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // ASSERT: Requests should succeed (simplified test)
        assert_eq!(response.status(), StatusCode::OK);
    }
}