use proof_messenger_relay::jwt_validator::{JwtValidator, JwtValidationError, Claims, extract_user_from_bearer_token};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

// Mock keys for testing - in production these would come from your Identity Provider
const MOCK_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7VJTUt9Us8cKB
wEiOfQIL3/C7/Y/iw/VuEiEnHl/YhE4FzydLOPsGpVtjwSUFbXTVfWfHXiQNNb6N
6oTF5klHCUKtAHxMTDzem2uiwUoukHkfLjpoHh9OkNvFQY2VfkqHXqEi0J2obsI+
7wgBjghSeXyOCI3/pSoGt4SBBBXjPzfQ5QDiZeKLHs6i6Ti2HfdGlD5n5FcXBjXZ
8MjMXAuVH7q5nGhxvfMUXjo5foQbL8ku5pn4AlgGtjGQuU9vDCEi7SeR7g/veqXg
M1VcCAlOVEWXe4mXBBbLzjlDKdJEOfFb/2kSWHmtjcH3/lwGa4n2UBXicJlbkVsM
7QjD9Aj1AgMBAAECggEBALc2lQAFVVTiEfjKXNd97UD8xvqpRkdGjQG95uEjB7/e
UIhyDxGKw+2OVcDcmBh5VxQzBJZtnncgOcJ8OcEiUziEjKwbQWXts3Y5CE6g3S+b
kHNJlTEa8C+fXEZtwkTtxVV0isAk8J+5Fw5UuV4fMv6dmqQp/grfn7zsmHmEhVcs
D0EkcFVs+VSwVK/0xSxAQGQzgvzfzlQeucyS2CFYqz0fTpFVejXdHFx8d0ZcSI0e
q9H4NdvYeE4B+jOCt/IrHBSoSMFuMz/oIlloQxmDsLS7J+wzJjCpTm+GNyfI2h+4
MRBYv/dK3DIjSMqmzrMzpd3zzFXmh2GvsUoDAQOdjZECgYEA4ckksVCl755qlnH5
PDhEpNn3ZaKjruuoiROakmb6OiHzkXjjGoK8TpHOHDJNpLyMlctTrTfHVcvMzPjV
3ddnOTbqbdHcmhyI6Hz2iq4uBdXfHRa9b/RBjskZjTTiJVtZvz1IrHMrMa/Od/jR
MqvRxbO8OuAoNpY1hzBRsXml3I0CgYEA1DkYmvbr5gjJ3VAvxIpI7JuaAvQ9H15T
BwiYdVtNFQy5QUrIw0C2PtHq6axTOMhe0UmBx7q2/Kq1MZyWmGhGK5UEaSTltZTL
TzKtaLkweJqhkVfgJaO7HK/reKhzrGJi7cqLzQcHdnIcF+lHGVQPzaQpNb6xI9S/
8u7tyS4F2dUCgYEAqiAyFxLQjwVHO1hxtTjRkCC7hYuB51i6+7u5djpjprMjmkiP
kf4lkAokaDiJSNM6C4lPrhKrwFJuLTjxzLVyDXmfb9mEWrUzFJFBNBkllTrujVQc
GtvoFr5FhyBXKMfhSEcQMmjlkfBhc1QrQqHs1EPluEhJQ8kVqEpQKi+vSgUCgYEA
wFvLNjuHKI4cDY4XEVOcgQnTnoNBW9+oxY6oQFOUvYM1XiHXyoHc5cRzaTjMBw3i
ncku3YOVNI9Ffm3Q3E+VnqBpzxzhzs6ri4GIyh5jhZAmIDWVFHf6dMnkqx+WX/BU
cjCyx9EqC/BroRRXtwgVxoPLh+WqFBmo2MmBWlgTa0UCgYEAwE2NoCJG9Ybzxacw
r3SviprVkfFMeCx4+TdAh/vMsdi0GO4pu9VU0bIcxSNZcqZiykHeKuF+MjlWBjJf
yOCTAJjCx4QjLyvMYRX84d+RgtQZnTLz+U4qJmP5Awuv+TI0hQDBQdSHGTJrAuYX
dMbA93s2GVvn2lKrz3jvjVpNXPU=
-----END PRIVATE KEY-----"#;

const MOCK_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu1SU1L7VLPHCgcBIjn0C
C9/wu/2P4sP1bhIhJx5f2IROBc8nSzj7BqVbY8ElBW101X1nx14kDTW+jeqExeZJ
RwlCrQB8TEw83ptrosFK7pB5Hy46aB4fTpDbxUGNlX5Kh16hItCdqG7CPu8IAY4I
Unl8jgiN/6UqBreEgQQV4z830OUA4mXiix7OoukYth33RpQ+Z+RXFwY12fDIzFwL
lR+6uZxocb3zFF46OX6EGy/JLuaZ+AJYBrYxkLlPbwwhIu0nke4P73ql4DNVXAgJ
TlRFl3uJlwQWy845QynSRDnxW/9pElh5rY3B9/5cBmuJ9lAV4nCZW5FbDO0Iw/QI
9QIDAQAB
-----END PUBLIC KEY-----"#;

/// Helper function to create a test JWT token
fn create_test_token(claims: Claims, private_key: &str) -> String {
    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();
    encode(&header, &claims, &encoding_key).unwrap()
}

/// TDD Test Case 1: Valid JWT validation
/// This test validates that our Resource Server can properly decode and validate
/// a JWT token that would be sent by a client application after OAuth2.0 authentication
#[test]
fn test_valid_jwt_validation() {
    // ARRANGE: Create a JWT validator as a Resource Server would
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

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

    let valid_jwt = create_test_token(claims, MOCK_PRIVATE_KEY);

    // ACT: The Resource Server validates the token and extracts user ID
    let user_id = validator.validate_token(&valid_jwt).unwrap();

    // ASSERT: The user ID should be correctly extracted
    assert_eq!(user_id, "user-123");
}

/// TDD Test Case 2: Invalid signature JWT rejection
/// This test ensures our Resource Server rejects tokens with invalid signatures
#[test]
fn test_invalid_signature_jwt() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

    // Create a token with an invalid signature (manually crafted)
    let invalid_jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImlzcyI6Imh0dHBzOi8vb2t0YS5jb20iLCJleHAiOjk5OTk5OTk5OTl9.invalid_signature_here";

    // ACT & ASSERT: The validator should reject the invalid token
    let result = validator.validate_token(invalid_jwt);
    assert!(matches!(result, Err(JwtValidationError::ValidationError(_))));
}

/// TDD Test Case 3: Expired token rejection
/// This test ensures our Resource Server rejects expired tokens
#[test]
fn test_expired_token_rejection() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

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

    let expired_jwt = create_test_token(expired_claims, MOCK_PRIVATE_KEY);

    // ACT & ASSERT: The validator should reject the expired token
    let result = validator.validate_token(&expired_jwt);
    assert!(matches!(result, Err(JwtValidationError::Expired)));
}

/// TDD Test Case 4: Invalid issuer rejection
/// This test ensures our Resource Server only accepts tokens from trusted issuers
#[test]
fn test_invalid_issuer_rejection() {
    // ARRANGE: Create a JWT validator expecting tokens from Okta
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

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

    let malicious_jwt = create_test_token(malicious_claims, MOCK_PRIVATE_KEY);

    // ACT & ASSERT: The validator should reject tokens from untrusted issuers
    let result = validator.validate_token(&malicious_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidIssuer)));
}

/// TDD Test Case 5: Scope extraction for authorization
/// This test validates that our Resource Server can extract OAuth2.0 scopes for authorization
#[test]
fn test_scope_extraction_for_authorization() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

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

    let jwt_with_scopes = create_test_token(claims_with_scopes, MOCK_PRIVATE_KEY);

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
fn test_bearer_token_extraction_from_header() {

    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

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

    let jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
    let auth_header = format!("Bearer {}", jwt);

    // ACT: Extract user ID from Authorization header (as would happen in middleware)
    let user_id = extract_user_from_bearer_token(&auth_header, &validator).unwrap();

    // ASSERT: User ID should be correctly extracted
    assert_eq!(user_id, "user-456");
}

/// TDD Test Case 7: Invalid Bearer format rejection
/// This test ensures malformed Authorization headers are rejected
#[test]
fn test_invalid_bearer_format_rejection() {

    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

    // ACT & ASSERT: Invalid header formats should be rejected
    let result = extract_user_from_bearer_token("Invalid token", &validator);
    assert!(matches!(result, Err(JwtValidationError::InvalidFormat)));

    let result = extract_user_from_bearer_token("Basic dXNlcjpwYXNz", &validator);
    assert!(matches!(result, Err(JwtValidationError::InvalidFormat)));
}

/// TDD Test Case 8: Audience validation
/// This test ensures our Resource Server validates the audience claim when configured
#[test]
fn test_audience_validation() {
    // ARRANGE: Create a JWT validator that expects a specific audience
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        Some("proof-messenger-api".to_string()),
    ).unwrap();

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

    let correct_jwt = create_test_token(correct_claims, MOCK_PRIVATE_KEY);

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

    let wrong_jwt = create_test_token(wrong_claims, MOCK_PRIVATE_KEY);

    // ACT & ASSERT: Wrong audience should be rejected
    let result = validator.validate_token(&wrong_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidAudience)));
}

/// TDD Test Case 9: Missing required claims
/// This test ensures tokens with missing required claims are rejected
#[test]
fn test_missing_required_claims() {
    // ARRANGE: Create a JWT validator
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        None,
    ).unwrap();

    // Create a token with empty subject (invalid)
    let invalid_claims = Claims {
        sub: "".to_string(), // Empty subject should be rejected
        iss: "https://okta.com".to_string(),
        aud: None,
        exp: 9999999999,
        iat: Some(1000000000),
        nbf: Some(1000000000),
        scope: None,
    };

    let invalid_jwt = create_test_token(invalid_claims, MOCK_PRIVATE_KEY);

    // ACT & ASSERT: Token with missing required claims should be rejected
    let result = validator.validate_token(&invalid_jwt);
    assert!(matches!(result, Err(JwtValidationError::MissingClaim(_))));
}

/// Integration Test: Complete OAuth2.0 Resource Server flow
/// This test simulates the complete flow of a Resource Server validating an OAuth2.0 token
#[test]
fn test_complete_oauth_resource_server_flow() {
    // ARRANGE: Set up the Resource Server (our Proof-Messenger Relay)
    let validator = JwtValidator::new_rsa256(
        MOCK_PUBLIC_KEY,
        "https://okta.com".to_string(),
        Some("proof-messenger-api".to_string()),
    ).unwrap();

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

    let client_jwt = create_test_token(client_token_claims, MOCK_PRIVATE_KEY);
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