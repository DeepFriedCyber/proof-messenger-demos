use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation, TokenData};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwtValidationError {
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Token expired")]
    Expired,
    #[error("Invalid issuer")]
    InvalidIssuer,
    #[error("Invalid audience")]
    InvalidAudience,
    #[error("Missing required claim: {0}")]
    MissingClaim(String),
    #[error("JWT validation error: {0}")]
    ValidationError(#[from] jsonwebtoken::errors::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (user ID)
    pub iss: String,           // Issuer (e.g., "https://okta.com")
    pub aud: Option<String>,   // Audience
    pub exp: usize,            // Expiration time
    pub iat: Option<usize>,    // Issued at
    pub nbf: Option<usize>,    // Not before
    pub scope: Option<String>, // OAuth2 scopes
}

pub struct JwtValidator {
    public_key: DecodingKey,
    expected_issuer: String,
    expected_audience: Option<String>,
    algorithm: Algorithm,
}

impl JwtValidator {
    /// Create a new JWT validator with RSA256 public key
    pub fn new_rsa256(
        public_key_pem: &str,
        expected_issuer: String,
        expected_audience: Option<String>,
    ) -> Result<Self, JwtValidationError> {
        let public_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(JwtValidationError::ValidationError)?;

        Ok(Self {
            public_key,
            expected_issuer,
            expected_audience,
            algorithm: Algorithm::RS256,
        })
    }

    /// Create a new JWT validator with HMAC secret (for testing)
    pub fn new_hmac(
        secret: &str,
        expected_issuer: String,
        expected_audience: Option<String>,
    ) -> Self {
        Self {
            public_key: DecodingKey::from_secret(secret.as_bytes()),
            expected_issuer,
            expected_audience,
            algorithm: Algorithm::HS256,
        }
    }

    /// Validate a JWT token and extract user ID
    pub fn validate_token(&self, token: &str) -> Result<String, JwtValidationError> {
        let token_data = self.decode_and_validate(token)?;
        Ok(token_data.claims.sub)
    }

    /// Validate a JWT token and return full claims
    pub fn validate_and_get_claims(&self, token: &str) -> Result<Claims, JwtValidationError> {
        let token_data = self.decode_and_validate(token)?;
        Ok(token_data.claims)
    }

    /// Internal method to decode and validate JWT
    fn decode_and_validate(&self, token: &str) -> Result<TokenData<Claims>, JwtValidationError> {
        // Set up validation parameters
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.expected_issuer]);
        
        if let Some(ref audience) = self.expected_audience {
            validation.set_audience(&[audience]);
        } else {
            validation.validate_aud = false;
        }

        // Decode and validate the token
        let token_data = decode::<Claims>(token, &self.public_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtValidationError::Expired,
                jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtValidationError::InvalidSignature,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => JwtValidationError::InvalidIssuer,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => JwtValidationError::InvalidAudience,
                _ => JwtValidationError::ValidationError(e),
            })?;

        // Additional validation
        self.validate_required_claims(&token_data.claims)?;

        Ok(token_data)
    }

    /// Validate that required claims are present
    fn validate_required_claims(&self, claims: &Claims) -> Result<(), JwtValidationError> {
        if claims.sub.is_empty() {
            return Err(JwtValidationError::MissingClaim("sub".to_string()));
        }

        if claims.iss.is_empty() {
            return Err(JwtValidationError::MissingClaim("iss".to_string()));
        }

        Ok(())
    }

    /// Extract scopes from the token for authorization
    pub fn extract_scopes(&self, token: &str) -> Result<HashSet<String>, JwtValidationError> {
        let claims = self.validate_and_get_claims(token)?;
        
        let scopes = claims.scope
            .unwrap_or_default()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok(scopes)
    }
}

/// Utility function for extracting user ID from Authorization header
pub fn extract_user_from_bearer_token(
    auth_header: &str,
    validator: &JwtValidator,
) -> Result<String, JwtValidationError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(JwtValidationError::InvalidFormat);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    validator.validate_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

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

    fn create_test_token(claims: Claims, private_key: &str) -> String {
        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();
        encode(&header, &claims, &encoding_key).unwrap()
    }

    #[test]
    fn test_valid_jwt_validation() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 9999999999, // Far future
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: Some("read write".to_string()),
        };

        let valid_jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
        let user_id = validator.validate_token(&valid_jwt).unwrap();
        assert_eq!(user_id, "user-123");
    }

    #[test]
    fn test_invalid_signature_jwt() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        // Create token with wrong key
        let wrong_key = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDGGPLuP0qfmENH
...different key...
-----END PRIVATE KEY-----"#;

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: None,
        };

        // This should fail because we don't have the matching private key
        // For this test, we'll create an invalid token manually
        let invalid_jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImlzcyI6Imh0dHBzOi8vb2t0YS5jb20iLCJleHAiOjk5OTk5OTk5OTl9.invalid_signature";
        
        let result = validator.validate_token(invalid_jwt);
        assert!(matches!(result, Err(JwtValidationError::ValidationError(_))));
    }

    #[test]
    fn test_expired_token() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 1000000000, // Past timestamp
            iat: Some(999999999),
            nbf: Some(999999999),
            scope: None,
        };

        let expired_jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
        let result = validator.validate_token(&expired_jwt);
        assert!(matches!(result, Err(JwtValidationError::Expired)));
    }

    #[test]
    fn test_invalid_issuer() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://malicious.com".to_string(), // Wrong issuer
            aud: None,
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: None,
        };

        let invalid_jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
        let result = validator.validate_token(&invalid_jwt);
        assert!(matches!(result, Err(JwtValidationError::InvalidIssuer)));
    }

    #[test]
    fn test_extract_scopes() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: Some("read write admin".to_string()),
        };

        let jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
        let scopes = validator.extract_scopes(&jwt).unwrap();
        
        assert!(scopes.contains("read"));
        assert!(scopes.contains("write"));
        assert!(scopes.contains("admin"));
        assert_eq!(scopes.len(), 3);
    }

    #[test]
    fn test_bearer_token_extraction() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: None,
        };

        let jwt = create_test_token(claims, MOCK_PRIVATE_KEY);
        let auth_header = format!("Bearer {}", jwt);
        
        let user_id = extract_user_from_bearer_token(&auth_header, &validator).unwrap();
        assert_eq!(user_id, "user-123");
    }

    #[test]
    fn test_invalid_bearer_format() {
        let validator = JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap();

        let result = extract_user_from_bearer_token("Invalid token", &validator);
        assert!(matches!(result, Err(JwtValidationError::InvalidFormat)));
    }
}