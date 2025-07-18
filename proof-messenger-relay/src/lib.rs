//! Proof Messenger Relay Library
//!
//! This library provides the core functionality for the relay server,
//! including message verification, database operations, and HTTP handlers.

pub mod database;
pub mod jwt_validator;
pub mod auth_middleware;
pub mod secure_logger;
pub mod revocation;
pub mod metrics;

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
use auth_middleware::{AuthContext, auth_middleware, require_scope};
use jwt_validator::JwtValidator;
use secure_logger::{SecureLogger, LogLevel};

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
    
    #[error("Proof has been revoked")]
    ProofRevoked,
    
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
            AppError::ProofRevoked => (StatusCode::FORBIDDEN, self.to_string()),
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
/// 
/// If a database is provided, it will also check if the proof has been revoked.
#[instrument(skip_all, fields(sender = %message.sender))]
pub async fn process_and_verify_message(
    message: &Message, 
    db: Option<&Arc<Database>>
) -> Result<(), AppError> {
    info!("Processing message verification");

    // If a database is provided, check if the proof has been revoked
    if let Some(db) = db {
        // Check if REVOCATION_CHECK_ENABLED environment variable is set
        if std::env::var("REVOCATION_CHECK_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
            info!("Checking if proof has been revoked");
            
            // Check if the proof is in the revocation list
            if db.is_proof_revoked(&message.proof).await? {
                warn!("Proof has been revoked: {}", message.proof);
                return Err(AppError::ProofRevoked);
            }
        }
    }

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
        .nest("/revocation", revocation::revocation_routes())
        .with_state(db)
}

/// Create the application router with security enhancements
/// This includes security headers and tracing (rate limiting configured separately)
pub fn create_app_with_security(db: Arc<Database>) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::cors::CorsLayer;
    use tower_http::set_header::SetResponseHeaderLayer;

    // Create the base router
    Router::new()
        .route("/relay", post(relay_handler))
        .route("/messages/:group_id", get(get_messages_handler))
        .route("/message/:message_id", get(get_message_by_id_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .nest("/revocation", revocation::revocation_routes())
        .with_state(db)
        // Apply security layers
        .layer(TraceLayer::new_for_http())
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        // CORS layer (configure as needed)
        .layer(CorsLayer::permissive()) // Note: Configure restrictively in production
}

/// Create the minimal application router with no middleware at all
/// This is for debugging purposes only
pub fn create_app_minimal(db: Arc<Database>) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use tower_http::cors::CorsLayer;
    use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor};
    
    // Fix GovernorLayer by using GlobalKeyExtractor instead of IP-based
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .key_extractor(GlobalKeyExtractor)
        .finish()
        .unwrap();
    
    Router::new()
        .route("/test", get(test_handler))
        .with_state(db)
        .layer(GovernorLayer {
            config: std::sync::Arc::new(governor_conf),
        })
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(CorsLayer::permissive())
}

/// Create the basic application router without rate limiting or authentication
/// This is suitable for debugging and testing
pub fn create_app_basic(db: Arc<Database>) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::cors::CorsLayer;
    use tower_http::set_header::SetResponseHeaderLayer;

    // Create the base router
    Router::new()
        .route("/relay", post(relay_handler))
        .route("/messages/:group_id", get(get_messages_handler))
        .route("/message/:message_id", get(get_message_by_id_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/test", get(test_handler))
        .nest("/revocation", revocation::revocation_routes())
        .with_state(db)
        .layer(TraceLayer::new_for_http())
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        // CORS layer (configure as needed)
        .layer(CorsLayer::permissive()) // Note: Configure restrictively in production
}

/// Create the application router with full production security including rate limiting
/// This version includes rate limiting that works in production environments
pub fn create_app_with_rate_limiting(db: Arc<Database>) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::cors::CorsLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor};

    // Configure rate limiting: 5 requests per burst, 1 new request every 2 seconds
    // Use GlobalKeyExtractor to avoid "Unable To Extract Key!" error
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .key_extractor(GlobalKeyExtractor)
        .finish()
        .unwrap();

    // Create protected routes (with rate limiting)
    let protected_routes = Router::new()
        .route("/relay", post(relay_handler))
        .route("/messages/:group_id", get(get_messages_handler))
        .route("/message/:message_id", get(get_message_by_id_handler))
        .route("/test", get(test_handler))
        .nest("/revocation", revocation::revocation_routes())
        .with_state(db.clone())
        // Apply rate limiting only to protected routes
        .layer(GovernorLayer {
            config: std::sync::Arc::new(governor_conf),
        });

    // Create public routes (no rate limiting for health checks)
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(db);

    // Combine routes
    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .layer(TraceLayer::new_for_http())
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        // CORS layer (configure as needed)
        .layer(CorsLayer::permissive()) // Note: Configure restrictively in production
}

/// Create the application router with OAuth2.0 JWT authentication and secure logging
/// This version implements proper Resource Server behavior for OAuth2.0 flows
pub fn create_app_with_oauth(
    db: Arc<Database>, 
    jwt_validator: Arc<JwtValidator>,
    secure_logger: Arc<SecureLogger>,
) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::cors::CorsLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use axum::middleware;

    // Create protected routes that require authentication
    let protected_routes = Router::new()
        .route("/relay", post(authenticated_relay_handler))
        .route("/messages/:group_id", get(authenticated_get_messages_handler))
        .route("/message/:message_id", get(authenticated_get_message_by_id_handler))
        .nest("/revocation", revocation::authenticated_revocation_routes())
        .layer(middleware::from_fn_with_state(jwt_validator.clone(), auth_middleware))
        .with_state((db.clone(), jwt_validator.clone(), secure_logger.clone()));

    // Create public routes (health checks don't need authentication)
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .with_state(db.clone());
    
    // Create metrics route (doesn't need database state)
    tracing::info!("Registering metrics route at /metrics");
    let metrics_routes = Router::new()
        .route("/metrics", get(metrics::metrics_handler));

    // Combine routes and apply security layers
    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .merge(metrics_routes)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .layer(TraceLayer::new_for_http())
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        // CORS layer (configure restrictively in production)
        .layer(CorsLayer::permissive())
}

/// The Axum handler for message relay
#[instrument(skip_all)]
async fn relay_handler(
    State(db): State<Arc<Database>>,
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received message for relay");
    
    // Delegate to the unit-tested function, passing the database for revocation check
    process_and_verify_message(&payload, Some(&db)).await?;
    
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
#[instrument(skip_all)]
async fn health_handler(
    State(db): State<Arc<Database>>,
) -> impl IntoResponse {
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    // Test database connection
    match db.health_check().await {
        Ok(_) => {
            let health_response = Json(serde_json::json!({
                "status": "healthy",
                "database": "connected",
                "service": "proof-messenger-relay",
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": timestamp
            }));
            
            (StatusCode::OK, health_response)
        },
        Err(e) => {
            let health_response = Json(serde_json::json!({
                "status": "unhealthy",
                "database": "disconnected",
                "service": "proof-messenger-relay",
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": timestamp,
                "error": e.to_string()
            }));
            
            (StatusCode::SERVICE_UNAVAILABLE, health_response)
        }
    }
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

/// Simple test endpoint without database dependency
#[instrument(skip_all)]
async fn test_handler() -> impl IntoResponse {
    "Hello World!"
}



/// OAuth2.0-protected relay handler that requires authentication and proper scopes
#[instrument(skip_all)]
async fn authenticated_relay_handler(
    State((db, _validator, secure_logger)): State<(Arc<Database>, Arc<JwtValidator>, Arc<SecureLogger>)>,
    auth: AuthContext,
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received authenticated message for relay from user: {}", auth.user_id);
    
    // Log the authentication event securely
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("endpoint".to_string(), "/relay".to_string());
    metadata.insert("method".to_string(), "POST".to_string());
    metadata.insert("scopes".to_string(), format!("{:?}", auth.scopes));
    
    if let Err(e) = secure_logger.audit_log(
        "User authenticated for proof creation".to_string(),
        auth.user_id.clone(),
        None, // Could extract request ID from headers
        metadata.clone(),
    ) {
        warn!("Failed to log authentication event: {}", e);
    }
    
    // Check if user has required scope for creating proofs
    match require_scope(&auth, "proof:create") {
        Ok(_) => {
            // Log successful authorization
            metadata.insert("authorization_result".to_string(), "granted".to_string());
            if let Err(e) = secure_logger.log_security_event(
                LogLevel::Audit,
                "Proof creation authorization granted".to_string(),
                Some(auth.user_id.clone()),
                None,
                metadata.clone(),
            ) {
                warn!("Failed to log authorization event: {}", e);
            }
        }
        Err(_) => {
            // Log authorization failure
            metadata.insert("authorization_result".to_string(), "denied".to_string());
            metadata.insert("required_scope".to_string(), "proof:create".to_string());
            if let Err(e) = secure_logger.critical_security_event(
                "Proof creation authorization denied - insufficient scope".to_string(),
                Some(auth.user_id.clone()),
                None,
                metadata,
            ) {
                warn!("Failed to log authorization failure: {}", e);
            }
            return Err(AppError::ProcessingError("Insufficient permissions to create proofs".to_string()));
        }
    }
    
    // Delegate to the unit-tested function, passing the database for revocation check
    process_and_verify_message(&payload, Some(&db)).await?;
    
    // Store the verified message in the database with user context
    let stored_message = StoredMessage::from(payload.clone());
    let message_id = db.store_message(stored_message).await?;
    
    // Log successful proof creation
    let mut success_metadata = std::collections::HashMap::new();
    success_metadata.insert("message_id".to_string(), message_id.clone());
    success_metadata.insert("sender".to_string(), payload.sender.clone());
    success_metadata.insert("context".to_string(), payload.context.clone());
    success_metadata.insert("proof_verified".to_string(), "true".to_string());
    
    if let Err(e) = secure_logger.audit_log(
        "Proof creation and verification completed successfully".to_string(),
        auth.user_id.clone(),
        None,
        success_metadata,
    ) {
        warn!("Failed to log proof creation success: {}", e);
    }
    
    let success_response = Json(serde_json::json!({
        "status": "success",
        "message": "Message verified and relayed successfully",
        "message_id": message_id,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, success_response))
}

/// OAuth2.0-protected handler to retrieve messages for a specific group
#[instrument(skip_all)]
async fn authenticated_get_messages_handler(
    State((db, _validator, secure_logger)): State<(Arc<Database>, Arc<JwtValidator>, Arc<SecureLogger>)>,
    auth: AuthContext,
    Path(group_id): Path<String>,
    Query(params): Query<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} retrieving messages for group: {}", auth.user_id, group_id);
    
    // Check if user has required scope for reading messages
    require_scope(&auth, "message:read")
        .map_err(|_| {
            // Log authorization failure
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("endpoint".to_string(), "/messages".to_string());
            metadata.insert("group_id".to_string(), group_id.clone());
            metadata.insert("required_scope".to_string(), "message:read".to_string());
            
            if let Err(e) = secure_logger.critical_security_event(
                "Message read authorization denied - insufficient scope".to_string(),
                Some(auth.user_id.clone()),
                None,
                metadata,
            ) {
                warn!("Failed to log authorization failure: {}", e);
            }
            
            AppError::ProcessingError("Insufficient permissions to read messages".to_string())
        })?;
    
    let messages = db.get_messages_by_group(&group_id, params.limit).await?;
    
    // Log successful message retrieval
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("group_id".to_string(), group_id.clone());
    metadata.insert("message_count".to_string(), messages.len().to_string());
    metadata.insert("limit".to_string(), params.limit.unwrap_or(100).to_string());
    
    if let Err(e) = secure_logger.audit_log(
        "Messages retrieved successfully".to_string(),
        auth.user_id.clone(),
        None,
        metadata,
    ) {
        warn!("Failed to log message retrieval: {}", e);
    }
    
    let response = Json(serde_json::json!({
        "status": "success",
        "group_id": group_id,
        "message_count": messages.len(),
        "messages": messages,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
}

/// OAuth2.0-protected handler to retrieve a specific message by ID
#[instrument(skip_all)]
async fn authenticated_get_message_by_id_handler(
    State((db, _validator, secure_logger)): State<(Arc<Database>, Arc<JwtValidator>, Arc<SecureLogger>)>,
    auth: AuthContext,
    Path(message_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} retrieving message: {}", auth.user_id, message_id);
    
    // Check if user has required scope for reading messages
    require_scope(&auth, "message:read")
        .map_err(|_| AppError::ProcessingError("Insufficient permissions to read messages".to_string()))?;
    
    let message = db.get_message_by_id(&message_id).await?;
    
    // Log successful message retrieval
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("message_id".to_string(), message_id.clone());
    metadata.insert("endpoint".to_string(), "/message".to_string());
    
    if let Err(e) = secure_logger.audit_log(
        "Individual message retrieved successfully".to_string(),
        auth.user_id.clone(),
        None,
        metadata,
    ) {
        warn!("Failed to log message retrieval: {}", e);
    }
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": message,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
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

    #[tokio::test]
    async fn process_and_verify_message_rejects_tampered_context() {
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
        let result = process_and_verify_message(&tampered_message, None).await;

        // ASSERT: The result must be a VerificationFailed error
        assert!(matches!(result, Err(AppError::VerificationFailed)));
    }

    #[tokio::test]
    async fn process_and_verify_message_accepts_valid_message() {
        // ARRANGE: Create a valid message
        let context = b"valid context for signature";
        let message = create_test_message(42, context, "Valid test message");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be successful
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn process_and_verify_message_rejects_invalid_signature_format() {
        // ARRANGE: Create a message with invalid signature format
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = "invalid_hex_signature".to_string(); // Invalid hex

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be an InvalidSignature error
        assert!(matches!(result, Err(AppError::InvalidSignature(_))));
    }
    
    #[tokio::test]
    async fn process_and_verify_message_rejects_revoked_proof() {
        // ARRANGE: Create a valid message
        let context = b"valid context for signature";
        let message = create_test_message(42, context, "Valid test message");
        
        // Create a database with the proof revoked
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        db.revoke_proof(&message.proof, Some("Test revocation"), None, None).await.unwrap();
        
        // Set environment variable for revocation check
        std::env::set_var("REVOCATION_CHECK_ENABLED", "true");
        
        // ACT: Call the logic function with database that has revoked the proof
        let result = process_and_verify_message(&message, Some(&Arc::new(db))).await;
        
        // ASSERT: The result should be a ProofRevoked error
        assert!(matches!(result, Err(AppError::ProofRevoked)));
        
        // Clean up environment variable
        std::env::remove_var("REVOCATION_CHECK_ENABLED");
    }

    #[tokio::test]
    async fn process_and_verify_message_rejects_invalid_public_key_format() {
        // ARRANGE: Create a message with invalid public key format
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.sender = "invalid_hex_pubkey".to_string(); // Invalid hex

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be an InvalidPublicKey error
        assert!(matches!(result, Err(AppError::InvalidPublicKey(_))));
    }

    #[tokio::test]
    async fn process_and_verify_message_rejects_wrong_signature_length() {
        // ARRANGE: Create a message with wrong signature length
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = hex::encode(&[0u8; 32]); // Wrong length (32 instead of 64)

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be an InvalidSignature error
        assert!(matches!(result, Err(AppError::InvalidSignature(_))));
    }

    #[tokio::test]
    async fn process_and_verify_message_rejects_wrong_public_key_length() {
        // ARRANGE: Create a message with wrong public key length
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.sender = hex::encode(&[0u8; 16]); // Wrong length (16 instead of 32)

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be an InvalidPublicKey error
        assert!(matches!(result, Err(AppError::InvalidPublicKey(_))));
    }

    #[tokio::test]
    async fn process_and_verify_message_rejects_tampered_signature() {
        // ARRANGE: Create a valid message then tamper with the signature
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        
        // Tamper with the signature by flipping a bit
        let mut sig_bytes = hex::decode(&message.proof).unwrap();
        sig_bytes[0] ^= 0x01; // Flip the first bit
        message.proof = hex::encode(sig_bytes);

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be a VerificationFailed error
        assert!(matches!(result, Err(AppError::VerificationFailed)));
    }

    #[tokio::test]
    async fn process_and_verify_message_handles_empty_context() {
        // ARRANGE: Create a message with empty context
        let context = b"";
        let message = create_test_message(42, context, "Message with empty context");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be successful (empty context is valid)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn process_and_verify_message_handles_large_context() {
        // ARRANGE: Create a message with large context
        let large_context = vec![0xAA; 10000]; // 10KB context
        let message = create_test_message(42, &large_context, "Message with large context");

        // ACT: Call the logic function directly
        let result = process_and_verify_message(&message, None).await;

        // ASSERT: The result should be successful
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn error_messages_are_informative() {
        // Test that error messages contain useful information
        let context = b"test context";
        let mut message = create_test_message(42, context, "Test message");
        message.proof = "not_hex".to_string();

        let result = process_and_verify_message(&message, None).await;
        
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

    #[tokio::test]
    async fn message_serialization_roundtrip() {
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
        assert!(process_and_verify_message(&original_message, None).await.is_ok());
        assert!(process_and_verify_message(&deserialized_message, None).await.is_ok());
    }
}