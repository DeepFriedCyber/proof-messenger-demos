# Proof-Messenger SDK Implementation Example

## 🎯 **Real-World Integration: Banking Application**

This example shows how a developer would integrate Proof-Messenger into an existing banking application in **under 1 hour**.

## 📋 **Before: Traditional Authorization**

```typescript
// Traditional approach - vulnerable to session hijacking, CSRF, etc.
async function transferFunds(amount: number, destination: string) {
  // Only session-based authorization
  if (!isUserLoggedIn()) {
    throw new Error('User not authenticated');
  }
  
  // No cryptographic proof of user intent
  const result = await bankAPI.transfer({
    amount,
    destination,
    sessionToken: getSessionToken() // Vulnerable to hijacking
  });
  
  return result;
}
```

## ✅ **After: Proof-Messenger Integration**

### Step 1: Install and Initialize (5 minutes)

```bash
# Install the SDK
npm install @proof-messenger/sdk @proof-messenger/react-components
```

```typescript
// config/proof-messenger.ts
import { createProofMessengerClient } from '@proof-messenger/sdk';

export const proofMessenger = createProofMessengerClient({
  // Your self-hosted relay server
  relayUrl: 'https://proof-relay.mybank.com/verify',
  
  // Integrate with existing auth system
  getIdentityToken: async () => {
    const session = await getSession();
    return session.jwt;
  },
  
  // Custom error handling
  onError: (error) => {
    console.error('Proof-Messenger error:', error);
    analytics.track('proof_messenger_error', { error: error.code });
  },
  
  // Bank branding
  theme: {
    primaryColor: "#1a365d",
    logo: "https://mybank.com/logo.png",
    fontFamily: "Inter, sans-serif"
  }
});
```

### Step 2: Update Business Logic (15 minutes)

```typescript
// services/transfer-service.ts
import { proofMessenger } from '../config/proof-messenger';

export async function transferFunds(
  amount: number, 
  destination: string,
  reference?: string
) {
  try {
    // Define the transaction context that will be cryptographically signed
    const context = {
      action: "wire_transfer",
      amount_usd_cents: amount,
      destination_account: destination,
      reference: reference || "",
      timestamp: new Date().toISOString(),
      user_id: getCurrentUserId(),
      session_id: getSessionId()
    };

    // Request cryptographic proof from user
    // This will show browser security prompt (Touch ID, Windows Hello, etc.)
    const proof = await proofMessenger.requestProof({ 
      context,
      requireBiometric: amount > 1000000, // $10,000+ requires biometric
      timeoutMs: 300000 // 5-minute timeout
    });

    // The proof object is a simple JSON structure:
    // {
    //   "context": {
    //     "action": "wire_transfer",
    //     "amount_usd_cents": 5000000,
    //     "destination_account": "ACME-CORP-12345",
    //     "reference": "Invoice #INV-2024-001",
    //     "timestamp": "2025-07-14T12:30:00.000Z",
    //     "user_id": "user123",
    //     "session_id": "sess_abc123"
    //   },
    //   "signature": "MEUCIQDxK8...",
    //   "publicKey": "MCowBQYDK2V...",
    //   "algorithm": "Ed25519",
    //   "identityToken": "eyJhbGciOiJIUzI1NiIs..."
    // }

    // Send proof to backend for verification and execution
    const response = await fetch('/api/execute-transfer', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Transfer failed');
    }

    const result = await response.json();

    // Log successful transfer with proof details
    auditLogger.log({
      event: 'wire_transfer_executed',
      user_id: getCurrentUserId(),
      amount,
      destination,
      proof_id: result.proofId,
      proof_signature: proof.signature,
      proof_algorithm: proof.algorithm,
      original_context: proof.context,
      timestamp: Date.now()
    });

    return result;

  } catch (error) {
    // Handle specific error types
    if (error.name === 'UserCancelledError') {
      throw new Error('Transfer cancelled by user');
    } else if (error.name === 'SecurityKeyError') {
      throw new Error('Security key not available. Please ensure your device supports biometric authentication.');
    } else if (error.name === 'NetworkError') {
      throw new Error('Network error. Please check your connection and try again.');
    } else {
      throw error;
    }
  }
}
```

### Step 3: Update UI Components (20 minutes)

```typescript
// components/TransferForm.tsx
import React, { useState } from 'react';
import { ApprovalButton } from '@proof-messenger/react-components';
import { transferFunds } from '../services/transfer-service';

export function TransferForm() {
  const [amount, setAmount] = useState('');
  const [destination, setDestination] = useState('');
  const [reference, setReference] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);

  const handleTransfer = async () => {
    if (!amount || !destination) return;
    
    setIsProcessing(true);
    
    try {
      const result = await transferFunds(
        parseFloat(amount) * 100, // Convert to cents
        destination,
        reference
      );
      
      // Show success message
      toast.success(`Transfer completed! Transaction ID: ${result.transactionId}`);
      
      // Reset form
      setAmount('');
      setDestination('');
      setReference('');
      
    } catch (error) {
      if (error.message.includes('cancelled by user')) {
        toast.warning('Transfer cancelled by user');
      } else {
        toast.error(`Transfer failed: ${error.message}`);
      }
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="transfer-form">
      <h2>Wire Transfer</h2>
      
      <div className="form-group">
        <label>Amount ($)</label>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.00"
          min="0.01"
          step="0.01"
        />
      </div>
      
      <div className="form-group">
        <label>Destination Account</label>
        <input
          type="text"
          value={destination}
          onChange={(e) => setDestination(e.target.value)}
          placeholder="ACME-CORP-12345"
        />
      </div>
      
      <div className="form-group">
        <label>Reference (Optional)</label>
        <input
          type="text"
          value={reference}
          onChange={(e) => setReference(e.target.value)}
          placeholder="Invoice #INV-2024-001"
        />
      </div>
      
      {/* Transfer button with built-in proof request */}
      <button
        onClick={handleTransfer}
        disabled={!amount || !destination || isProcessing}
        className="transfer-button"
        style={{
          backgroundColor: "#1a365d",
          color: "white",
          padding: "12px 24px",
          borderRadius: "8px",
          border: "none",
          fontSize: "16px",
          cursor: isProcessing ? "not-allowed" : "pointer",
          opacity: (!amount || !destination || isProcessing) ? 0.6 : 1
        }}
      >
        {isProcessing ? 'Processing...' : `Transfer $${amount || '0.00'}`}
      </button>
    </div>
  );
}
```

### Step 4: Add Server-Side Verification (15 minutes)

```typescript
// api/transfer.ts (Next.js API route example)
import { createProofMessengerClient } from '@proof-messenger/node';
import { executeTransfer } from '../services/bank-api';

const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.mybank.com/verify',
  apiKey: process.env.PROOF_MESSENGER_API_KEY,
  serverMode: true
});

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }
  
  const { proof } = req.body;
  
  try {
    // Verify the cryptographic proof
    const verification = await proofMessenger.verifyProof(proof);
    
    if (!verification.valid) {
      return res.status(400).json({ 
        error: 'Invalid proof',
        details: verification.error 
      });
    }
    
    // Extract verified context data
    const { action, amount_usd_cents, destination_account, reference } = verification.context;
    
    // Validate the action type
    if (action !== 'wire_transfer') {
      return res.status(400).json({ error: 'Invalid action in proof' });
    }
    
    // Store the proof for audit trail before executing transfer
    await storeProofForAudit({
      proof_id: verification.proofId,
      original_proof: proof,
      verified_context: verification.context,
      user_id: verification.userId,
      verified_at: verification.verifiedAt
    });

    // Execute the transfer with verified data
    const result = await executeTransfer({
      amount: amount_usd_cents,
      destination: destination_account,
      reference: reference,
      proof_id: verification.proofId,
      verified_at: verification.verifiedAt,
      user_id: verification.userId
    });
    
    res.json({
      success: true,
      transactionId: result.transactionId,
      proofId: verification.proofId
    });
    
  } catch (error) {
    console.error('Transfer error:', error);
    res.status(500).json({ 
      error: 'Transfer failed',
      message: error.message 
    });
  }
}

// Helper function to store proofs for audit trail
async function storeProofForAudit(auditData) {
  const { proof_id, original_proof, verified_context, user_id, verified_at } = auditData;
  
  // Store in audit database
  await db.audit_trail.create({
    proof_id,
    user_id,
    action: verified_context.action,
    
    // Store the complete proof object as JSON
    proof_data: {
      context: original_proof.context,
      signature: original_proof.signature,
      publicKey: original_proof.publicKey,
      algorithm: original_proof.algorithm,
      identityToken: original_proof.identityToken
    },
    
    // Store verified context separately for easy querying
    verified_context,
    verified_at,
    created_at: new Date()
  });
  
  console.log(`Proof ${proof_id} stored for audit trail`);
}
```

### Step 5: Add Audit Dashboard (5 minutes)

```typescript
// components/AuditDashboard.tsx
import React, { useEffect, useState } from 'react';
import { AuditTrail } from '@proof-messenger/react-components';
import { proofMessenger } from '../config/proof-messenger';

export function AuditDashboard() {
  const [auditData, setAuditData] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAuditData();
  }, []);

  const loadAuditData = async () => {
    try {
      const trail = await proofMessenger.getAuditTrail({
        startDate: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000), // Last 30 days
        endDate: new Date(),
        transactionTypes: ['wire_transfer'],
        limit: 100
      });
      
      setAuditData(trail.entries);
    } catch (error) {
      console.error('Failed to load audit data:', error);
    } finally {
      setLoading(false);
    }
  };

  const exportReport = async () => {
    const report = await proofMessenger.exportAuditReport({
      format: 'csv',
      dateRange: { 
        start: '2024-01-01', 
        end: new Date().toISOString().split('T')[0] 
      },
      includeProofDetails: true
    });
    
    // Download the report
    const blob = new Blob([report], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'transfer-audit-report.csv';
    a.click();
  };

  if (loading) {
    return <div>Loading audit data...</div>;
  }

  return (
    <div className="audit-dashboard">
      <div className="header">
        <h2>Transfer Audit Trail</h2>
        <button onClick={exportReport} className="export-btn">
          Export Report
        </button>
      </div>
      
      <AuditTrail
        entries={auditData}
        columns={[
          'timestamp',
          'userId', 
          'action',
          'amount',
          'destination',
          'status',
          'proofId'
        ]}
        onEntryClick={(entry) => {
          // Show detailed proof information
          console.log('Proof details:', entry);
        }}
        theme={{
          headerColor: "#f8f9fa",
          rowHoverColor: "#e9ecef"
        }}
      />
    </div>
  );
}
```

## 🎯 **Integration Results**

### ✅ **What We Achieved in Under 1 Hour**

1. **🔒 Cryptographic Authorization**: Every transfer now requires cryptographic proof of user intent
2. **🚫 Non-Repudiation**: Users cannot deny approving specific transactions
3. **📱 Biometric Security**: High-value transfers require biometric confirmation
4. **📊 Complete Audit Trail**: Every action is cryptographically logged
5. **🎨 Branded Experience**: Approval UI matches bank's branding
6. **⚡ Real-Time Status**: Users see approval status in real-time
7. **🛡️ Server Verification**: Backend validates all proofs before execution

### 📊 **Security Improvements**

| Before | After |
|--------|-------|
| ❌ Session-based auth only | ✅ Cryptographic proof required |
| ❌ Vulnerable to CSRF/hijacking | ✅ Context-bound signatures |
| ❌ No non-repudiation | ✅ Cryptographic non-repudiation |
| ❌ Basic audit logs | ✅ Cryptographic audit trail |
| ❌ No user intent verification | ✅ Explicit transaction approval |

### 🚀 **Developer Experience**

```typescript
// Before: Complex, error-prone
if (session.isValid() && csrf.verify() && user.hasPermission()) {
  // Hope this is what the user intended...
  await executeTransfer();
}

// After: Simple, secure
const approval = await proofMessenger.requestApproval({
  title: "Transfer Authorization",
  context: transferDetails
});

if (approval.approved) {
  // Cryptographically proven user intent
  await executeTransfer(approval.proof);
}
```

## 🔧 **Advanced Features (Optional)**

### Custom Validation Rules

```typescript
// Add business-specific validation
const proofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.mybank.com/verify',
  
  validators: {
    wire_transfer: (context) => {
      // Daily limit check
      if (context.amount_usd_cents > getDailyLimit(context.user_id)) {
        throw new ValidationError("Exceeds daily transfer limit");
      }
      
      // Blocked accounts check
      if (isAccountBlocked(context.destination_account)) {
        throw new ValidationError("Destination account is blocked");
      }
      
      // Business hours check
      if (!isBusinessHours()) {
        throw new ValidationError("Transfers only allowed during business hours");
      }
      
      return true;
    }
  }
});
```

### Batch Operations

```typescript
// Approve multiple transfers at once
const batchApproval = await proofMessenger.requestBatchApproval({
  title: "Payroll Batch Authorization",
  description: "Approve payroll for 150 employees",
  
  contexts: payrollTransfers.map(transfer => ({
    action: "payroll_transfer",
    employee_id: transfer.employeeId,
    amount_usd_cents: transfer.amount,
    account: transfer.account
  })),
  
  summary: {
    total_amount_usd_cents: totalPayrollAmount,
    employee_count: payrollTransfers.length,
    pay_period: "2024-01"
  }
});
```

### Real-Time Notifications

```typescript
// Listen for approval status updates
const approvalRequest = proofMessenger.requestApproval({
  title: "Large Transfer Authorization",
  context: transferDetails
});

// Show real-time status to user
approvalRequest.onStatusChange((status) => {
  updateUI({
    message: getStatusMessage(status.type),
    progress: getProgressPercentage(status.type)
  });
});

// Handle final result
const result = await approvalRequest.result;
```

## 📱 **Mobile Integration**

```typescript
// React Native example
import { createProofMessengerClient } from '@proof-messenger/react-native';

const mobileProofMessenger = createProofMessengerClient({
  relayUrl: 'https://proof-relay.mybank.com/verify',
  
  // Use device biometrics
  enableBiometrics: true,
  
  // Secure keychain storage
  useSecureStorage: true,
  
  // Push notifications for approval requests
  pushNotifications: {
    senderId: 'mybank-fcm-sender-id',
    topic: 'transfer-approvals'
  }
});

// Mobile transfer approval
const handleMobileTransfer = async () => {
  const approval = await mobileProofMessenger.requestApproval({
    title: "Mobile Transfer",
    context: transferDetails,
    requireBiometric: true // Always require biometric on mobile
  });
  
  if (approval.approved) {
    await executeTransfer(approval.proof);
    showSuccessNotification();
  }
};
```

## 🎉 **Summary: From Zero to Secure in Under 1 Hour**

This example demonstrates how a developer can transform their application's security posture in minimal time:

1. **⏱️ 5 minutes**: Install and configure SDK
2. **⏱️ 15 minutes**: Update business logic with proof requests
3. **⏱️ 20 minutes**: Add approval UI components
4. **⏱️ 15 minutes**: Implement server-side verification
5. **⏱️ 5 minutes**: Add audit dashboard

**Total: 60 minutes to cryptographic authorization**

The SDK abstracts away all cryptographic complexity while providing enterprise-grade security, making it incredibly easy for developers to add strong authorization to their applications.

---

**Example Version**: 1.0  
**Integration Time**: < 1 hour  
**Security Level**: Enterprise-grade  
**Developer Experience**: ⭐⭐⭐⭐⭐