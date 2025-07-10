//! Integration tests for the relay server

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::body::to_bytes;
use proof_messenger_relay::{config::Config, relay::RelayState};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let config = Config::default();
    let state = Arc::new(RelayState::new(config));
    let app = proof_messenger_relay::create_app(state, false).await.unwrap();
    
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body()).await.unwrap();
    let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(health_response["status"], "healthy");
    assert!(health_response["version"].is_string());
}

#[tokio::test]
async fn test_status_endpoint() {
    let config = Config::default();
    let state = Arc::new(RelayState::new(config));
    let app = proof_messenger_relay::create_app(state, false).await.unwrap();
    
    let request = Request::builder()
        .uri("/status")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body()).await.unwrap();
    let status_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(status_response["server"], "proof-messenger-relay");
    assert_eq!(status_response["connections_active"], 0);
    assert_eq!(status_response["messages_relayed"], 0);
}

#[tokio::test]
async fn test_verify_endpoint_invalid_message() {
    let config = Config::default();
    let state = Arc::new(RelayState::new(config));
    let app = proof_messenger_relay::create_app(state, false).await.unwrap();
    
    let invalid_message = json!({
        "message": {
            "id": "test-123",
            "sender": "invalid-key",
            "recipient": "also-invalid",
            "content": "Hello",
            "signature": "invalid-signature"
        }
    });
    
    let request = Request::builder()
        .uri("/verify")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_message.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body()).await.unwrap();
    let verify_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(verify_response["valid"], false);
    assert!(verify_response["errors"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_stats_endpoint() {
    let config = Config::default();
    let state = Arc::new(RelayState::new(config));
    let app = proof_messenger_relay::create_app(state, false).await.unwrap();
    
    let request = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body()).await.unwrap();
    let stats_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(stats_response["connections_total"], 0);
    assert_eq!(stats_response["connections_active"], 0);
    assert_eq!(stats_response["messages_relayed"], 0);
    assert_eq!(stats_response["messages_verified"], 0);
    assert_eq!(stats_response["messages_rejected"], 0);
    assert_eq!(stats_response["proofs_verified"], 0);
}

#[tokio::test]
async fn test_relay_state_operations() {
    let config = Config::default();
    let state = RelayState::new(config);
    
    // Test initial stats
    let stats = state.get_stats().await;
    assert_eq!(stats.connections_total, 0);
    assert_eq!(stats.connections_active, 0);
    
    // Test adding connection
    let addr = "127.0.0.1:12345".parse().unwrap();
    state.add_connection(addr).await;
    
    let stats = state.get_stats().await;
    assert_eq!(stats.connections_total, 1);
    assert_eq!(stats.connections_active, 1);
    
    // Test removing connection
    state.remove_connection(addr).await;
    
    let stats = state.get_stats().await;
    assert_eq!(stats.connections_total, 1);
    assert_eq!(stats.connections_active, 0);
    
    // Test incrementing counters
    state.increment_messages_relayed().await;
    state.increment_messages_verified().await;
    state.increment_messages_rejected().await;
    state.increment_proofs_verified(5).await;
    
    let stats = state.get_stats().await;
    assert_eq!(stats.messages_relayed, 1);
    assert_eq!(stats.messages_verified, 1);
    assert_eq!(stats.messages_rejected, 1);
    assert_eq!(stats.proofs_verified, 5);
}

#[tokio::test]
async fn test_config_validation() {
    let mut config = Config::default();
    
    // Valid config should pass
    assert!(config.validate().is_ok());
    
    // Invalid port should fail
    config.server.port = 0;
    assert!(config.validate().is_err());
    
    // Reset and test other validations
    config = Config::default();
    config.server.max_connections = 0;
    assert!(config.validate().is_err());
    
    config = Config::default();
    config.security.max_content_length = 0;
    assert!(config.validate().is_err());
    
    config = Config::default();
    config.logging.level = "invalid".to_string();
    assert!(config.validate().is_err());
}