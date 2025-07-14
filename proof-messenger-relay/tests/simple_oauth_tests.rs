use proof_messenger_relay::jwt_validator::{JwtValidator, JwtValidationError, Claims, extract_user_from_bearer_token};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

/// Helper function to create a test JWT token with HMAC
fn create_test_token_hmac(claims: Claims, secret: &str) -> String {
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &encoding_key).unwrap()
}

/// TDD Test Case 1: Valid JWT validation with HMAC (simpler for testing)
/// This test validates that our Resource Server can properly decode and validate
/// a JWT token that would be sent by a client application after OAuth2.0 authentication
#[test]
fn test_valid_jwt_validation_hmac() {
    // ARRANGE: Create a JWT validator as a Resource Server would (using HMAC for simplicity)
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create a realistic JWT payload as if it came from Okta
    let claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 9999999999, // Far future expiration
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: Some("read write".to_string()),
    };

    let valid_jwt = create_test_token_hmac(claims, "test-secret-key");

    // ACT: The Resource Server validates the token and extracts user ID
    let user_id = validator.validate_token(&valid_jwt).unwrap();

    // ASSERT: The user ID should be correctly extracted
    assert_eq!(user_id, "user-123");
}

/// TDD Test Case 2: Invalid signature JWT rejection with HMAC
/// This test ensures our Resource Server rejects tokens with invalid signatures
#[test]
fn test_invalid_signature_jwt_hmac() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create a token with wrong secret
    let claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: None,
    };

    let invalid_jwt = create_test_token_hmac(claims, "wrong-secret-key");

    // ACT & ASSERT: The validator should reject the invalid token
    let result = validator.validate_token(&invalid_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidSignature)));
}

/// TDD Test Case 3: Expired token rejection
/// This test ensures our Resource Server rejects expired tokens
#[test]
fn test_expired_token_rejection_hmac() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create an expired token
    let expired_claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 1000000000, // Past timestamp (expired)
        iat: Some(999999999),
        nbf: Some(999999999),
        scope: None,
    };

    let expired_jwt = create_test_token_hmac(expired_claims, "test-secret-key");

    // ACT & ASSERT: The validator should reject the expired token
    let result = validator.validate_token(&expired_jwt);
    assert!(matches!(result, Err(JwtValidationError::Expired)));
}

/// TDD Test Case 4: Invalid issuer rejection
/// This test ensures our Resource Server only accepts tokens from trusted issuers
#[test]
fn test_invalid_issuer_rejection_hmac() {
    // ARRANGE: Create a JWT validator expecting tokens from Okta
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create a token from a malicious issuer
    let malicious_claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://malicious.com".to_string(), // Wrong issuer
        aud: None,
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: None,
    };

    let malicious_jwt = create_test_token_hmac(malicious_claims, "test-secret-key");

    // ACT & ASSERT: The validator should reject tokens from untrusted issuers
    let result = validator.validate_token(&malicious_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidIssuer)));
}

/// TDD Test Case 5: Scope extraction for authorization
/// This test validates that our Resource Server can extract OAuth2.0 scopes for authorization
#[test]
fn test_scope_extraction_for_authorization_hmac() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create a token with specific scopes
    let claims_with_scopes = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: Some("read write admin".to_string()),
    };

    let jwt_with_scopes = create_test_token_hmac(claims_with_scopes, "test-secret-key");

    // ACT: Extract scopes for authorization decisions
    let scopes = validator.extract_scopes(&jwt_with_scopes).unwrap();

    // ASSERT: All scopes should be correctly extracted
    assert!(scopes.contains("read"));
    assert!(scopes.contains("write"));
    assert!(scopes.contains("admin"));
    assert_eq!(scopes.len(), 3);
}

/// TDD Test Case 6: Bearer token extraction from Authorization header
/// This test validates the complete flow of extracting and validating JWT from HTTP headers
#[test]
fn test_bearer_token_extraction_from_header_hmac() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // Create a valid token
    let claims = Claims {
        sub: "user-456".to_string(),
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: Some("read".to_string()),
    };

    let jwt = create_test_token_hmac(claims, "test-secret-key");
    let auth_header = format!("Bearer {}", jwt);

    // ACT: Extract user ID from Authorization header (as would happen in middleware)
    let user_id = extract_user_from_bearer_token(&auth_header, &validator).unwrap();

    // ASSERT: User ID should be correctly extracted
    assert_eq!(user_id, "user-456");
}

/// TDD Test Case 7: Invalid Bearer format rejection
/// This test ensures malformed Authorization headers are rejected
#[test]
fn test_invalid_bearer_format_rejection_hmac() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );

    // ACT & ASSERT: Invalid header formats should be rejected
    let result = extract_user_from_bearer_token("Invalid token", &validator);
    assert!(matches!(result, Err(JwtValidationError::InvalidFormat)));

    let result = extract_user_from_bearer_token("Basic dXNlcjpwYXNz", &validator);
    assert!(matches!(result, Err(JwtValidationError::InvalidFormat)));
}

/// TDD Test Case 8: Audience validation
/// This test ensures our Resource Server validates the audience claim when configured
#[test]
fn test_audience_validation_hmac() {
    // ARRANGE: Create a JWT validator that expects a specific audience
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        Some("proof-messenger-api".to_string()),
    );

    // Create a token with correct audience
    let correct_claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: Some("proof-messenger-api".to_string()),
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: None,
    };

    let correct_jwt = create_test_token_hmac(correct_claims, "test-secret-key");

    // ACT & ASSERT: Valid audience should be accepted
    let user_id = validator.validate_token(&correct_jwt).unwrap();
    assert_eq!(user_id, "user-123");

    // Create a token with wrong audience
    let wrong_claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        aud: Some("wrong-api".to_string()),
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: None,
    };

    let wrong_jwt = create_test_token_hmac(wrong_claims, "test-secret-key");

    // ACT & ASSERT: Wrong audience should be rejected
    let result = validator.validate_token(&wrong_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidAudience)));
}

/// Integration Test: Complete OAuth2.0 Resource Server flow
/// This test simulates the complete flow of a Resource Server validating an OAuth2.0 token
#[test]
fn test_complete_oauth_resource_server_flow_hmac() {
    // ARRANGE: Set up the Resource Server (our Proof-Messenger Relay)
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        Some("proof-messenger-api".to_string()),
    );

    // Simulate a client application that has already obtained a token from Okta
    let client_token_claims = Claims {
        sub: "enterprise-user-789".to_string(),
        iss: "https://okta.com".to_string(),
        aud: Some("proof-messenger-api".to_string()),
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: Some("proof:create proof:verify message:read".to_string()),
    };

    let client_jwt = create_test_token_hmac(client_token_claims, "test-secret-key");
    let authorization_header = format!("Bearer {}", client_jwt);

    // ACT: Resource Server validates the token (this would happen in middleware)
    let user_id = extract_user_from_bearer_token(&authorization_header, &validator).unwrap();
    let scopes = validator.extract_scopes(&client_jwt).unwrap();

    // ASSERT: Resource Server should successfully authenticate and authorize the request
    assert_eq!(user_id, "enterprise-user-789");
    assert!(scopes.contains("proof:create"));
    assert!(scopes.contains("proof:verify"));
    assert!(scopes.contains("message:read"));

    // Additional authorization logic could check if user has required scopes
    let has_create_permission = scopes.contains("proof:create");
    let has_verify_permission = scopes.contains("proof:verify");
    
    assert!(has_create_permission);
    assert!(has_verify_permission);
}