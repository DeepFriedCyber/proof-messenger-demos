# Proof Object Specification

## 🎯 **Design Philosophy**

The proof object is designed to be **simple, self-contained, and developer-friendly**. It's a plain JSON object that can be easily stored, transmitted, and processed without complex deserialization or special handling.

## 📋 **Proof Object Structure**

```typescript
interface CryptographicProof {
  context: Record<string, any>;    // The signed context data
  signature: string;               // Base64-encoded signature
  publicKey: string;               // Base64-encoded public key
  algorithm: string;               // Signature algorithm (e.g., "Ed25519")
  identityToken?: string;          // Optional JWT from getIdentityToken config
}
```

## 🔍 **Example Proof Object**

```json
{
  "context": {
    "action": "approve-payment",
    "amount": 5000,
    "currency": "USD",
    "recipient": "ACME-CORP-123",
    "timestamp": "2025-07-14T12:30:00.000Z",
    "user_id": "user_12345",
    "session_id": "sess_abc123"
  },
  "signature": "MEUCIQDxK8rF2vN3mJ4pL6qR8sT9uV0wX1yZ2aB3cD4eF5gH6iJkLmNoP7qR8sT9uV0wX1yZ2aB3cD4eF5gH6i",
  "publicKey": "MCowBQYDK2VwAyEAm4f8Tv6nF2vN3mJ4pL6qR8sT9uV0wX1yZ2aB3cD4eF5gH6i",
  "algorithm": "Ed25519",
  "identityToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyXzEyMzQ1IiwiaWF0IjoxNjQyNzgxNDAwfQ.signature"
}
```

## 🔧 **Field Descriptions**

### **`context`** (Required)
- **Type**: `Record<string, any>` (JSON object)
- **Purpose**: The exact data that was cryptographically signed
- **Content**: Any JSON-serializable data structure
- **Examples**:
  ```typescript
  // Payment approval
  {
    "action": "approve-payment",
    "amount": 5000,
    "currency": "USD",
    "recipient": "ACME-CORP-123",
    "timestamp": "2025-07-14T12:30:00.000Z"
  }
  
  // Document signing
  {
    "action": "sign-document",
    "document_id": "DOC-2024-001",
    "document_hash": "sha256:abc123...",
    "signer_role": "CFO",
    "timestamp": "2025-07-14T12:30:00.000Z"
  }
  
  // Access approval
  {
    "action": "grant-access",
    "resource": "/admin/users",
    "permission": "read",
    "duration_hours": 24,
    "timestamp": "2025-07-14T12:30:00.000Z"
  }
  ```

### **`signature`** (Required)
- **Type**: `string`
- **Format**: Base64-encoded
- **Purpose**: Cryptographic signature of the context data
- **Generation**: Created by signing the JSON-serialized context with the user's private key
- **Verification**: Used to verify the authenticity and integrity of the context

### **`publicKey`** (Required)
- **Type**: `string`
- **Format**: Base64-encoded
- **Purpose**: The user's public key used to verify the signature
- **Source**: Generated from the user's hardware security key or biometric authentication
- **Usage**: Enables signature verification without needing to store user keys

### **`algorithm`** (Required)
- **Type**: `string`
- **Purpose**: Specifies the cryptographic algorithm used for signing
- **Supported Values**:
  - `"Ed25519"` - EdDSA using Curve25519 (recommended)
  - `"ECDSA-P256"` - ECDSA using P-256 curve
  - `"ECDSA-P384"` - ECDSA using P-384 curve
- **Usage**: Tells the verifier which algorithm to use for signature verification

### **`identityToken`** (Optional)
- **Type**: `string`
- **Format**: JWT (JSON Web Token)
- **Purpose**: Links the proof to an authenticated user session
- **Source**: Returned by the `getIdentityToken` function in client configuration
- **Content**: Contains user identity information (user ID, roles, etc.)
- **Usage**: Enables user identification and authorization context

## 🔒 **Security Properties**

### **Integrity**
- The `signature` ensures the `context` data hasn't been tampered with
- Any modification to the context will cause signature verification to fail
- The proof is cryptographically bound to the exact context data

### **Authenticity**
- The `publicKey` proves which user created the proof
- The signature can only be created by someone with access to the corresponding private key
- Hardware-backed keys provide strong authentication guarantees

### **Non-Repudiation**
- The user cannot deny creating the proof (signature proves authorship)
- The exact context data is cryptographically bound to the user's identity
- Provides legal-grade evidence of user intent

### **Freshness**
- The `timestamp` in context prevents replay attacks
- Verifiers can reject proofs that are too old
- Each proof is unique due to timestamp inclusion

## 📦 **Storage and Transmission**

### **JSON Serialization**
```typescript
// Serialize for storage or transmission
const proofJson = JSON.stringify(proof);

// Deserialize from storage or network
const proof = JSON.parse(proofJson);
```

### **Database Storage**
```sql
-- PostgreSQL example
CREATE TABLE transaction_proofs (
  id SERIAL PRIMARY KEY,
  transaction_id VARCHAR(255),
  proof_context JSONB,           -- Queryable JSON
  proof_signature TEXT,          -- Base64 signature
  proof_public_key TEXT,         -- Base64 public key
  proof_algorithm VARCHAR(50),   -- Algorithm name
  identity_token TEXT,           -- Optional JWT
  created_at TIMESTAMP DEFAULT NOW()
);

-- Insert proof
INSERT INTO transaction_proofs 
(transaction_id, proof_context, proof_signature, proof_public_key, proof_algorithm, identity_token)
VALUES ($1, $2, $3, $4, $5, $6);

-- Query by action type
SELECT * FROM transaction_proofs 
WHERE proof_context->>'action' = 'approve-payment'
AND created_at > NOW() - INTERVAL '30 days';
```

### **HTTP Transmission**
```typescript
// Send proof in HTTP request
const response = await fetch('/api/execute-payment', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ proof })
});

// Receive proof in API endpoint
app.post('/api/execute-payment', (req, res) => {
  const { proof } = req.body;
  // proof is already a parsed JavaScript object
});
```

### **Compact Encoding (Optional)**
```typescript
// Base64 encoding for URLs or compact storage
const compactProof = btoa(JSON.stringify(proof));
const originalProof = JSON.parse(atob(compactProof));

// Compression for large contexts
import { gzip, ungzip } from 'pako';
const compressed = gzip(JSON.stringify(proof));
const decompressed = JSON.parse(ungzip(compressed, { to: 'string' }));
```

## ✅ **Validation Examples**

### **Structure Validation**
```typescript
function isValidProof(obj: any): obj is CryptographicProof {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof obj.context === 'object' &&
    obj.context !== null &&
    typeof obj.signature === 'string' &&
    obj.signature.length > 0 &&
    typeof obj.publicKey === 'string' &&
    obj.publicKey.length > 0 &&
    typeof obj.algorithm === 'string' &&
    ['Ed25519', 'ECDSA-P256', 'ECDSA-P384'].includes(obj.algorithm) &&
    (obj.identityToken === undefined || typeof obj.identityToken === 'string')
  );
}
```

### **Business Logic Validation**
```typescript
function validatePaymentProof(proof: CryptographicProof): boolean {
  const { context } = proof;
  
  return (
    context.action === 'approve-payment' &&
    typeof context.amount === 'number' &&
    context.amount > 0 &&
    typeof context.currency === 'string' &&
    context.currency.length === 3 && // ISO currency code
    typeof context.recipient === 'string' &&
    context.recipient.length > 0 &&
    typeof context.timestamp === 'string' &&
    !isNaN(Date.parse(context.timestamp)) // Valid ISO timestamp
  );
}
```

### **Security Validation**
```typescript
function validateProofSecurity(proof: CryptographicProof): boolean {
  // Check timestamp freshness (within last 5 minutes)
  const timestamp = new Date(proof.context.timestamp);
  const now = new Date();
  const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000);
  
  if (timestamp < fiveMinutesAgo) {
    throw new Error('Proof is too old');
  }
  
  // Check signature format (Base64)
  if (!/^[A-Za-z0-9+/]+=*$/.test(proof.signature)) {
    throw new Error('Invalid signature format');
  }
  
  // Check public key format (Base64)
  if (!/^[A-Za-z0-9+/]+=*$/.test(proof.publicKey)) {
    throw new Error('Invalid public key format');
  }
  
  return true;
}
```

## 🔍 **Verification Process**

### **Client-Side Pre-Verification**
```typescript
// Before sending to server
function preVerifyProof(proof: CryptographicProof): boolean {
  // 1. Structure validation
  if (!isValidProof(proof)) {
    throw new Error('Invalid proof structure');
  }
  
  // 2. Business logic validation
  if (!validatePaymentProof(proof)) {
    throw new Error('Invalid payment proof data');
  }
  
  // 3. Security validation
  if (!validateProofSecurity(proof)) {
    throw new Error('Proof security validation failed');
  }
  
  return true;
}
```

### **Server-Side Cryptographic Verification**
```typescript
// On the server
async function verifyProof(proof: CryptographicProof) {
  // 1. Validate structure
  if (!isValidProof(proof)) {
    throw new Error('Invalid proof structure');
  }
  
  // 2. Cryptographic verification
  const verification = await proofMessenger.verifyProof(proof);
  
  if (!verification.valid) {
    throw new Error(`Proof verification failed: ${verification.error}`);
  }
  
  // 3. Identity token verification (if present)
  if (proof.identityToken) {
    const tokenValid = await verifyJWT(proof.identityToken);
    if (!tokenValid) {
      throw new Error('Invalid identity token');
    }
  }
  
  return verification;
}
```

## 📊 **Audit and Compliance**

### **Audit Trail Storage**
```typescript
async function storeAuditTrail(proof: CryptographicProof, result: any) {
  const auditEntry = {
    // Proof identification
    proof_signature: proof.signature,
    proof_public_key: proof.publicKey,
    proof_algorithm: proof.algorithm,
    
    // Context data
    action: proof.context.action,
    context_data: proof.context,
    
    // Identity information
    user_id: extractUserIdFromToken(proof.identityToken),
    identity_token: proof.identityToken,
    
    // Execution metadata
    execution_result: result,
    executed_at: new Date(),
    
    // Compliance data
    ip_address: getClientIP(),
    user_agent: getUserAgent(),
    
    // Complete proof for forensics
    original_proof: proof
  };
  
  await auditLog.create(auditEntry);
}
```

### **Compliance Reporting**
```typescript
async function generateComplianceReport(filters: {
  startDate: Date;
  endDate: Date;
  userId?: string;
  action?: string;
}) {
  const auditEntries = await auditLog.findAll({
    where: {
      executed_at: {
        [Op.between]: [filters.startDate, filters.endDate]
      },
      ...(filters.userId && { user_id: filters.userId }),
      ...(filters.action && { action: filters.action })
    }
  });
  
  return auditEntries.map(entry => ({
    timestamp: entry.executed_at,
    user_id: entry.user_id,
    action: entry.action,
    context: entry.context_data,
    proof_algorithm: entry.proof_algorithm,
    execution_status: entry.execution_result.success ? 'SUCCESS' : 'FAILED',
    proof_signature: entry.proof_signature.substring(0, 16) + '...' // Truncated for report
  }));
}
```

## 🎯 **Key Benefits**

### **Developer Experience**
- **Simple JSON**: No complex deserialization or special handling required
- **Self-Contained**: All verification data included in the object
- **Type Safe**: Full TypeScript support with clear interfaces
- **Debuggable**: Human-readable structure for easy troubleshooting

### **Security**
- **Tamper-Evident**: Any modification breaks the cryptographic signature
- **Non-Repudiable**: Cryptographic proof of user intent
- **Hardware-Backed**: Leverages device security features
- **Standards-Based**: Uses proven cryptographic algorithms

### **Operational**
- **Auditable**: Complete audit trail with original proof preservation
- **Scalable**: Efficient storage and transmission
- **Compliant**: Supports regulatory requirements
- **Forensic**: Enables detailed investigation of transactions

This proof object design strikes the perfect balance between simplicity for developers and security for enterprises, making it easy to integrate while providing strong cryptographic guarantees.

---

**Specification Version**: 1.0  
**Last Updated**: December 2024  
**Compatibility**: All Proof-Messenger SDK versions  
**Security Level**: Enterprise-grade