# OAuth2.0 Resource Server Implementation

## Overview

This implementation transforms the Proof-Messenger Relay Server into a proper **OAuth2.0 Resource Server** that validates JWT tokens from Identity Providers (IdPs) like Okta, Auth0, or Azure AD.

## Key Correction from Initial Approach

**Initial Misunderstanding**: Implementing an OAuth client to get access tokens
**Corrected Approach**: Implementing a Resource Server that validates received JWT tokens

### Why This Matters

In OAuth2.0 flows:
- **Authorization Server** (e.g., Okta): Issues tokens
- **Client Application**: Requests and uses tokens  
- **Resource Server** (our relay): Validates tokens and serves protected resources

Our relay server should **validate** tokens, not **request** them.

## Implementation Architecture

### Core Components

1. **JWT Validator** (`jwt_validator.rs`)
   - Validates JWT signatures using public keys from IdP
   - Verifies issuer, audience, expiration
   - Extracts user ID and scopes for authorization

2. **Authentication Middleware** (`auth_middleware.rs`)
   - Intercepts requests to protected endpoints
   - Extracts and validates JWT from Authorization header
   - Adds authentication context to requests

3. **Protected Handlers** (in `lib.rs`)
   - OAuth2.0-aware versions of relay endpoints
   - Enforce scope-based authorization
   - Include user context in responses

## TDD Test Implementation

### Test Coverage

✅ **Valid JWT Validation**
```rust
#[test]
fn test_valid_jwt_validation_hmac() {
    let validator = JwtValidator::new_hmac(
        "test-secret-key",
        "https://okta.com".to_string(),
        None,
    );
    
    let claims = Claims {
        sub: "user-123".to_string(),
        iss: "https://okta.com".to_string(),
        exp: 9999999999, // Far future
        // ...
    };
    
    let jwt = create_test_token_hmac(claims, "test-secret-key");
    let user_id = validator.validate_token(&jwt).unwrap();
    
    assert_eq!(user_id, "user-123");
}
```

✅ **Invalid Signature Rejection**
```rust
#[test]
fn test_invalid_signature_jwt_hmac() {
    let validator = JwtValidator::new_hmac("test-secret-key", ...);
    let invalid_jwt = create_test_token_hmac(claims, "wrong-secret-key");
    
    let result = validator.validate_token(&invalid_jwt);
    assert!(matches!(result, Err(JwtValidationError::InvalidSignature)));
}
```

✅ **Expired Token Rejection**
✅ **Invalid Issuer Rejection**  
✅ **Scope Extraction for Authorization**
✅ **Bearer Token Extraction from Headers**
✅ **Audience Validation**
✅ **Complete OAuth2.0 Resource Server Flow**

## API Endpoints

### Protected Endpoints (Require JWT)

| Endpoint | Method | Required Scope | Description |
|----------|--------|----------------|-------------|
| `/relay` | POST | `proof:create` | Create and verify proofs |
| `/messages/:group_id` | GET | `message:read` | Get messages by group |
| `/message/:message_id` | GET | `message:read` | Get specific message |

### Public Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/ready` | GET | Readiness check |

## Usage Examples

### 1. Client Authentication Flow

```bash
# 1. Client gets JWT from Identity Provider (outside our system)
curl -X POST https://your-okta-domain/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials&client_id=your-client&client_secret=your-secret&scope=proof:create message:read"

# Response: {"access_token": "eyJ0eXAiOiJKV1Q...", "token_type": "Bearer", ...}
```

### 2. Using JWT with Resource Server

```bash
# 2. Client uses JWT to access protected resources
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1Q..." \
     -H "Content-Type: application/json" \
     -d '{"sender":"...","context":"...","body":"...","proof":"..."}' \
     http://localhost:3000/relay

# Response: {"status":"success","message_id":"...","authenticated_user":"user-123"}
```

### 3. Scope-Based Authorization

```bash
# This will fail if JWT doesn't contain "message:read" scope
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1Q..." \
     http://localhost:3000/messages/group1

# Response: 403 Forbidden (if insufficient scope)
# Response: 200 OK with data (if valid scope)
```

## Configuration for Production

### 1. RSA Key Configuration

```rust
// For production, use RSA public key from IdP's JWKS endpoint
let jwt_validator = Arc::new(JwtValidator::new_rsa256(
    &public_key_pem,  // From https://your-okta-domain/.well-known/jwks
    "https://your-okta-domain".to_string(),
    Some("proof-messenger-api".to_string()),
)?);
```

### 2. Server Setup

```rust
use proof_messenger_relay::{create_app_with_oauth, Database, jwt_validator::JwtValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(Database::new("database.db").await?);
    db.migrate().await?;
    
    let jwt_validator = Arc::new(JwtValidator::new_rsa256(
        &std::fs::read_to_string("public_key.pem")?,
        "https://your-okta-domain".to_string(),
        Some("proof-messenger-api".to_string()),
    )?);
    
    let app = create_app_with_oauth(db, jwt_validator);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Security Features

### 1. Token Validation
- ✅ Signature verification using IdP public key
- ✅ Issuer validation (prevents token reuse across systems)
- ✅ Audience validation (ensures token is for our API)
- ✅ Expiration checking
- ✅ Not-before time validation

### 2. Authorization
- ✅ Scope-based access control
- ✅ User context extraction
- ✅ Fine-grained permissions per endpoint

### 3. Security Headers
- ✅ HSTS (HTTP Strict Transport Security)
- ✅ X-Content-Type-Options: nosniff
- ✅ X-Frame-Options: DENY
- ✅ CORS configuration

## Integration with Identity Providers

### Okta Integration

1. **Configure Okta Application**:
   - Create API application in Okta
   - Define custom scopes: `proof:create`, `message:read`
   - Note the issuer URL and audience

2. **Get Public Key**:
   ```bash
   curl https://your-okta-domain/.well-known/jwks
   ```

3. **Configure Resource Server**:
   ```rust
   let jwt_validator = JwtValidator::new_rsa256(
       &public_key_from_jwks,
       "https://your-okta-domain".to_string(),
       Some("api://proof-messenger").to_string(),
   )?;
   ```

### Auth0 Integration

Similar process with Auth0-specific endpoints:
- JWKS: `https://your-domain.auth0.com/.well-known/jwks.json`
- Issuer: `https://your-domain.auth0.com/`

## Testing Strategy

### Unit Tests
- JWT validation logic
- Scope extraction
- Error handling for various failure modes

### Integration Tests  
- End-to-end OAuth2.0 flows
- Middleware integration
- Protected endpoint access

### Security Tests
- Invalid token rejection
- Expired token handling
- Scope enforcement
- Audience validation

## Error Handling

The implementation provides detailed error responses:

```json
// Invalid token
{
  "error": "Invalid signature",
  "status": 401
}

// Insufficient scope
{
  "error": "Insufficient permissions to create proofs", 
  "status": 500
}

// Missing token
{
  "error": "Unauthorized",
  "status": 401
}
```

## Performance Considerations

1. **Token Caching**: Consider caching validated tokens (with TTL)
2. **Public Key Caching**: Cache IdP public keys with refresh mechanism
3. **Async Validation**: All validation is async-friendly
4. **Minimal Dependencies**: Uses efficient JWT libraries

## Deployment

### Docker Configuration

The OAuth2.0 Resource Server can be deployed using the existing Docker setup with additional environment variables:

```yaml
# docker-compose.yml
services:
  proof-messenger-relay:
    environment:
      - JWT_ISSUER=https://your-okta-domain
      - JWT_AUDIENCE=proof-messenger-api
      - JWT_PUBLIC_KEY_PATH=/app/public_key.pem
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proof-messenger-relay
spec:
  template:
    spec:
      containers:
      - name: relay
        env:
        - name: JWT_ISSUER
          value: "https://your-okta-domain"
        - name: JWT_AUDIENCE  
          value: "proof-messenger-api"
```

## Conclusion

This implementation correctly positions the Proof-Messenger Relay Server as an **OAuth2.0 Resource Server** that:

1. ✅ **Validates** JWT tokens (doesn't request them)
2. ✅ **Enforces** scope-based authorization
3. ✅ **Integrates** with standard Identity Providers
4. ✅ **Follows** OAuth2.0 best practices
5. ✅ **Provides** comprehensive test coverage
6. ✅ **Implements** proper security measures

The TDD approach ensures robust validation logic, and the modular design allows easy integration with various Identity Providers while maintaining security and performance.