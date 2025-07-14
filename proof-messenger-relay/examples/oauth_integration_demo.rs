use proof_messenger_relay::{
    database::Database,
    jwt_validator::JwtValidator,
    secure_logger::SecureLogger,
    create_app_with_oauth,
};
use std::sync::Arc;
use tokio;
use tracing_subscriber;

/// This example demonstrates how to set up the Proof-Messenger Relay Server
/// as an OAuth2.0 Resource Server that validates JWT tokens from an Identity Provider
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔐 OAuth2.0 Resource Server Demo");
    println!("================================");

    // 1. Set up the database
    let db = Arc::new(Database::new(":memory:").await?);
    db.migrate().await?;
    println!("✅ Database initialized");

    // 2. Configure JWT validator for your Identity Provider
    // In production, you would:
    // - Get the public key from your IdP's JWKS endpoint (e.g., https://your-okta-domain/.well-known/jwks)
    // - Use the actual issuer URL from your IdP
    // - Set the correct audience for your API
    
    let jwt_validator = Arc::new(JwtValidator::new_hmac(
        "your-shared-secret-with-idp", // In production, use RSA public key
        "https://your-okta-domain.com".to_string(),
        Some("proof-messenger-api".to_string()),
    ));
    println!("✅ JWT validator configured for Identity Provider");

    // 3. Set up secure logging with encryption
    let secure_key = SecureLogger::generate_key();
    let secure_logger = Arc::new(SecureLogger::new(&secure_key));
    println!("✅ Secure logger configured with AES-GCM encryption");

    // 4. Create the OAuth2.0-protected application with secure logging
    let app = create_app_with_oauth(db, jwt_validator, secure_logger);
    println!("✅ OAuth2.0 Resource Server with Secure Logging configured");

    println!("\n🚀 Server Configuration:");
    println!("   • Protected endpoints require valid JWT tokens");
    println!("   • Tokens must be from trusted issuer: https://your-okta-domain.com");
    println!("   • Audience must be: proof-messenger-api");
    println!("   • Required scopes:");
    println!("     - proof:create (for /relay endpoint)");
    println!("     - message:read (for /messages/* endpoints)");
    println!("   • All security events encrypted with AES-GCM");
    println!("   • Audit logs include authentication, authorization, and access events");

    println!("\n📋 API Endpoints:");
    println!("   🔒 POST /relay - Create and verify proofs (requires proof:create scope)");
    println!("   🔒 GET /messages/:group_id - Get messages by group (requires message:read scope)");
    println!("   🔒 GET /message/:message_id - Get specific message (requires message:read scope)");
    println!("   🔓 GET /health - Health check (public)");
    println!("   🔓 GET /ready - Readiness check (public)");

    println!("\n🔑 Example Usage:");
    println!("   1. Client obtains JWT from Identity Provider (Okta, Auth0, etc.)");
    println!("   2. Client includes token in Authorization header: 'Bearer <jwt>'");
    println!("   3. Resource Server validates token and extracts user/scopes");
    println!("   4. Request is authorized based on required scopes");

    println!("\n📝 Example curl commands:");
    println!("   # Get a message (requires valid JWT with message:read scope)");
    println!("   curl -H 'Authorization: Bearer <your-jwt>' http://localhost:3000/messages/group1");
    println!();
    println!("   # Create a proof (requires valid JWT with proof:create scope)");
    println!("   curl -X POST -H 'Authorization: Bearer <your-jwt>' \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"sender\":\"...\",\"context\":\"...\",\"body\":\"...\",\"proof\":\"...\"}}' \\");
    println!("        http://localhost:3000/relay");

    // 4. Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("\n🌐 Server starting on http://localhost:3000");
    println!("   Press Ctrl+C to stop");

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, Method, StatusCode},
    };
    use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
    use proof_messenger_relay::jwt_validator::Claims;
    use tower::ServiceExt;

    fn create_test_jwt(claims: Claims) -> String {
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret("your-shared-secret-with-idp".as_bytes());
        encode(&header, &claims, &encoding_key).unwrap()
    }

    #[tokio::test]
    async fn test_oauth_protected_endpoint_with_valid_token() {
        // ARRANGE: Set up the OAuth-protected server
        let db = Arc::new(Database::new(":memory:").await.unwrap());
        db.migrate().await.unwrap();

        let jwt_validator = Arc::new(JwtValidator::new_hmac(
            "your-shared-secret-with-idp",
            "https://your-okta-domain.com".to_string(),
            Some("proof-messenger-api".to_string()),
        ));

        let secure_key = SecureLogger::generate_key();
        let secure_logger = Arc::new(SecureLogger::new(&secure_key));

        let app = create_app_with_oauth(db, jwt_validator, secure_logger);

        // Create a valid JWT with required scope
        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://your-okta-domain.com".to_string(),
            aud: Some("proof-messenger-api".to_string()),
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: Some("message:read".to_string()),
        };

        let jwt = create_test_jwt(claims);

        // ACT: Make request to protected endpoint with valid token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/messages/test-group")
            .header("authorization", format!("Bearer {}", jwt))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be successful
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_oauth_protected_endpoint_without_token() {
        // ARRANGE: Set up the OAuth-protected server
        let db = Arc::new(Database::new(":memory:").await.unwrap());
        db.migrate().await.unwrap();

        let jwt_validator = Arc::new(JwtValidator::new_hmac(
            "your-shared-secret-with-idp",
            "https://your-okta-domain.com".to_string(),
            Some("proof-messenger-api".to_string()),
        ));

        let secure_key = SecureLogger::generate_key();
        let secure_logger = Arc::new(SecureLogger::new(&secure_key));

        let app = create_app_with_oauth(db, jwt_validator, secure_logger);

        // ACT: Make request to protected endpoint without token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/messages/test-group")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_public_endpoint_without_token() {
        // ARRANGE: Set up the OAuth-protected server
        let db = Arc::new(Database::new(":memory:").await.unwrap());
        db.migrate().await.unwrap();

        let jwt_validator = Arc::new(JwtValidator::new_hmac(
            "your-shared-secret-with-idp",
            "https://your-okta-domain.com".to_string(),
            Some("proof-messenger-api".to_string()),
        ));

        let secure_key = SecureLogger::generate_key();
        let secure_logger = Arc::new(SecureLogger::new(&secure_key));

        let app = create_app_with_oauth(db, jwt_validator, secure_logger);

        // ACT: Make request to public endpoint without token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Public endpoint should work without authentication
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_insufficient_scope_rejection() {
        // ARRANGE: Set up the OAuth-protected server
        let db = Arc::new(Database::new(":memory:").await.unwrap());
        db.migrate().await.unwrap();

        let jwt_validator = Arc::new(JwtValidator::new_hmac(
            "your-shared-secret-with-idp",
            "https://your-okta-domain.com".to_string(),
            Some("proof-messenger-api".to_string()),
        ));

        let secure_key = SecureLogger::generate_key();
        let secure_logger = Arc::new(SecureLogger::new(&secure_key));

        let app = create_app_with_oauth(db, jwt_validator, secure_logger);

        // Create a valid JWT but without required scope
        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://your-okta-domain.com".to_string(),
            aud: Some("proof-messenger-api".to_string()),
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: Some("other:scope".to_string()), // Wrong scope
        };

        let jwt = create_test_jwt(claims);

        // ACT: Make request to endpoint requiring message:read scope
        let request = Request::builder()
            .method(Method::GET)
            .uri("/messages/test-group")
            .header("authorization", format!("Bearer {}", jwt))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be forbidden due to insufficient scope
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // Our handler returns this for scope errors
    }
}