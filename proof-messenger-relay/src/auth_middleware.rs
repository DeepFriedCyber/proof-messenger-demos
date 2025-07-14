use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use crate::jwt_validator::{JwtValidator, JwtValidationError, extract_user_from_bearer_token};

/// Authentication context that gets added to request extensions
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub scopes: std::collections::HashSet<String>,
}

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(validator): State<Arc<JwtValidator>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate the JWT token
    let user_id = extract_user_from_bearer_token(auth_header, &validator)
        .map_err(|e| match e {
            JwtValidationError::InvalidFormat => StatusCode::BAD_REQUEST,
            JwtValidationError::InvalidSignature => StatusCode::UNAUTHORIZED,
            JwtValidationError::Expired => StatusCode::UNAUTHORIZED,
            JwtValidationError::InvalidIssuer => StatusCode::UNAUTHORIZED,
            JwtValidationError::InvalidAudience => StatusCode::UNAUTHORIZED,
            JwtValidationError::MissingClaim(_) => StatusCode::BAD_REQUEST,
            JwtValidationError::ValidationError(_) => StatusCode::UNAUTHORIZED,
        })?;

    // Extract scopes for authorization
    let token = &auth_header[7..]; // Remove "Bearer " prefix
    let scopes = validator.extract_scopes(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add authentication context to request extensions
    let auth_context = AuthContext { user_id, scopes };
    request.extensions_mut().insert(auth_context);

    // Continue to the next middleware/handler
    Ok(next.run(request).await)
}

/// Authorization helper to check if user has required scope
pub fn require_scope(auth_context: &AuthContext, required_scope: &str) -> Result<(), StatusCode> {
    if auth_context.scopes.contains(required_scope) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Extractor for authentication context from request extensions
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt_validator::{JwtValidator, Claims};
    use axum::{
        body::Body,
        http::{Request, Method},
        middleware,

        routing::get,
        Router,
    };
    use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
    use tower::ServiceExt;

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

    fn create_test_token(claims: Claims) -> String {
        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(MOCK_PRIVATE_KEY.as_bytes()).unwrap();
        encode(&header, &claims, &encoding_key).unwrap()
    }

    async fn protected_handler(_auth: AuthContext) -> &'static str {
        // This handler requires authentication
        "Protected resource accessed"
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        // ARRANGE: Set up the validator and app with auth middleware
        let validator = Arc::new(JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap());

        let app = Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(validator.clone(), auth_middleware))
            .with_state(validator);

        // Create a valid token
        let claims = Claims {
            sub: "user-123".to_string(),
            iss: "https://okta.com".to_string(),
            aud: None,
            exp: 9999999999,
            iat: Some(1000000000),
            nbf: Some(1000000000),
            scope: Some("read write".to_string()),
        };

        let token = create_test_token(claims);

        // ACT: Make a request with valid authorization header
        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be successful
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        // ARRANGE: Set up the validator and app with auth middleware
        let validator = Arc::new(JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap());

        let app = Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(validator.clone(), auth_middleware))
            .with_state(validator);

        // ACT: Make a request without authorization header
        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_invalid_token() {
        // ARRANGE: Set up the validator and app with auth middleware
        let validator = Arc::new(JwtValidator::new_rsa256(
            MOCK_PUBLIC_KEY,
            "https://okta.com".to_string(),
            None,
        ).unwrap());

        let app = Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(validator.clone(), auth_middleware))
            .with_state(validator);

        // ACT: Make a request with invalid token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .header("authorization", "Bearer invalid.token.here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // ASSERT: Request should be unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_require_scope_with_valid_scope() {
        // ARRANGE: Create auth context with scopes
        let mut scopes = std::collections::HashSet::new();
        scopes.insert("read".to_string());
        scopes.insert("write".to_string());

        let auth_context = AuthContext {
            user_id: "user-123".to_string(),
            scopes,
        };

        // ACT & ASSERT: Should allow access with valid scope
        assert!(require_scope(&auth_context, "read").is_ok());
        assert!(require_scope(&auth_context, "write").is_ok());
    }

    #[test]
    fn test_require_scope_with_invalid_scope() {
        // ARRANGE: Create auth context with limited scopes
        let mut scopes = std::collections::HashSet::new();
        scopes.insert("read".to_string());

        let auth_context = AuthContext {
            user_id: "user-123".to_string(),
            scopes,
        };

        // ACT & ASSERT: Should deny access without required scope
        let result = require_scope(&auth_context, "admin");
        assert!(matches!(result, Err(StatusCode::FORBIDDEN)));
    }
}