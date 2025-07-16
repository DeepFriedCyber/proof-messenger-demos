//! Proof Revocation Module
//!
//! This module provides functionality for managing the revocation of cryptographic proofs.
//! It implements a centralized Revocation List managed by the application backend.

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};
use chrono::{DateTime, Utc};

use crate::{database::Database, auth_middleware::AuthContext, AppError};

/// Request body for revoking a proof
#[derive(Serialize, Deserialize)]
pub struct RevokeProofRequest {
    /// The signature of the proof to revoke (hex encoded)
    pub proof_signature: String,
    /// Optional reason for revocation
    pub reason: Option<String>,
    /// Optional TTL in hours (default: 24 hours)
    pub ttl_hours: Option<i64>,
}

/// Response for revocation status
#[derive(Serialize, Deserialize)]
pub struct RevocationStatusResponse {
    /// Whether the proof is revoked
    pub is_revoked: bool,
    /// When the check was performed
    pub checked_at: DateTime<Utc>,
}

/// Create router for revocation endpoints
pub fn revocation_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/revoke", post(revoke_proof_handler))
        .route("/check/:signature", get(check_revocation_handler))
        .route("/list", get(list_revocations_handler))
        .route("/cleanup", post(cleanup_revocations_handler))
}

/// Create router for authenticated revocation endpoints
pub fn authenticated_revocation_routes() -> Router<(Arc<Database>, Arc<crate::jwt_validator::JwtValidator>, Arc<crate::secure_logger::SecureLogger>)> {
    Router::new()
        .route("/revoke", post(authenticated_revoke_proof_handler))
        .route("/check/:signature", get(authenticated_check_revocation_handler))
        .route("/list", get(authenticated_list_revocations_handler))
        .route("/cleanup", post(authenticated_cleanup_revocations_handler))
}

/// Handler to revoke a proof
#[instrument(skip_all)]
async fn revoke_proof_handler(
    State(db): State<Arc<Database>>,
    Json(payload): Json<RevokeProofRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Revoking proof: {}", payload.proof_signature);
    
    // Default TTL to 24 hours if not specified
    let ttl_hours = payload.ttl_hours.unwrap_or(24);
    
    db.revoke_proof(
        &payload.proof_signature,
        payload.reason.as_deref(),
        None, // No authenticated user in this context
        Some(ttl_hours),
    ).await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": "Proof revoked successfully",
        "proof_signature": payload.proof_signature,
        "ttl_hours": ttl_hours
    }));
    
    Ok((StatusCode::OK, response))
}

/// Handler to check if a proof is revoked
#[instrument(skip_all)]
async fn check_revocation_handler(
    State(db): State<Arc<Database>>,
    Path(signature): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Checking revocation status for proof: {}", signature);
    
    let is_revoked = db.is_proof_revoked(&signature).await?;
    
    let response = Json(RevocationStatusResponse {
        is_revoked,
        checked_at: Utc::now(),
    });
    
    Ok((StatusCode::OK, response))
}

/// Handler to list all active revocations
#[instrument(skip_all)]
async fn list_revocations_handler(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, AppError> {
    info!("Listing active revocations");
    
    let revocations = db.get_active_revocations().await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "count": revocations.len(),
        "revocations": revocations
    }));
    
    Ok((StatusCode::OK, response))
}

/// Handler to clean up expired revocations
#[instrument(skip_all)]
async fn cleanup_revocations_handler(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, AppError> {
    info!("Cleaning up expired revocations");
    
    let removed_count = db.cleanup_expired_revocations().await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": "Expired revocations cleaned up",
        "removed_count": removed_count
    }));
    
    Ok((StatusCode::OK, response))
}

/// Authenticated handler to revoke a proof
#[instrument(skip_all)]
async fn authenticated_revoke_proof_handler(
    State((db, _, secure_logger)): State<(Arc<Database>, Arc<crate::jwt_validator::JwtValidator>, Arc<crate::secure_logger::SecureLogger>)>,
    auth: AuthContext,
    Json(payload): Json<RevokeProofRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} revoking proof: {}", auth.user_id, payload.proof_signature);
    
    // Check if user has required scope for revoking proofs
    crate::auth_middleware::require_scope(&auth, "proof:revoke")
        .map_err(|_| AppError::ProcessingError("Insufficient permissions to revoke proofs".to_string()))?;
    
    // Default TTL to 24 hours if not specified
    let ttl_hours = payload.ttl_hours.unwrap_or(24);
    
    // Revoke the proof
    db.revoke_proof(
        &payload.proof_signature,
        payload.reason.as_deref(),
        Some(&auth.user_id),
        Some(ttl_hours),
    ).await?;
    
    // Log the revocation
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("proof_signature".to_string(), payload.proof_signature.clone());
    metadata.insert("ttl_hours".to_string(), ttl_hours.to_string());
    if let Some(reason) = &payload.reason {
        metadata.insert("reason".to_string(), reason.clone());
    }
    
    if let Err(e) = secure_logger.audit_log(
        "Proof revoked".to_string(),
        auth.user_id.clone(),
        None,
        metadata,
    ) {
        warn!("Failed to log proof revocation: {}", e);
    }
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": "Proof revoked successfully",
        "proof_signature": payload.proof_signature,
        "ttl_hours": ttl_hours,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
}

/// Authenticated handler to check if a proof is revoked
#[instrument(skip_all)]
async fn authenticated_check_revocation_handler(
    State((db, _, _)): State<(Arc<Database>, Arc<crate::jwt_validator::JwtValidator>, Arc<crate::secure_logger::SecureLogger>)>,
    auth: AuthContext,
    Path(signature): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} checking revocation status for proof: {}", auth.user_id, signature);
    
    // Check if user has required scope for checking revocations
    crate::auth_middleware::require_scope(&auth, "proof:read")
        .map_err(|_| AppError::ProcessingError("Insufficient permissions to check proof revocations".to_string()))?;
    
    let is_revoked = db.is_proof_revoked(&signature).await?;
    
    let response = Json(serde_json::json!({
        "is_revoked": is_revoked,
        "checked_at": Utc::now(),
        "proof_signature": signature,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
}

/// Authenticated handler to list all active revocations
#[instrument(skip_all)]
async fn authenticated_list_revocations_handler(
    State((db, _, _)): State<(Arc<Database>, Arc<crate::jwt_validator::JwtValidator>, Arc<crate::secure_logger::SecureLogger>)>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} listing active revocations", auth.user_id);
    
    // Check if user has required scope for listing revocations
    crate::auth_middleware::require_scope(&auth, "proof:read")
        .map_err(|_| AppError::ProcessingError("Insufficient permissions to list proof revocations".to_string()))?;
    
    let revocations = db.get_active_revocations().await?;
    
    let response = Json(serde_json::json!({
        "status": "success",
        "count": revocations.len(),
        "revocations": revocations,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
}

/// Authenticated handler to clean up expired revocations
#[instrument(skip_all)]
async fn authenticated_cleanup_revocations_handler(
    State((db, _, secure_logger)): State<(Arc<Database>, Arc<crate::jwt_validator::JwtValidator>, Arc<crate::secure_logger::SecureLogger>)>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    info!("Authenticated user {} cleaning up expired revocations", auth.user_id);
    
    // Check if user has required scope for managing revocations
    crate::auth_middleware::require_scope(&auth, "proof:manage")
        .map_err(|_| AppError::ProcessingError("Insufficient permissions to manage proof revocations".to_string()))?;
    
    let removed_count = db.cleanup_expired_revocations().await?;
    
    // Log the cleanup
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("removed_count".to_string(), removed_count.to_string());
    
    if let Err(e) = secure_logger.audit_log(
        "Expired proof revocations cleaned up".to_string(),
        auth.user_id.clone(),
        None,
        metadata,
    ) {
        warn!("Failed to log revocation cleanup: {}", e);
    }
    
    let response = Json(serde_json::json!({
        "status": "success",
        "message": "Expired revocations cleaned up",
        "removed_count": removed_count,
        "authenticated_user": auth.user_id
    }));
    
    Ok((StatusCode::OK, response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::body::Body;
    use tower::ServiceExt;
    use hyper::Method;
    
    async fn setup_test_app() -> Router {
        let db = Arc::new(crate::database::Database::new("sqlite::memory:").await.unwrap());
        db.migrate().await.unwrap();
        
        Router::new()
            .merge(revocation_routes())
            .with_state(db)
    }
    
    #[tokio::test]
    async fn test_revoke_and_check_proof() {
        // ARRANGE: Setup test app
        let app = setup_test_app().await;
        let proof_signature = "test_revoke_signature_123";
        
        // Create revocation request
        let revoke_request = RevokeProofRequest {
            proof_signature: proof_signature.to_string(),
            reason: Some("Test revocation".to_string()),
            ttl_hours: Some(24),
        };
        
        // ACT: Revoke the proof
        let revoke_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/revoke")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&revoke_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // ASSERT: Revocation should succeed
        assert_eq!(revoke_response.status(), StatusCode::OK);
        
        // ACT: Check if the proof is revoked
        let check_response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/check/{}", proof_signature))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // ASSERT: Check should confirm revocation
        assert_eq!(check_response.status(), StatusCode::OK);
        
        // Parse response body
        let body = axum::body::to_bytes(check_response.into_body(), usize::MAX).await.unwrap();
        let response: RevocationStatusResponse = serde_json::from_slice(&body).unwrap();
        
        assert!(response.is_revoked);
    }
}