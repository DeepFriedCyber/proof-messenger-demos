# Proof Revocation Mechanism

## Overview

The Proof Revocation Mechanism provides a way to invalidate cryptographic proofs after they have been created but before they have been used to complete a workflow. This is essential for real-world scenarios where workflows may need to be canceled or revoked due to various business reasons.

## The Challenge: Real-World Complexity

In a perfect world, a proof is generated and immediately used to authorize an irreversible action. However, in the real world, workflows get canceled for various legitimate reasons:

- A user might approve a payment, but the recipient's account is discovered to be fraudulent a minute later
- A doctor might get consent, but the patient changes their mind before the records are accessed
- A transaction might be flagged by a fraud detection system after initial approval

We need a mechanism to invalidate a proof after it has been created but before it has been used to complete a workflow. This is a notoriously difficult problem in distributed systems.

## Our Approach: A Pragmatic Revocation List

We adopt a simple, robust, and understandable pattern: a centralized Revocation List managed by the application backend. This avoids overly complex and slow solutions like blockchain-based revocation.

## Core Components

### 1. The Revocation List

This is simply a list of identifiers for proofs that are no longer valid. In its simplest form, it is a database table that stores the unique signatures of revoked proofs.

Key features:
- Uses the proof signature as the unique identifier
- Supports optional TTL (Time-to-Live) for automatic expiration
- Stores metadata like revocation reason and who performed the revocation

### 2. The Application Backend

This is the "owner" of the business logic. It alone decides when and why a proof should be revoked. It exposes secure API endpoints to:
- Revoke a proof
- Check if a proof is revoked
- List active revocations
- Clean up expired revocations

### 3. The Relay Server

Its verification logic has been updated. Before performing the expensive cryptographic check, it first performs a quick, cheap check against the Revocation List.

## The Updated Verification Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Receive Proof  │────▶│ Check if Proof  │────▶│  Cryptographic  │
│                 │     │   is Revoked    │     │   Verification  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │                        │
                               │                        │
                               ▼                        ▼
                        ┌─────────────┐         ┌─────────────┐
                        │             │         │             │
                        │   Reject    │         │   Accept    │
                        │             │         │             │
                        └─────────────┘         └─────────────┘
```

The revocation check is a simple, understandable first step before the core cryptographic work happens.

## The Revocation Process

1. **Event Trigger**: A business event occurs that requires a previously issued proof to be invalidated. For example, the application's fraud detection engine flags a transaction after the user has approved it.

2. **Revocation Command**: The Application Backend makes a secure, authenticated API call to its own internal "Revoke Proof" endpoint. The only piece of information it needs to send is the unique identifier of the proof. The best identifier is the signature itself, as it is guaranteed to be unique.

3. **Update the List**: The revocation endpoint adds the proof's signature to the Revocation List database. It's also recommended to set a Time-to-Live (TTL) on this entry. A proof is generally only valid for a short-lived workflow, so its revocation status doesn't need to be stored forever. A TTL of 24 hours would be more than sufficient for most use cases.

## API Endpoints

The Relay Server provides the following API endpoints for proof revocation:

### Public Endpoints

- `POST /revocation/revoke` - Revoke a proof
- `GET /revocation/check/:signature` - Check if a proof is revoked
- `GET /revocation/list` - List all active revocations
- `POST /revocation/cleanup` - Clean up expired revocations

### Authenticated Endpoints (with OAuth2.0)

- `POST /revocation/revoke` - Revoke a proof (requires `proof:revoke` scope)
- `GET /revocation/check/:signature` - Check if a proof is revoked (requires `proof:read` scope)
- `GET /revocation/list` - List all active revocations (requires `proof:read` scope)
- `POST /revocation/cleanup` - Clean up expired revocations (requires `proof:manage` scope)

## Configuration

To support revocation, the Relay Server uses the following environment variables:

```
# .env for Relay Server
REVOCATION_CHECK_ENABLED=true
REVOCATION_LIST_API_URL="https://api.my-app.com/internal/check-revocation"
REVOCATION_LIST_API_KEY="secure-internal-api-key"
REVOCATION_DEFAULT_TTL_HOURS=24
```

## Implementation Example

### Revoking a Proof

```bash
curl -X POST http://localhost:3000/revocation/revoke \
  -H "Content-Type: application/json" \
  -d '{
    "proof_signature": "a1b2c3d4e5f6...",
    "reason": "Fraud detection alert",
    "ttl_hours": 24
  }'
```

### Checking if a Proof is Revoked

```bash
curl http://localhost:3000/revocation/check/a1b2c3d4e5f6...
```

### Relay Server's Verification Logic

```rust
// Pseudocode for Relay Server's main handler
async fn process_and_verify_message(message: &Message, db: Option<&Database>) -> Result<(), Error> {
  // Check if revocation is enabled
  if std::env::var("REVOCATION_CHECK_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
    if let Some(db) = db {
      // Check if the proof is in the revocation list
      if db.is_proof_revoked(&message.proof).await? {
        return Err(Error::ProofRevoked);
      }
    }
  }
  
  // If not revoked, proceed with cryptographic verification
  verify_proof_cryptographically(message)?;
  
  Ok(())
}
```

## Best Practices

1. **Set Appropriate TTLs**: Proofs should have a natural expiration time. The revocation TTL should be at least as long as the maximum expected time between proof creation and use.

2. **Secure the Revocation API**: The ability to revoke proofs is powerful and should be properly secured with authentication and authorization.

3. **Monitor Revocation Patterns**: Unusual patterns in proof revocations might indicate security issues or user experience problems.

4. **Provide Clear User Feedback**: When a proof is rejected due to revocation, provide clear feedback to the user about what happened and what they should do next.

5. **Implement Cleanup Jobs**: Regularly clean up expired revocations to keep the database efficient.

## Conclusion

By implementing this revocation mechanism, we demonstrate a deep understanding of the entire lifecycle of a transaction in a complex enterprise environment. We show that we have thought beyond the "happy path" and have provided a practical solution for when things go wrong.