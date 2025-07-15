# FinTech Multi-Party Approval Demo

## 🎯 **Demo Concept: The "Aha!" Moment**

**Goal**: Demonstrate how Proof-Messenger solves a high-value, high-risk problem that is **impossible to solve with passkeys alone**: a corporate wire transfer requiring sequential, non-repudiable approval from multiple parties.

**The "Aha!" Moment**: The viewer sees a single, cryptographically secure "proof bundle" that contains the immutable history of all approvals, making the transaction audit-proof.

## 👥 **Characters & Roles**

- **Alice (Finance Clerk)**: Can initiate payment requests
- **Bob (Finance Manager)**: Can approve requests up to $10,000
- **Carol (CFO)**: Must co-sign any payment over $10,000

## 🏢 **Setting: ACME Dynamics Payment Portal**

A clean, professional dashboard interface showing:
- Pending payments queue
- Completed transactions
- Approval workflow status
- Cryptographic proof verification

## 📋 **Demo Flow: Step-by-Step Implementation**

### **Step 1: The Initiation (Alice)**

#### **Scene Setup**
```typescript
// Alice's dashboard component
function PaymentDashboard({ user }: { user: User }) {
  const [payments, setPayments] = useState([]);
  const [showNewPayment, setShowNewPayment] = useState(false);

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>ACME Dynamics - Payment Portal</h1>
        <div className="user-info">
          <span>Welcome, {user.name} (Finance Clerk)</span>
          <span className="auth-status">✅ Authenticated with passkey</span>
        </div>
      </header>

      <div className="actions">
        <button 
          onClick={() => setShowNewPayment(true)}
          className="primary-button"
        >
          + New Wire Transfer
        </button>
      </div>

      {showNewPayment && (
        <NewPaymentModal 
          onClose={() => setShowNewPayment(false)}
          onSubmit={handlePaymentInitiation}
        />
      )}

      <PaymentsList payments={payments} />
    </div>
  );
}
```

#### **Payment Form**
```typescript
function NewPaymentModal({ onClose, onSubmit }) {
  const [formData, setFormData] = useState({
    recipient: '',
    amount: '',
    reason: ''
  });

  const handleSubmit = async () => {
    // Validate form
    if (!formData.recipient || !formData.amount || !formData.reason) {
      alert('Please fill in all fields');
      return;
    }

    const amount = parseFloat(formData.amount);
    if (amount <= 0) {
      alert('Amount must be greater than 0');
      return;
    }

    await onSubmit(formData);
    onClose();
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h2>New Wire Transfer</h2>
        
        <div className="form-group">
          <label>Recipient</label>
          <input
            type="text"
            value={formData.recipient}
            onChange={(e) => setFormData({...formData, recipient: e.target.value})}
            placeholder="Quantum Innovations Inc."
          />
        </div>

        <div className="form-group">
          <label>Amount ($)</label>
          <input
            type="number"
            value={formData.amount}
            onChange={(e) => setFormData({...formData, amount: e.target.value})}
            placeholder="25000"
            min="0.01"
            step="0.01"
          />
        </div>

        <div className="form-group">
          <label>Reason</label>
          <input
            type="text"
            value={formData.reason}
            onChange={(e) => setFormData({...formData, reason: e.target.value})}
            placeholder="Q3 Server Infrastructure Purchase"
          />
        </div>

        <div className="modal-actions">
          <button onClick={onClose} className="secondary-button">
            Cancel
          </button>
          <button onClick={handleSubmit} className="primary-button">
            Submit for Approval
          </button>
        </div>
      </div>
    </div>
  );
}
```

#### **Alice's Proof Generation**
```typescript
async function handlePaymentInitiation(paymentData) {
  try {
    // 1. Create the initial payment context
    const context = {
      action: 'initiate-payment',
      payment_id: generatePaymentId(),
      recipient: paymentData.recipient,
      amount_usd_cents: Math.round(parseFloat(paymentData.amount) * 100),
      reason: paymentData.reason,
      initiated_by: 'alice@acmedynamics.com',
      initiated_at: new Date().toISOString(),
      approval_chain: [], // Will be populated as approvals are added
      status: 'pending_manager_approval'
    };

    // 2. Show clear approval modal
    const approvalModal = showApprovalModal({
      title: "Payment Initiation",
      message: `You are initiating a payment of $${paymentData.amount} to ${paymentData.recipient}. Please use your security key to sign this request.`,
      context: context
    });

    // 3. Request cryptographic proof from Alice
    const proof = await proofMessenger.requestProof({ 
      context,
      requireBiometric: true // Always require biometric for financial transactions
    });

    // 4. Submit to backend
    const response = await fetch('/api/payments/initiate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    if (!response.ok) {
      throw new Error('Failed to initiate payment');
    }

    const result = await response.json();

    // 5. Update UI
    showSuccessMessage(`Payment initiated successfully! ID: ${result.paymentId}`);
    refreshPaymentsList();

  } catch (error) {
    if (error.name === 'UserCancelledError') {
      showMessage('Payment initiation cancelled');
    } else {
      showErrorMessage(`Failed to initiate payment: ${error.message}`);
    }
  }
}
```

### **Step 2: The First Approval (Bob)**

#### **Bob's Dashboard View**
```typescript
function ManagerDashboard({ user }: { user: User }) {
  const [pendingApprovals, setPendingApprovals] = useState([]);

  useEffect(() => {
    loadPendingApprovals();
  }, []);

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>ACME Dynamics - Manager Portal</h1>
        <div className="user-info">
          <span>Welcome, {user.name} (Finance Manager)</span>
          <span className="approval-limit">Approval Limit: $10,000</span>
        </div>
      </header>

      <div className="pending-approvals">
        <h2>Pending Approvals ({pendingApprovals.length})</h2>
        {pendingApprovals.map(payment => (
          <PaymentApprovalCard 
            key={payment.id} 
            payment={payment}
            onApprove={handleManagerApproval}
          />
        ))}
      </div>
    </div>
  );
}
```

#### **Payment Approval Card**
```typescript
function PaymentApprovalCard({ payment, onApprove }) {
  const [showDetails, setShowDetails] = useState(false);
  const requiresCFOApproval = payment.amount_usd_cents > 1000000; // $10,000

  return (
    <div className="approval-card">
      <div className="card-header">
        <h3>Payment Request #{payment.id}</h3>
        <span className="amount">${(payment.amount_usd_cents / 100).toLocaleString()}</span>
      </div>

      <div className="card-content">
        <p><strong>To:</strong> {payment.recipient}</p>
        <p><strong>Reason:</strong> {payment.reason}</p>
        <p><strong>Initiated by:</strong> {payment.initiated_by}</p>
        <p><strong>Date:</strong> {new Date(payment.initiated_at).toLocaleString()}</p>

        {requiresCFOApproval && (
          <div className="policy-notice">
            ⚠️ This payment exceeds your $10,000 approval limit and requires additional signature from the CFO.
          </div>
        )}

        <div className="proof-chain">
          <h4>Approval Chain:</h4>
          <div className="proof-item">
            ✅ <strong>Initiated:</strong> Alice (Finance Clerk)
            <span className="proof-id">Proof: {payment.initiation_proof_id}</span>
          </div>
        </div>
      </div>

      <div className="card-actions">
        <button 
          onClick={() => setShowDetails(!showDetails)}
          className="secondary-button"
        >
          {showDetails ? 'Hide' : 'Show'} Details
        </button>
        <button 
          onClick={() => onApprove(payment)}
          className="primary-button"
        >
          {requiresCFOApproval ? 'Approve & Forward to CFO' : 'Approve & Execute'}
        </button>
      </div>

      {showDetails && (
        <div className="proof-details">
          <h4>Cryptographic Proof Details:</h4>
          <pre>{JSON.stringify(payment.initiation_proof, null, 2)}</pre>
        </div>
      )}
    </div>
  );
}
```

#### **Bob's Approval Process**
```typescript
async function handleManagerApproval(payment) {
  try {
    const requiresCFOApproval = payment.amount_usd_cents > 1000000;

    // 1. Create approval context
    const context = {
      action: 'manager-approval',
      payment_id: payment.id,
      original_context: payment.context, // Reference to Alice's original context
      approved_by: 'bob@acmedynamics.com',
      approved_at: new Date().toISOString(),
      approval_level: 'manager',
      next_approval_required: requiresCFOApproval ? 'cfo' : null,
      manager_notes: 'Approved within policy limits'
    };

    // 2. Show clear approval modal
    const message = requiresCFOApproval 
      ? `You are approving a payment of $${(payment.amount_usd_cents / 100).toLocaleString()}. This will be forwarded to Carol (CFO) for final approval. Please use your security key to co-sign.`
      : `You are providing final approval for a payment of $${(payment.amount_usd_cents / 100).toLocaleString()}. Please use your security key to sign.`;

    showApprovalModal({
      title: "Manager Approval",
      message: message,
      context: context
    });

    // 3. Request cryptographic proof from Bob
    const proof = await proofMessenger.requestProof({ 
      context,
      requireBiometric: true
    });

    // 4. Submit approval
    const response = await fetch(`/api/payments/${payment.id}/manager-approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    if (!response.ok) {
      throw new Error('Failed to approve payment');
    }

    const result = await response.json();

    // 5. Update UI
    const nextStep = requiresCFOApproval ? 'forwarded to CFO' : 'executed';
    showSuccessMessage(`Payment approved and ${nextStep}!`);
    refreshPendingApprovals();

  } catch (error) {
    if (error.name === 'UserCancelledError') {
      showMessage('Approval cancelled');
    } else {
      showErrorMessage(`Failed to approve payment: ${error.message}`);
    }
  }
}
```

### **Step 3: The Final Authorization (Carol)**

#### **CFO Dashboard**
```typescript
function CFODashboard({ user }: { user: User }) {
  const [pendingFinalApprovals, setPendingFinalApprovals] = useState([]);

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>ACME Dynamics - CFO Portal</h1>
        <div className="user-info">
          <span>Welcome, {user.name} (Chief Financial Officer)</span>
          <span className="authority-level">Final Authorization Authority</span>
        </div>
      </header>

      <div className="final-approvals">
        <h2>Pending Final Approvals ({pendingFinalApprovals.length})</h2>
        {pendingFinalApprovals.map(payment => (
          <FinalApprovalCard 
            key={payment.id} 
            payment={payment}
            onApprove={handleCFOApproval}
          />
        ))}
      </div>
    </div>
  );
}
```

#### **Final Approval Card**
```typescript
function FinalApprovalCard({ payment, onApprove }) {
  return (
    <div className="final-approval-card">
      <div className="card-header">
        <h3>Final Authorization Required</h3>
        <span className="amount critical">${(payment.amount_usd_cents / 100).toLocaleString()}</span>
      </div>

      <div className="approval-history">
        <h4>Complete Approval History:</h4>
        
        <div className="approval-step completed">
          <div className="step-icon">✅</div>
          <div className="step-content">
            <strong>Initiated by Alice</strong> (Finance Clerk)
            <div className="step-details">
              <span>Date: {new Date(payment.initiated_at).toLocaleString()}</span>
              <span>Proof ID: {payment.initiation_proof_id}</span>
            </div>
          </div>
        </div>

        <div className="approval-step completed">
          <div className="step-icon">✅</div>
          <div className="step-content">
            <strong>Approved by Bob</strong> (Finance Manager)
            <div className="step-details">
              <span>Date: {new Date(payment.manager_approved_at).toLocaleString()}</span>
              <span>Proof ID: {payment.manager_proof_id}</span>
            </div>
          </div>
        </div>

        <div className="approval-step pending">
          <div className="step-icon">⏳</div>
          <div className="step-content">
            <strong>Awaiting CFO Authorization</strong>
            <div className="step-details">
              <span>Final approval required for amounts over $10,000</span>
            </div>
          </div>
        </div>
      </div>

      <div className="transaction-details">
        <h4>Transaction Details:</h4>
        <p><strong>Recipient:</strong> {payment.recipient}</p>
        <p><strong>Amount:</strong> ${(payment.amount_usd_cents / 100).toLocaleString()}</p>
        <p><strong>Reason:</strong> {payment.reason}</p>
      </div>

      <div className="card-actions">
        <button 
          onClick={() => onApprove(payment)}
          className="critical-button"
        >
          Final Approve & Execute
        </button>
      </div>
    </div>
  );
}
```

#### **Carol's Final Authorization**
```typescript
async function handleCFOApproval(payment) {
  try {
    // 1. Create final authorization context
    const context = {
      action: 'cfo-final-authorization',
      payment_id: payment.id,
      original_context: payment.context,
      manager_approval_context: payment.manager_approval_context,
      authorized_by: 'carol@acmedynamics.com',
      authorized_at: new Date().toISOString(),
      authorization_level: 'cfo',
      final_authorization: true,
      cfo_notes: 'Final authorization provided - execute payment'
    };

    // 2. Show critical approval modal
    showApprovalModal({
      title: "FINAL AUTHORIZATION",
      message: `You are providing the final authorization to execute a payment of $${(payment.amount_usd_cents / 100).toLocaleString()}. This action is irreversible. Please use your security key to sign.`,
      context: context,
      critical: true // Special styling for final approvals
    });

    // 3. Request cryptographic proof from Carol
    const proof = await proofMessenger.requestProof({ 
      context,
      requireBiometric: true
    });

    // 4. Submit final authorization
    const response = await fetch(`/api/payments/${payment.id}/cfo-authorize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ proof })
    });

    if (!response.ok) {
      throw new Error('Failed to authorize payment');
    }

    const result = await response.json();

    // 5. Update UI
    showSuccessMessage(`Payment authorized and executed! Transaction ID: ${result.transactionId}`);
    refreshPendingApprovals();

  } catch (error) {
    if (error.name === 'UserCancelledError') {
      showMessage('Authorization cancelled');
    } else {
      showErrorMessage(`Failed to authorize payment: ${error.message}`);
    }
  }
}
```

### **Step 4: The "Aha!" Moment - The Audit Trail**

#### **Completed Transaction View**
```typescript
function CompletedTransactionView({ transactionId }) {
  const [transaction, setTransaction] = useState(null);
  const [proofBundle, setProofBundle] = useState(null);
  const [verificationResult, setVerificationResult] = useState(null);

  useEffect(() => {
    loadTransactionDetails();
  }, [transactionId]);

  const loadTransactionDetails = async () => {
    const response = await fetch(`/api/transactions/${transactionId}`);
    const data = await response.json();
    setTransaction(data.transaction);
    setProofBundle(data.proofBundle);
  };

  const verifyProofChain = async () => {
    const response = await fetch(`/api/transactions/${transactionId}/verify`, {
      method: 'POST'
    });
    const result = await response.json();
    setVerificationResult(result);
  };

  if (!transaction) return <div>Loading...</div>;

  return (
    <div className="transaction-details">
      <header className="transaction-header">
        <h1>Transaction Details</h1>
        <div className="transaction-status executed">
          ✅ EXECUTED
        </div>
      </header>

      <div className="transaction-summary">
        <h2>Transaction Summary</h2>
        <div className="summary-grid">
          <div className="summary-item">
            <label>Recipient</label>
            <span>{transaction.recipient}</span>
          </div>
          <div className="summary-item">
            <label>Amount</label>
            <span className="amount">${(transaction.amount_usd_cents / 100).toLocaleString()}</span>
          </div>
          <div className="summary-item">
            <label>Reason</label>
            <span>{transaction.reason}</span>
          </div>
          <div className="summary-item">
            <label>Executed At</label>
            <span>{new Date(transaction.executed_at).toLocaleString()}</span>
          </div>
        </div>
      </div>

      <div className="proof-bundle-section">
        <h2>🔒 Cryptographic Proof Bundle</h2>
        <p className="proof-explanation">
          This transaction required approval from multiple parties. Each approval created a cryptographic proof 
          that cannot be forged or repudiated. The complete proof chain is shown below.
        </p>

        <div className="proof-chain">
          <div className="proof-step">
            <div className="proof-header">
              <h3>1. Payment Initiation</h3>
              <span className="proof-status verified">✅ VERIFIED</span>
            </div>
            <div className="proof-details">
              <p><strong>Signer:</strong> Alice (Finance Clerk)</p>
              <p><strong>Action:</strong> initiate-payment</p>
              <p><strong>Public Key:</strong> <code>0xABC123...DEF456</code></p>
              <p><strong>Timestamp:</strong> {transaction.initiated_at}</p>
              <div className="proof-context">
                <strong>Signed Context:</strong>
                <pre>{JSON.stringify(proofBundle.initiation.context, null, 2)}</pre>
              </div>
            </div>
          </div>

          <div className="proof-step">
            <div className="proof-header">
              <h3>2. Manager Approval</h3>
              <span className="proof-status verified">✅ VERIFIED</span>
            </div>
            <div className="proof-details">
              <p><strong>Signer:</strong> Bob (Finance Manager)</p>
              <p><strong>Action:</strong> manager-approval</p>
              <p><strong>Public Key:</strong> <code>0xDEF456...GHI789</code></p>
              <p><strong>Timestamp:</strong> {transaction.manager_approved_at}</p>
              <div className="proof-context">
                <strong>Signed Context:</strong>
                <pre>{JSON.stringify(proofBundle.managerApproval.context, null, 2)}</pre>
              </div>
            </div>
          </div>

          <div className="proof-step">
            <div className="proof-header">
              <h3>3. CFO Final Authorization</h3>
              <span className="proof-status verified">✅ VERIFIED</span>
            </div>
            <div className="proof-details">
              <p><strong>Signer:</strong> Carol (CFO)</p>
              <p><strong>Action:</strong> cfo-final-authorization</p>
              <p><strong>Public Key:</strong> <code>0xGHI789...JKL012</code></p>
              <p><strong>Timestamp:</strong> {transaction.cfo_authorized_at}</p>
              <div className="proof-context">
                <strong>Signed Context:</strong>
                <pre>{JSON.stringify(proofBundle.cfoAuthorization.context, null, 2)}</pre>
              </div>
            </div>
          </div>
        </div>

        <div className="verification-section">
          <button 
            onClick={verifyProofChain}
            className="verify-button"
            disabled={!!verificationResult}
          >
            🔍 Verify Entire Chain
          </button>

          {verificationResult && (
            <div className="verification-result">
              <div className="verification-status success">
                ✅ <strong>All signatures are cryptographically valid for this context.</strong>
              </div>
              <div className="verification-details">
                <p>✅ Initiation proof verified: Alice's signature is valid</p>
                <p>✅ Manager approval verified: Bob's signature is valid</p>
                <p>✅ CFO authorization verified: Carol's signature is valid</p>
                <p>✅ Context integrity verified: No data has been tampered with</p>
                <p>✅ Chain of custody verified: All approvals reference correct contexts</p>
              </div>
              <div className="non-repudiation-notice">
                <strong>🛡️ This transaction is non-repudiable.</strong>
                <p>Each party cryptographically signed their specific approval, creating an immutable audit trail that can be verified by any third party.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

## 🔧 **Backend Implementation**

### **Payment Initiation Endpoint**
```typescript
// POST /api/payments/initiate
app.post('/api/payments/initiate', async (req, res) => {
  try {
    const { proof } = req.body;

    // Verify Alice's initiation proof
    const verification = await proofMessenger.verifyProof(proof);
    
    if (!verification.valid) {
      return res.status(400).json({ error: 'Invalid initiation proof' });
    }

    const { context } = verification;
    
    // Validate initiation context
    if (context.action !== 'initiate-payment') {
      return res.status(400).json({ error: 'Invalid action in proof' });
    }

    // Create payment record
    const payment = await db.payments.create({
      id: context.payment_id,
      recipient: context.recipient,
      amount_usd_cents: context.amount_usd_cents,
      reason: context.reason,
      initiated_by: context.initiated_by,
      initiated_at: context.initiated_at,
      status: 'pending_manager_approval',
      
      // Store complete proof
      initiation_proof: proof,
      initiation_proof_id: verification.proofId
    });

    // Notify manager
    await notifyManager(payment);

    res.json({
      success: true,
      paymentId: payment.id,
      status: payment.status
    });

  } catch (error) {
    console.error('Payment initiation error:', error);
    res.status(500).json({ error: 'Failed to initiate payment' });
  }
});
```

### **Manager Approval Endpoint**
```typescript
// POST /api/payments/:id/manager-approve
app.post('/api/payments/:id/manager-approve', async (req, res) => {
  try {
    const { id } = req.params;
    const { proof } = req.body;

    // Get existing payment
    const payment = await db.payments.findById(id);
    if (!payment) {
      return res.status(404).json({ error: 'Payment not found' });
    }

    if (payment.status !== 'pending_manager_approval') {
      return res.status(400).json({ error: 'Payment not in correct state for manager approval' });
    }

    // Verify manager's approval proof
    const verification = await proofMessenger.verifyProof(proof);
    
    if (!verification.valid) {
      return res.status(400).json({ error: 'Invalid manager approval proof' });
    }

    const { context } = verification;
    
    // Validate approval context
    if (context.action !== 'manager-approval' || context.payment_id !== id) {
      return res.status(400).json({ error: 'Invalid approval context' });
    }

    // Determine next step
    const requiresCFOApproval = payment.amount_usd_cents > 1000000;
    const newStatus = requiresCFOApproval ? 'pending_cfo_approval' : 'approved';

    // Update payment
    await db.payments.update(id, {
      status: newStatus,
      manager_approved_by: context.approved_by,
      manager_approved_at: context.approved_at,
      manager_approval_proof: proof,
      manager_proof_id: verification.proofId
    });

    // Execute or notify CFO
    if (requiresCFOApproval) {
      await notifyCFO(payment);
    } else {
      await executePayment(payment);
    }

    res.json({
      success: true,
      status: newStatus,
      requiresCFOApproval
    });

  } catch (error) {
    console.error('Manager approval error:', error);
    res.status(500).json({ error: 'Failed to process manager approval' });
  }
});
```

### **CFO Authorization Endpoint**
```typescript
// POST /api/payments/:id/cfo-authorize
app.post('/api/payments/:id/cfo-authorize', async (req, res) => {
  try {
    const { id } = req.params;
    const { proof } = req.body;

    // Get existing payment
    const payment = await db.payments.findById(id);
    if (!payment || payment.status !== 'pending_cfo_approval') {
      return res.status(400).json({ error: 'Payment not ready for CFO authorization' });
    }

    // Verify CFO's authorization proof
    const verification = await proofMessenger.verifyProof(proof);
    
    if (!verification.valid) {
      return res.status(400).json({ error: 'Invalid CFO authorization proof' });
    }

    const { context } = verification;
    
    // Validate authorization context
    if (context.action !== 'cfo-final-authorization' || context.payment_id !== id) {
      return res.status(400).json({ error: 'Invalid authorization context' });
    }

    // Update payment
    await db.payments.update(id, {
      status: 'approved',
      cfo_authorized_by: context.authorized_by,
      cfo_authorized_at: context.authorized_at,
      cfo_authorization_proof: proof,
      cfo_proof_id: verification.proofId
    });

    // Execute payment
    const executionResult = await executePayment(payment);

    res.json({
      success: true,
      status: 'executed',
      transactionId: executionResult.transactionId
    });

  } catch (error) {
    console.error('CFO authorization error:', error);
    res.status(500).json({ error: 'Failed to process CFO authorization' });
  }
});
```

### **Proof Chain Verification Endpoint**
```typescript
// POST /api/transactions/:id/verify
app.post('/api/transactions/:id/verify', async (req, res) => {
  try {
    const { id } = req.params;

    // Get complete transaction with all proofs
    const transaction = await db.payments.findById(id);
    if (!transaction) {
      return res.status(404).json({ error: 'Transaction not found' });
    }

    const verificationResults = [];

    // Verify initiation proof
    const initiationVerification = await proofMessenger.verifyProof(transaction.initiation_proof);
    verificationResults.push({
      step: 'initiation',
      signer: 'Alice (Finance Clerk)',
      valid: initiationVerification.valid,
      proofId: initiationVerification.proofId,
      error: initiationVerification.error
    });

    // Verify manager approval proof
    if (transaction.manager_approval_proof) {
      const managerVerification = await proofMessenger.verifyProof(transaction.manager_approval_proof);
      verificationResults.push({
        step: 'manager_approval',
        signer: 'Bob (Finance Manager)',
        valid: managerVerification.valid,
        proofId: managerVerification.proofId,
        error: managerVerification.error
      });
    }

    // Verify CFO authorization proof
    if (transaction.cfo_authorization_proof) {
      const cfoVerification = await proofMessenger.verifyProof(transaction.cfo_authorization_proof);
      verificationResults.push({
        step: 'cfo_authorization',
        signer: 'Carol (CFO)',
        valid: cfoVerification.valid,
        proofId: cfoVerification.proofId,
        error: cfoVerification.error
      });
    }

    // Check overall validity
    const allValid = verificationResults.every(result => result.valid);

    res.json({
      success: true,
      overallValid: allValid,
      verificationResults,
      message: allValid 
        ? 'All signatures are cryptographically valid for this context. This transaction is non-repudiable.'
        : 'One or more signatures failed verification.'
    });

  } catch (error) {
    console.error('Proof chain verification error:', error);
    res.status(500).json({ error: 'Failed to verify proof chain' });
  }
});
```

## 🎨 **UI Styling**

### **CSS for Demo**
```css
/* Dashboard Styling */
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: 'Inter', sans-serif;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 2px solid #e5e7eb;
}

.user-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 5px;
}

.auth-status {
  color: #10b981;
  font-size: 14px;
}

.approval-limit {
  color: #6b7280;
  font-size: 14px;
}

/* Approval Cards */
.approval-card {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.amount {
  font-size: 24px;
  font-weight: bold;
  color: #1f2937;
}

.amount.critical {
  color: #dc2626;
}

.policy-notice {
  background: #fef3c7;
  border: 1px solid #f59e0b;
  border-radius: 6px;
  padding: 12px;
  margin: 15px 0;
  color: #92400e;
}

/* Proof Chain Styling */
.proof-chain {
  margin: 20px 0;
}

.proof-step {
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  margin-bottom: 15px;
  overflow: hidden;
}

.proof-header {
  background: #f9fafb;
  padding: 15px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.proof-status.verified {
  color: #10b981;
  font-weight: bold;
}

.proof-details {
  padding: 20px;
}

.proof-context {
  margin-top: 15px;
}

.proof-context pre {
  background: #f3f4f6;
  padding: 15px;
  border-radius: 6px;
  font-size: 12px;
  overflow-x: auto;
}

/* Buttons */
.primary-button {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.primary-button:hover {
  background: #2563eb;
}

.critical-button {
  background: #dc2626;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.critical-button:hover {
  background: #b91c1c;
}

.verify-button {
  background: #059669;
  color: white;
  border: none;
  padding: 15px 30px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  margin: 20px 0;
}

/* Verification Results */
.verification-result {
  background: #f0fdf4;
  border: 1px solid #10b981;
  border-radius: 8px;
  padding: 20px;
  margin-top: 20px;
}

.verification-status.success {
  color: #065f46;
  font-size: 18px;
  margin-bottom: 15px;
}

.verification-details p {
  margin: 8px 0;
  color: #065f46;
}

.non-repudiation-notice {
  background: #dbeafe;
  border: 1px solid #3b82f6;
  border-radius: 6px;
  padding: 15px;
  margin-top: 15px;
}

.non-repudiation-notice strong {
  color: #1e40af;
}
```

## 🎯 **Key Demo Messages**

### **The Problem Statement**
"Traditional authorization systems rely on sessions and passwords that can be hijacked, shared, or repudiated. In high-stakes financial transactions, you need cryptographic proof that specific individuals approved specific actions."

### **The Solution Demonstration**
"Watch as each approval creates an immutable cryptographic signature that cannot be forged, shared, or denied. The final proof bundle contains the complete, verifiable history of all approvals."

### **The "Aha!" Moment**
"This is impossible with passkeys alone. Passkeys authenticate 'who you are' but Proof-Messenger proves 'what you approved.' Each signature is bound to the exact transaction details, creating legal-grade evidence of intent."

### **The Business Value**
"For enterprises handling high-value transactions, this provides:
- **Non-repudiation**: Users cannot deny their approvals
- **Audit compliance**: Complete cryptographic audit trail
- **Fraud prevention**: Impossible to forge or replay approvals
- **Regulatory compliance**: Meets SOX, PCI-DSS, and banking regulations"

This demo perfectly showcases the unique value proposition of Proof-Messenger: solving the "verifiable transactional intent" problem that no other solution addresses.

---

**Demo Version**: 1.0  
**Target Audience**: Enterprise decision makers, compliance officers, security architects  
**Demo Duration**: 10-15 minutes  
**Key Takeaway**: Cryptographic proof of multi-party approval that's impossible with traditional auth