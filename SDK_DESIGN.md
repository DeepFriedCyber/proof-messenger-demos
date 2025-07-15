# Proof-Messenger JavaScript/TypeScript SDK Design

## 🎯 **Design Goals**

1. **⚡ Simplicity**: Abstract away all cryptographic complexity
2. **🔍 Clarity**: Intuitive method names reflecting business actions
3. **🛡️ Type Safety**: Full TypeScript support with excellent autocompletion
4. **🚀 Modern**: Promise-based with async/await syntax
5. **📱 Universal**: Works in browsers, Node.js, and React Native

## 📋 **Core API Design**

### 1. Initialization

```typescript
// Import the factory function
import { createProofMessengerClient } from '@proof-messenger/sdk';

// Define the configuration for the client
const config = {
  // The URL of the self-hosted Relay Server
  relayUrl: 'https://proof-relay.my-company.com/verify',
  
  // Optional: A function that provides the user's identity token (e.g., a JWT)
  // This is used to bind the proof to an authenticated session.
  getIdentityToken: async () => {
    // This function would get the current user's session token
    // from your auth provider (e.g., Auth0, Okta, custom).
    return getSession().jwt;
  },
  
  // Optional: Custom error handler
  onError: (error) => {
    console.error('Proof-Messenger error:', error);
    // Send to your error tracking service
  },
  
  // Optional: Development mode for testing
  development: process.env.NODE_ENV === 'development'
};

// Create the client instance
const proofMessenger = createProofMessengerClient(config);
```

### 2. Request Cryptographic Proof

```typescript
// Inside an application function, e.g., when a user clicks "Approve Payment"
async function onApprovePayment(paymentDetails) {
  try {
    // 1. Define the context for the proof. This is any JSON object.
    const context = {
      action: 'approve-payment',
      amount: paymentDetails.amount,
      currency: paymentDetails.currency,
      recipient: paymentDetails.recipient,
      timestamp: new Date().toISOString()
    };

    // 2. Request the proof from the user.
    // The SDK handles showing the browser's security prompt (e.g., Touch ID / Windows Hello).
    const proof = await proofMessenger.requestProof({ context });

    // 3. Send the proof to your application backend for server-side verification.
    await fetch('/api/execute-payment', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    console.log('Payment successfully authorized and sent to backend!');

  } catch (error) {
    // The SDK will throw specific, catchable errors.
    if (error.name === 'UserCancelledError') {
      console.log('User cancelled the approval.');
    } else if (error.name === 'SecurityKeyError') {
      console.error('No security key was available.');
    } else {
      console.error('An unexpected error occurred:', error);
    }
  }
}
```

### 2.1. The Proof Object Structure

The proof returned by `requestProof()` is a simple JSON object designed for easy storage and transmission:

```typescript
// Example proof object
const proof = {
  "context": {
    "action": "approve-payment",
    "amount": 5000,
    "currency": "USD", 
    "recipient": "ACME-CORP-123",
    "timestamp": "2025-07-14T12:30:00.000Z"
  },
  "signature": "base64-encoded-signature-string...",
  "publicKey": "base64-encoded-public-key...",
  "algorithm": "Ed25519",
  "identityToken": "optional-jwt-from-config..."
};

// The proof is self-contained and can be:
// ✅ Stored in databases
// ✅ Transmitted over HTTP
// ✅ Logged for audit trails
// ✅ Cached for offline verification
// ✅ Serialized to JSON strings

// Example: Store proof in database
await db.transactions.create({
  id: generateId(),
  amount: 5000,
  recipient: "ACME-CORP-123",
  authorization_proof: JSON.stringify(proof), // Simple JSON storage
  created_at: new Date()
});

// Example: Send proof in HTTP request
const response = await fetch('/api/execute-payment', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ proof }) // Direct serialization
});
```
```

### 3. Verify Proof (Server-Side)

```typescript
// Backend API endpoint (e.g., /api/execute-payment)
app.post('/api/execute-payment', async (req, res) => {
  try {
    const { proof } = req.body;
    
    // The proof object contains everything needed for verification:
    // {
    //   "context": { "action": "approve-payment", "amount": 5000, ... },
    //   "signature": "base64-encoded-signature...",
    //   "publicKey": "base64-encoded-public-key...",
    //   "algorithm": "Ed25519",
    //   "identityToken": "optional-jwt..."
    // }
    
    // Verify the cryptographic proof
    const verificationResult = await proofMessenger.verifyProof(proof);
    
    if (verificationResult.valid) {
      console.log("✅ Proof is valid");
      console.log("Context:", verificationResult.context);
      console.log("User ID:", verificationResult.userId);
      
      // Extract the signed context (same as proof.context, but verified)
      const { action, amount, currency, recipient } = verificationResult.context;
      
      // Validate business logic
      if (action !== 'approve-payment') {
        return res.status(400).json({ error: 'Invalid action in proof' });
      }
      
      // Execute the payment with verified context
      const result = await processPayment({
        amount,
        currency,
        recipient,
        proofId: verificationResult.proofId
      });
      
      res.json({ success: true, transactionId: result.id });
      
    } else {
      console.log("❌ Proof verification failed");
      res.status(400).json({ error: 'Invalid proof' });
    }
    
  } catch (error) {
    console.error('Verification error:', error);
    res.status(500).json({ error: 'Verification failed' });
  }
});
```

### 3.1. Proof Verification Details

```typescript
// The verifyProof() method validates:
// 1. Signature authenticity using the provided public key
// 2. Context integrity (data hasn't been tampered with)
// 3. Identity token validity (if provided)
// 4. Timestamp freshness (configurable)

const verificationResult = await proofMessenger.verifyProof({
  "context": { "action": "approve-payment", "amount": 5000, ... },
  "signature": "MEUCIQDx...",
  "publicKey": "MCowBQYD...",
  "algorithm": "Ed25519",
  "identityToken": "eyJhbGciOiJIUzI1NiIs..."
});

// Returns:
// {
//   valid: true,
//   context: { "action": "approve-payment", "amount": 5000, ... }, // Verified context
//   userId: "user123",           // Extracted from identity token
//   publicKey: "MCowBQYD...",    // User's public key
//   algorithm: "Ed25519",        // Signature algorithm used
//   verifiedAt: 1642781400000,   // Verification timestamp
//   proofId: "proof_abc123"      // Generated unique identifier
// }

// Example: Store verified proof for audit
await auditLog.create({
  proof_id: verificationResult.proofId,
  user_id: verificationResult.userId,
  action: verificationResult.context.action,
  context: verificationResult.context,
  verified_at: verificationResult.verifiedAt,
  original_proof: proof // Store original proof object
});
```
```

### 4. Batch Operations

```typescript
// Request proof for multiple related actions
async function onApprovePayroll(payrollData) {
  try {
    // Define batch context - all payroll transfers in one proof
    const context = {
      action: 'approve-payroll-batch',
      pay_period: '2024-01',
      total_amount: payrollData.totalAmount,
      employee_count: payrollData.employees.length,
      transfers: payrollData.employees.map(emp => ({
        employee_id: emp.id,
        amount: emp.salary,
        account: emp.account
      })),
      timestamp: new Date().toISOString()
    };

    // Single proof covers all transfers
    const proof = await proofMessenger.requestProof({ context });

    // Send to backend for batch processing
    await fetch('/api/execute-payroll', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    console.log('Payroll batch successfully authorized!');

  } catch (error) {
    if (error.name === 'UserCancelledError') {
      console.log('Payroll approval cancelled.');
    } else {
      console.error('Payroll approval failed:', error);
    }
  }
}
```

### 5. Real-Time Status Updates

```typescript
// Listen for real-time approval status updates
const approvalRequest = proofMessenger.requestApproval({
  title: "Contract Signature",
  context: { contract_id: "CONTRACT-2024-001" }
});

// Handle status updates
approvalRequest.onStatusChange((status) => {
  switch (status.type) {
    case 'sent':
      console.log("📱 Approval request sent to user's device");
      break;
    case 'viewed':
      console.log("👀 User is viewing the approval request");
      break;
    case 'approved':
      console.log("✅ User approved the request");
      break;
    case 'denied':
      console.log("❌ User denied the request");
      break;
    case 'timeout':
      console.log("⏰ Request timed out");
      break;
  }
});

// Wait for final result
const result = await approvalRequest.result;
```

### 6. Error Handling

```typescript
async function handleTransactionApproval(transactionDetails) {
  try {
    const context = {
      action: 'approve-transaction',
      amount: transactionDetails.amount,
      timestamp: new Date().toISOString()
    };
    
    const proof = await proofMessenger.requestProof({ context });
    
    // Send to backend
    await submitTransaction(proof);
    
  } catch (error) {
    // Handle specific error types
    switch (error.name) {
      case 'UserCancelledError':
        showMessage('Transaction cancelled by user');
        break;
        
      case 'SecurityKeyError':
        showMessage('Security key not available. Please ensure your device supports WebAuthn.');
        break;
        
      case 'NetworkError':
        showMessage('Network error. Please check your connection and try again.');
        break;
        
      case 'RelayServerError':
        showMessage('Authorization service temporarily unavailable. Please try again later.');
        break;
        
      case 'UserNotAuthenticatedError':
        // Redirect to login
        window.location.href = '/login';
        break;
        
      default:
        console.error('Unexpected error:', error);
        showMessage('An unexpected error occurred. Please try again.');
    }
  }
}
```

## � **Working with Proof Objects**

### Proof Object Properties

```typescript
interface CryptographicProof {
  // The exact context data that was signed
  context: Record<string, any>;
  
  // Base64-encoded cryptographic signature
  signature: string;
  
  // Base64-encoded public key used for signing
  publicKey: string;
  
  // Signature algorithm (e.g., "Ed25519", "ECDSA-P256")
  algorithm: string;
  
  // Optional identity token from getIdentityToken config
  identityToken?: string;
}
```

### Storage Examples

```typescript
// 1. Database Storage (PostgreSQL)
CREATE TABLE transaction_proofs (
  id SERIAL PRIMARY KEY,
  transaction_id VARCHAR(255),
  proof_context JSONB,           -- Store context as JSONB for querying
  proof_signature TEXT,          -- Base64 signature
  proof_public_key TEXT,         -- Base64 public key
  proof_algorithm VARCHAR(50),   -- Algorithm name
  identity_token TEXT,           -- Optional JWT
  created_at TIMESTAMP DEFAULT NOW()
);

// Insert proof
await db.query(`
  INSERT INTO transaction_proofs 
  (transaction_id, proof_context, proof_signature, proof_public_key, proof_algorithm, identity_token)
  VALUES ($1, $2, $3, $4, $5, $6)
`, [
  transactionId,
  JSON.stringify(proof.context),
  proof.signature,
  proof.publicKey,
  proof.algorithm,
  proof.identityToken
]);

// Query proofs by action
const paymentProofs = await db.query(`
  SELECT * FROM transaction_proofs 
  WHERE proof_context->>'action' = 'approve-payment'
  AND created_at > NOW() - INTERVAL '30 days'
`);
```

### Serialization Examples

```typescript
// 1. JSON Serialization (for HTTP, storage, etc.)
const proofJson = JSON.stringify(proof);
const parsedProof = JSON.parse(proofJson);

// 2. Base64 Encoding (for URLs, compact storage)
const proofBase64 = btoa(JSON.stringify(proof));
const decodedProof = JSON.parse(atob(proofBase64));

// 3. Compression (for large contexts)
import { gzip, ungzip } from 'pako';

const compressed = gzip(JSON.stringify(proof));
const decompressed = JSON.parse(ungzip(compressed, { to: 'string' }));

// 4. Secure Storage (encrypted at rest)
import { encrypt, decrypt } from './crypto-utils';

const encryptedProof = encrypt(JSON.stringify(proof), secretKey);
const decryptedProof = JSON.parse(decrypt(encryptedProof, secretKey));
```

### Validation Examples

```typescript
// Client-side proof validation (before sending to server)
function validateProofStructure(proof: any): proof is CryptographicProof {
  return (
    typeof proof === 'object' &&
    proof !== null &&
    typeof proof.context === 'object' &&
    typeof proof.signature === 'string' &&
    typeof proof.publicKey === 'string' &&
    typeof proof.algorithm === 'string' &&
    (proof.identityToken === undefined || typeof proof.identityToken === 'string')
  );
}

// Business logic validation
function validatePaymentProof(proof: CryptographicProof): boolean {
  const { context } = proof;
  
  return (
    context.action === 'approve-payment' &&
    typeof context.amount === 'number' &&
    context.amount > 0 &&
    typeof context.currency === 'string' &&
    typeof context.recipient === 'string' &&
    typeof context.timestamp === 'string'
  );
}

// Usage
try {
  const proof = await proofMessenger.requestProof({ context });
  
  if (!validateProofStructure(proof)) {
    throw new Error('Invalid proof structure');
  }
  
  if (!validatePaymentProof(proof)) {
    throw new Error('Invalid payment proof data');
  }
  
  // Safe to send to server
  await sendToServer(proof);
} catch (error) {
  console.error('Proof validation failed:', error);
}
```

### Audit Trail Examples

```typescript
// Create comprehensive audit trail
async function createAuditTrail(proof: CryptographicProof, result: any) {
  const auditEntry = {
    // Proof metadata
    proof_id: generateProofId(proof),
    signature: proof.signature,
    public_key: proof.publicKey,
    algorithm: proof.algorithm,
    
    // Context data
    action: proof.context.action,
    context_data: proof.context,
    
    // Identity information
    user_id: extractUserIdFromToken(proof.identityToken),
    identity_token: proof.identityToken,
    
    // Execution results
    execution_result: result,
    executed_at: new Date().toISOString(),
    
    // Compliance metadata
    ip_address: getClientIP(),
    user_agent: getUserAgent(),
    session_id: getSessionId()
  };
  
  await auditLog.create(auditEntry);
}

// Query audit trail
async function getAuditTrail(filters: {
  userId?: string;
  action?: string;
  dateRange?: { start: Date; end: Date };
}) {
  const query = `
    SELECT 
      proof_id,
      action,
      context_data,
      user_id,
      executed_at,
      execution_result
    FROM audit_trail 
    WHERE 1=1
    ${filters.userId ? 'AND user_id = $1' : ''}
    ${filters.action ? 'AND action = $2' : ''}
    ${filters.dateRange ? 'AND executed_at BETWEEN $3 AND $4' : ''}
    ORDER BY executed_at DESC
  `;
  
  return await db.query(query, [
    filters.userId,
    filters.action,
    filters.dateRange?.start,
    filters.dateRange?.end
  ].filter(Boolean));
}
```

## �🔧 **Advanced Features**

### 7. Custom Validation Rules

```typescript
// Define custom validation for your business logic
const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.my-company.com/verify',
  
  // Custom validation rules
  validators: {
    // Validate financial transactions
    wire_transfer: (context) => {
      if (context.amount_usd_cents > 100000000) { // $1M limit
        throw new ValidationError("Amount exceeds daily limit");
      }
      
      if (!context.destination_account.match(/^[A-Z0-9-]+$/)) {
        throw new ValidationError("Invalid account format");
      }
      
      return true;
    },
    
    // Validate contract signatures
    contract_signature: (context) => {
      if (!context.contract_id || !context.signer_role) {
        throw new ValidationError("Missing required contract fields");
      }
      
      return true;
    }
  }
});
```

### 8. Audit and Compliance

```typescript
// Get audit trail for compliance reporting
const auditTrail = await proofMessenger.getAuditTrail({
  startDate: new Date('2024-01-01'),
  endDate: new Date('2024-01-31'),
  userId: 'user123',
  transactionTypes: ['wire_transfer', 'contract_signature'],
  limit: 100
});

auditTrail.entries.forEach(entry => {
  console.log(`${entry.timestamp}: ${entry.action} by ${entry.userId}`);
  console.log(`Status: ${entry.status}, Proof ID: ${entry.proofId}`);
});

// Export audit data for compliance
const complianceReport = await proofMessenger.exportAuditReport({
  format: 'csv', // or 'json', 'pdf'
  dateRange: { start: '2024-01-01', end: '2024-01-31' },
  includeProofDetails: true
});
```

### 9. Integration Helpers

```typescript
// React Hook for easy integration
import { useProofMessenger } from '@proof-messenger/react';

function TransferButton({ amount, destination }) {
  const { requestApproval, isLoading } = useProofMessenger();
  
  const handleTransfer = async () => {
    const result = await requestApproval({
      title: "Wire Transfer",
      context: {
        action: "wire_transfer",
        amount_usd_cents: amount * 100,
        destination_account: destination
      }
    });
    
    if (result.approved) {
      // Execute transfer with proof
      await executeTransfer(result.proof);
    }
  };
  
  return (
    <button onClick={handleTransfer} disabled={isLoading}>
      {isLoading ? 'Requesting Approval...' : 'Transfer Funds'}
    </button>
  );
}
```

### 10. Testing and Development

```typescript
// Mock client for testing
import { createMockProofMessengerClient } from '@proof-messenger/sdk/testing';

const mockClient = createMockProofMessengerClient({
  // Always approve in tests
  autoApprove: true,
  
  // Simulate network delays
  networkDelay: 100,
  
  // Custom mock responses
  mockResponses: {
    wire_transfer: { approved: true, proofId: 'test-proof-123' },
    contract_signature: { approved: false, reason: 'user_denied' }
  }
});

// Use in tests
describe('Payment Flow', () => {
  it('should handle successful approval', async () => {
    const result = await mockClient.requestApproval({
      title: "Test Transfer",
      context: { action: "wire_transfer", amount_usd_cents: 100000 }
    });
    
    expect(result.approved).toBe(true);
    expect(result.proofId).toBe('test-proof-123');
  });
});
```

## 📱 **Platform-Specific Features**

### Web Browser

```typescript
// Browser-specific features
const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.my-company.com/verify',
  
  // Use WebAuthn for hardware-backed security
  preferWebAuthn: true,
  
  // Show native browser notifications
  enableNotifications: true,
  
  // Integrate with browser's credential manager
  useCredentialManager: true
});
```

### React Native

```typescript
// React Native specific features
import { createProofMessengerClient } from '@proof-messenger/react-native';

const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.my-company.com/verify',
  
  // Use device biometrics (Touch ID, Face ID)
  enableBiometrics: true,
  
  // Use secure keychain storage
  useSecureStorage: true,
  
  // Push notification configuration
  pushNotifications: {
    senderId: 'your-fcm-sender-id',
    topic: 'proof-approvals'
  }
});
```

### Node.js Server

```typescript
// Server-side usage
import { createProofMessengerClient } from '@proof-messenger/node';

const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.internal.company.com/verify',
  
  // Server-to-server authentication
  apiKey: process.env.PROOF_MESSENGER_API_KEY,
  
  // Use for verification only (no user interaction)
  serverMode: true
});

// Verify proofs received from clients
app.post('/api/execute-transfer', async (req, res) => {
  const { proof, transferDetails } = req.body;
  
  const verification = await proofMessenger.verifyProof({
    proof,
    expectedContext: transferDetails
  });
  
  if (verification.valid) {
    await executeTransfer(transferDetails);
    res.json({ success: true });
  } else {
    res.status(400).json({ error: 'Invalid proof' });
  }
});
```

## 🎨 **UI Components**

### Pre-built React Components

```typescript
import { 
  ApprovalButton, 
  ApprovalModal, 
  ApprovalStatus,
  AuditTrail 
} from '@proof-messenger/react-components';

function PaymentForm() {
  return (
    <div>
      <ApprovalButton
        title="Approve Payment"
        context={{
          action: "payment",
          amount_usd_cents: 50000,
          recipient: "ACME Corp"
        }}
        onApproved={(proof) => {
          console.log('Payment approved:', proof);
          executePayment(proof);
        }}
        onDenied={(reason) => {
          console.log('Payment denied:', reason);
        }}
        theme={{
          primaryColor: "#007bff",
          borderRadius: "8px"
        }}
      />
      
      <ApprovalStatus 
        requestId="req-123"
        showProgress={true}
        onStatusChange={(status) => {
          console.log('Status changed:', status);
        }}
      />
    </div>
  );
}
```

## 📊 **Configuration Options**

```typescript
interface ProofMessengerConfig {
  // Required: Your self-hosted relay server URL
  relayUrl: string;
  
  // Optional: Function to get user's identity token
  getIdentityToken?: () => Promise<string>;
  
  // Optional: Custom error handler
  onError?: (error: ProofMessengerError) => void;
  
  // Optional: Development mode settings
  development?: boolean;
  
  // Optional: Request timeout (default: 5 minutes)
  defaultTimeoutMs?: number;
  
  // Optional: Retry configuration
  retryConfig?: {
    maxRetries: number;
    backoffMs: number;
  };
  
  // Optional: Custom validation rules
  validators?: Record<string, (context: any) => boolean>;
  
  // Optional: UI theme customization
  theme?: {
    primaryColor?: string;
    secondaryColor?: string;
    fontFamily?: string;
    borderRadius?: string;
    logo?: string;
  };
  
  // Optional: Platform-specific settings
  platform?: {
    // Web browser settings
    preferWebAuthn?: boolean;
    enableNotifications?: boolean;
    useCredentialManager?: boolean;
    
    // Mobile settings
    enableBiometrics?: boolean;
    useSecureStorage?: boolean;
    pushNotifications?: {
      senderId: string;
      topic: string;
    };
    
    // Server settings
    apiKey?: string;
    serverMode?: boolean;
  };
}
```

## 🔍 **Type Definitions**

```typescript
// Core types
interface ProofRequest {
  context: Record<string, any>;     // The data to be cryptographically signed
  timeoutMs?: number;              // Optional: Request timeout (default: 5min)
  requireBiometric?: boolean;       // Optional: Force biometric authentication
}

interface CryptographicProof {
  context: Record<string, any>;    // The signed context data
  signature: string;               // Base64-encoded signature
  publicKey: string;               // Base64-encoded public key
  algorithm: string;               // Signature algorithm (e.g., "Ed25519")
  identityToken?: string;          // Optional JWT from getIdentityToken config
}

interface VerificationResult {
  valid: boolean;                  // Whether the proof is cryptographically valid
  proofId: string;                 // Unique proof identifier
  context: Record<string, any>;    // The verified context data
  userId?: string;                 // User who created the proof
  verifiedAt: number;              // Verification timestamp
  error?: string;                  // Error message if verification failed
}

interface AuditTrailEntry {
  proofId: string;
  timestamp: number;
  userId: string;
  context: Record<string, any>;
  status: 'created' | 'verified' | 'rejected';
}

// Error types with specific names for easy catching
class UserCancelledError extends Error {
  name = 'UserCancelledError';
}

class SecurityKeyError extends Error {
  name = 'SecurityKeyError';
}

class NetworkError extends Error {
  name = 'NetworkError';
}

class RelayServerError extends Error {
  name = 'RelayServerError';
}

class UserNotAuthenticatedError extends Error {
  name = 'UserNotAuthenticatedError';
}
```

## 🚀 **Getting Started Example**

```typescript
// Complete example: Adding Proof-Messenger to an existing app
import { createProofMessengerClient } from '@proof-messenger/sdk';

// 1. Initialize the client
const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.my-company.com/verify',
  getIdentityToken: async () => {
    return localStorage.getItem('auth_token');
  }
});

// 2. Add to your existing business logic
async function transferFunds(amount, destination) {
  try {
    // Define the transaction context
    const context = {
      action: 'transfer-funds',
      amount_usd_cents: amount,
      destination_account: destination,
      timestamp: new Date().toISOString()
    };

    // Request cryptographic proof from user
    const proof = await proofMessenger.requestProof({ context });

    // Send proof to backend for verification and execution
    const response = await fetch('/api/transfer', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    if (!response.ok) {
      throw new Error('Transfer failed');
    }

    const result = await response.json();
    return result;

  } catch (error) {
    if (error.name === 'UserCancelledError') {
      throw new Error('Transfer cancelled by user');
    } else if (error.name === 'SecurityKeyError') {
      throw new Error('Security key not available');
    } else {
      throw error;
    }
  }
}

// 3. Use in your UI
document.getElementById('transfer-btn').addEventListener('click', async () => {
  try {
    const result = await transferFunds(50000, 'ACME-CORP-123');
    showSuccess(`Transfer completed! ID: ${result.transactionId}`);
  } catch (error) {
    showError('Transfer failed: ' + error.message);
  }
});
```

## 📦 **Package Structure**

```
@proof-messenger/
├── sdk/                    # Core SDK
├── react/                  # React hooks and utilities
├── react-components/       # Pre-built React components
├── react-native/          # React Native specific features
├── node/                  # Node.js server-side utilities
├── vue/                   # Vue.js integration
├── angular/               # Angular integration
└── testing/               # Testing utilities and mocks
```

This SDK design makes it incredibly simple for developers to add strong authorization to their applications while abstracting away all the cryptographic complexity. The API is intuitive, type-safe, and follows modern JavaScript/TypeScript best practices.

---

**Design Version**: 1.0  
**Target Implementation**: Q1 2025  
**Estimated Integration Time**: < 1 hour for basic use cases