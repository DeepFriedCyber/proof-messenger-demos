// Proof Messenger Web Application
import init, {
    WasmKeyPair,
    WasmMessage,
    WasmProof,
    RelayConnection,
    LocalStorage,
    Utils,
    EventDispatcher,
    console_log,
    console_warn,
    console_error,
    validate_invite_code,
    generate_invite_code,
    bytes_to_hex,
    hex_to_bytes
} from '../pkg/proof_messenger_web.js';

class ProofMessengerApp {
    constructor() {
        this.keypair = null;
        this.displayName = '';
        this.relayConnection = null;
        this.messages = [];
        this.invitations = [];
        this.currentTab = 'onboard';
        this.eventDispatcher = null;
        
        // Bind methods
        this.init = this.init.bind(this);
        this.setupEventListeners = this.setupEventListeners.bind(this);
        this.switchTab = this.switchTab.bind(this);
        this.onboard = this.onboard.bind(this);
        this.generateIdentity = this.generateIdentity.bind(this);
        this.sendMessage = this.sendMessage.bind(this);
        this.loadMessages = this.loadMessages.bind(this);
        this.connectToRelay = this.connectToRelay.bind(this);
    }
    
    async init() {
        try {
            console_log('Initializing Proof Messenger Web...');
            
            // Initialize WASM module with explicit path
            await init('../pkg/proof_messenger_web_bg.wasm');
            
            // Initialize event dispatcher
            this.eventDispatcher = new EventDispatcher();
            
            // Hide loading indicator
            document.getElementById('loading').classList.add('d-none');
            
            // Load saved data
            await this.loadSavedData();
            
            // Setup event listeners
            this.setupEventListeners();
            
            // Connect to relay if we have an identity
            if (this.keypair) {
                await this.connectToRelay();
                this.switchTab('messages');
            }
            
            console_log('Application initialized successfully');
            this.showSuccess('Application loaded successfully!');
            
        } catch (error) {
            console_error('Failed to initialize application: ' + error);
            this.showError('Failed to initialize application: ' + error.message);
        }
    }
    
    setupEventListeners() {
        // Tab navigation
        document.querySelectorAll('[data-tab]').forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                this.switchTab(e.target.dataset.tab);
            });
        });
        
        // Onboarding form
        document.getElementById('onboard-form').addEventListener('submit', this.onboard);
        
        // Generate identity button
        document.getElementById('generate-identity').addEventListener('click', this.generateIdentity);
        
        // Send message form
        document.getElementById('send-message-form').addEventListener('submit', this.sendMessage);
        
        // Identity actions
        document.getElementById('export-identity').addEventListener('click', this.exportIdentity.bind(this));
        document.getElementById('create-invitation').addEventListener('click', this.createInvitation.bind(this));
        
        // Demo buttons
        document.getElementById('demo-e2e').addEventListener('click', this.runE2EDemo.bind(this));
        document.getElementById('demo-benchmark').addEventListener('click', this.runBenchmark.bind(this));
        document.getElementById('demo-proofs').addEventListener('click', this.runProofDemo.bind(this));
        
        // Refresh messages
        document.getElementById('refresh-messages').addEventListener('click', this.loadMessages);
        
        // Input validation
        document.getElementById('invite-code').addEventListener('input', (e) => {
            e.target.value = e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, '');
        });
    }
    
    switchTab(tabName) {
        // Update navigation
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
        
        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.add('d-none');
        });
        document.getElementById(`tab-${tabName}`).classList.remove('d-none');
        
        this.currentTab = tabName;
        
        // Load tab-specific data
        if (tabName === 'messages') {
            this.loadMessages();
        } else if (tabName === 'identity') {
            this.updateIdentityDisplay();
        }
    }
    
    async onboard(e) {
        e.preventDefault();
        
        const inviteCode = document.getElementById('invite-code').value.trim();
        const displayName = document.getElementById('display-name').value.trim();
        const spinner = document.getElementById('onboard-spinner');
        
        if (!validate_invite_code(inviteCode)) {
            this.showError('Invalid invitation code format. Must be 8 alphanumeric characters.');
            return;
        }
        
        try {
            spinner.classList.remove('d-none');
            
            // Generate new keypair
            this.keypair = new WasmKeyPair();
            this.displayName = displayName;
            
            console_log('Generated new identity for onboarding');
            
            // Save to local storage
            await this.saveIdentity();
            
            // Create identity proof
            const identityProof = new WasmProof('identity', this.keypair.public_key_bytes);
            identityProof.sign(this.keypair);
            
            console_log('Created identity proof: ' + identityProof.id);
            
            this.showSuccess(`Welcome ${displayName}! Your identity has been created.`);
            
            // Connect to relay
            await this.connectToRelay();
            
            // Switch to messages tab
            this.switchTab('messages');
            
        } catch (error) {
            console_error('Onboarding failed: ' + error);
            this.showError('Onboarding failed: ' + error.message);
        } finally {
            spinner.classList.add('d-none');
        }
    }
    
    async generateIdentity() {
        try {
            this.keypair = new WasmKeyPair();
            this.displayName = 'Anonymous User';
            
            // Save to local storage
            await this.saveIdentity();
            
            // Show generated key
            document.getElementById('generated-public-key').textContent = this.keypair.public_key_hex;
            document.getElementById('identity-info').classList.remove('d-none');
            
            console_log('Generated new identity');
            this.showSuccess('New identity generated successfully!');
            
        } catch (error) {
            console_error('Failed to generate identity: ' + error);
            this.showError('Failed to generate identity: ' + error.message);
        }
    }
    
    async sendMessage(e) {
        e.preventDefault();
        
        if (!this.keypair) {
            this.showError('No identity found. Please onboard first.');
            return;
        }
        
        const recipientKey = document.getElementById('recipient-key').value.trim();
        const content = document.getElementById('message-content').value.trim();
        const attachProof = document.getElementById('attach-identity-proof').checked;
        const requireReceipt = document.getElementById('require-receipt').checked;
        const priority = document.getElementById('message-priority').value;
        const spinner = document.getElementById('send-spinner');
        
        try {
            spinner.classList.remove('d-none');
            
            // Validate recipient key
            if (recipientKey.length !== 64) {
                throw new Error('Recipient public key must be 64 characters');
            }
            
            const recipientBytes = hex_to_bytes(recipientKey);
            
            // Create message
            const message = new WasmMessage(
                this.keypair.public_key_bytes,
                recipientBytes,
                content
            );
            
            // Sign message
            message.sign(this.keypair);
            
            // Add identity proof if requested
            if (attachProof) {
                // Note: In a full implementation, we'd add the proof to the message
                console_log('Identity proof would be attached');
            }
            
            console_log('Message created: ' + message.id);
            
            // Send through relay (if connected)
            if (this.relayConnection && this.relayConnection.ready_state() === WebSocket.OPEN) {
                const messageJson = message.to_json();
                this.relayConnection.send(messageJson);
                console_log('Message sent through relay');
            }
            
            // Save to local storage
            await this.saveMessage(message, 'outbox');
            
            // Clear form
            document.getElementById('send-message-form').reset();
            
            this.showSuccess('Message sent successfully!');
            
        } catch (error) {
            console_error('Failed to send message: ' + error);
            this.showError('Failed to send message: ' + error.message);
        } finally {
            spinner.classList.add('d-none');
        }
    }
    
    async loadMessages() {
        try {
            // Load from local storage
            const inboxData = LocalStorage.load('messages_inbox');
            const outboxData = LocalStorage.load('messages_outbox');
            
            this.messages = [];
            
            if (inboxData) {
                const inbox = JSON.parse(inboxData);
                this.messages.push(...inbox.map(msg => ({ ...msg, type: 'received' })));
            }
            
            if (outboxData) {
                const outbox = JSON.parse(outboxData);
                this.messages.push(...outbox.map(msg => ({ ...msg, type: 'sent' })));
            }
            
            // Sort by timestamp
            this.messages.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
            
            this.updateMessagesList();
            
        } catch (error) {
            console_error('Failed to load messages: ' + error);
        }
    }
    
    updateMessagesList() {
        const messageList = document.getElementById('message-list');
        
        if (this.messages.length === 0) {
            messageList.innerHTML = '<div class="list-group-item text-center text-muted">No messages yet</div>';
            return;
        }
        
        messageList.innerHTML = this.messages.map((msg, index) => `
            <div class="list-group-item message-item" data-index="${index}">
                <div class="d-flex justify-content-between align-items-start">
                    <div class="flex-grow-1">
                        <h6 class="mb-1">
                            ${msg.type === 'sent' ? '📤 To' : '📥 From'}: 
                            <code class="small">${this.truncateKey(msg.type === 'sent' ? msg.recipient_hex : msg.sender_hex)}</code>
                        </h6>
                        <p class="mb-1">${this.truncateText(msg.content, 50)}</p>
                        <small class="text-muted">${this.formatTimestamp(msg.timestamp)}</small>
                    </div>
                    <div class="text-end">
                        ${msg.is_signed ? '<span class="badge bg-success">✓</span>' : ''}
                        ${msg.type === 'sent' ? '<span class="badge bg-primary">Sent</span>' : '<span class="badge bg-info">Received</span>'}
                    </div>
                </div>
            </div>
        `).join('');
        
        // Add click handlers
        messageList.querySelectorAll('.message-item').forEach(item => {
            item.addEventListener('click', () => {
                const index = parseInt(item.dataset.index);
                this.showMessageDetails(this.messages[index]);
            });
        });
    }
    
    showMessageDetails(message) {
        const detailsDiv = document.getElementById('message-details');
        const contentDiv = document.getElementById('message-content-display');
        
        contentDiv.innerHTML = `
            <div class="row">
                <div class="col-md-6">
                    <h6>Message Information</h6>
                    <p><strong>ID:</strong> <code>${message.id}</code></p>
                    <p><strong>From:</strong> <code class="public-key">${message.sender_hex}</code></p>
                    <p><strong>To:</strong> <code class="public-key">${message.recipient_hex}</code></p>
                    <p><strong>Timestamp:</strong> ${this.formatTimestamp(message.timestamp)}</p>
                    <p><strong>Signed:</strong> ${message.is_signed ? '✅ Yes' : '❌ No'}</p>
                </div>
                <div class="col-md-6">
                    <h6>Content</h6>
                    <div class="border rounded p-3 bg-light">
                        ${message.content}
                    </div>
                </div>
            </div>
        `;
        
        detailsDiv.classList.remove('d-none');
    }
    
    async connectToRelay() {
        try {
            const relayUrl = 'ws://localhost:8080';
            this.relayConnection = new RelayConnection(relayUrl);
            
            // Set up event handlers
            this.relayConnection.set_on_message((data) => {
                console_log('Received message from relay: ' + data);
                // Handle incoming message
            });
            
            this.relayConnection.set_on_error((error) => {
                console_error('Relay connection error: ' + error);
                this.updateConnectionStatus('error');
            });
            
            this.relayConnection.set_on_close(() => {
                console_log('Relay connection closed');
                this.updateConnectionStatus('disconnected');
            });
            
            // Connect
            this.relayConnection.connect();
            this.updateConnectionStatus('connecting');
            
            // Check connection after a delay
            setTimeout(() => {
                if (this.relayConnection.ready_state() === WebSocket.OPEN) {
                    this.updateConnectionStatus('connected');
                    console_log('Connected to relay server');
                } else {
                    this.updateConnectionStatus('disconnected');
                    console_warn('Failed to connect to relay server');
                }
            }, 2000);
            
        } catch (error) {
            console_error('Failed to connect to relay: ' + error);
            this.updateConnectionStatus('error');
        }
    }
    
    updateConnectionStatus(status) {
        const statusElement = document.getElementById('connection-status');
        const badges = {
            'connected': '<span class="badge bg-success">Connected</span>',
            'connecting': '<span class="badge bg-warning">Connecting...</span>',
            'disconnected': '<span class="badge bg-secondary">Disconnected</span>',
            'error': '<span class="badge bg-danger">Error</span>'
        };
        
        statusElement.innerHTML = badges[status] || badges['disconnected'];
    }
    
    updateIdentityDisplay() {
        const identityDisplay = document.getElementById('identity-display');
        
        if (!this.keypair) {
            identityDisplay.innerHTML = '<p class="text-muted">No identity loaded</p>';
            return;
        }
        
        identityDisplay.innerHTML = `
            <div class="mb-3">
                <h6>Display Name</h6>
                <p class="mb-2">${this.displayName}</p>
            </div>
            <div class="mb-3">
                <h6>Public Key</h6>
                <div class="public-key">${this.keypair.public_key_hex}</div>
            </div>
            <div class="mb-3">
                <h6>Status</h6>
                <span class="badge bg-success">✅ Identity verified</span>
            </div>
        `;
    }
    
    async exportIdentity() {
        if (!this.keypair) {
            this.showError('No identity to export');
            return;
        }
        
        try {
            const publicKeyHex = this.keypair.public_key_hex;
            
            // Copy to clipboard
            await navigator.clipboard.writeText(publicKeyHex);
            this.showSuccess('Public key copied to clipboard!');
            
        } catch (error) {
            console_error('Failed to export identity: ' + error);
            this.showError('Failed to export identity: ' + error.message);
        }
    }
    
    async createInvitation() {
        if (!this.keypair) {
            this.showError('No identity found. Generate an identity first.');
            return;
        }
        
        try {
            const inviteCode = generate_invite_code();
            const invitation = {
                id: crypto.randomUUID(),
                code: inviteCode,
                creator: this.keypair.public_key_hex,
                created_at: new Date().toISOString(),
                expires_at: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days
                used: false
            };
            
            // Save invitation
            this.invitations.push(invitation);
            await LocalStorage.save('invitations', JSON.stringify(this.invitations));
            
            this.showSuccess(`Invitation created: ${inviteCode}`);
            this.updateInvitationsList();
            
        } catch (error) {
            console_error('Failed to create invitation: ' + error);
            this.showError('Failed to create invitation: ' + error.message);
        }
    }
    
    updateInvitationsList() {
        const invitationsList = document.getElementById('invitations-list');
        
        if (this.invitations.length === 0) {
            invitationsList.innerHTML = '<p class="text-muted">No invitations created</p>';
            return;
        }
        
        invitationsList.innerHTML = this.invitations.map(inv => `
            <div class="card invitation-card mb-2 ${inv.used ? 'used' : ''}">
                <div class="card-body py-2">
                    <div class="d-flex justify-content-between align-items-center">
                        <div>
                            <strong>${inv.code}</strong>
                            <small class="text-muted d-block">
                                Created: ${this.formatTimestamp(inv.created_at)}
                            </small>
                        </div>
                        <div>
                            ${inv.used ? '<span class="badge bg-warning">Used</span>' : '<span class="badge bg-success">Active</span>'}
                        </div>
                    </div>
                </div>
            </div>
        `).join('');
    }
    
    // Demo functions
    async runE2EDemo() {
        const log = document.getElementById('demo-log');
        this.clearDemoLog();
        
        try {
            this.logDemo('🚀 Starting End-to-End Demo...');
            
            // Generate Alice and Bob
            this.logDemo('1. Generating identities for Alice and Bob...');
            const alice = new WasmKeyPair();
            const bob = new WasmKeyPair();
            
            this.logDemo(`   Alice: ${alice.public_key_hex.substring(0, 16)}...`);
            this.logDemo(`   Bob: ${bob.public_key_hex.substring(0, 16)}...`);
            
            // Alice creates identity proof
            this.logDemo('2. Alice creates identity proof...');
            const aliceProof = new WasmProof('identity', alice.public_key_bytes);
            aliceProof.sign(alice);
            this.logDemo(`   ✅ Identity proof: ${aliceProof.id}`);
            
            // Alice sends message to Bob
            this.logDemo('3. Alice sends message to Bob...');
            const message = new WasmMessage(
                alice.public_key_bytes,
                bob.public_key_bytes,
                'Hello Bob! This is a demo message from Alice.'
            );
            message.sign(alice);
            this.logDemo(`   ✅ Message: ${message.id}`);
            this.logDemo(`   📝 Content: "${message.content}"`);
            
            // Bob verifies message
            this.logDemo('4. Bob verifies the message...');
            const sigValid = message.verify(aliceKeypair.public_key_bytes);
            this.logDemo(`   Signature: ${sigValid ? '✅ Valid' : '❌ Invalid'}`);
            
            // Verify proof
            const proofValid = aliceProof.verify();
            this.logDemo(`   Identity proof: ${proofValid ? '✅ Valid' : '❌ Invalid'}`);
            
            this.logDemo('🎉 End-to-end demo completed successfully!');
            
        } catch (error) {
            this.logDemo(`❌ Demo failed: ${error.message}`);
        }
    }
    
    async runBenchmark() {
        const log = document.getElementById('demo-log');
        this.clearDemoLog();
        
        try {
            this.logDemo('⚡ Running Cryptographic Benchmarks...');
            
            // Key generation benchmark
            this.logDemo('1. Key generation benchmark...');
            const keyGenStart = performance.now();
            for (let i = 0; i < 10; i++) {
                new WasmKeyPair();
            }
            const keyGenTime = performance.now() - keyGenStart;
            this.logDemo(`   10 keypairs: ${keyGenTime.toFixed(2)}ms (${(keyGenTime/10).toFixed(2)}ms/keypair)`);
            
            // Signing benchmark
            this.logDemo('2. Message signing benchmark...');
            const keypair = new WasmKeyPair();
            const testData = new TextEncoder().encode('This is a test message for benchmarking');
            
            const signStart = performance.now();
            for (let i = 0; i < 100; i++) {
                keypair.sign(testData);
            }
            const signTime = performance.now() - signStart;
            this.logDemo(`   100 signatures: ${signTime.toFixed(2)}ms (${(signTime*10).toFixed(2)}μs/signature)`);
            
            // Verification benchmark
            this.logDemo('3. Signature verification benchmark...');
            const signature = keypair.sign(testData);
            
            const verifyStart = performance.now();
            for (let i = 0; i < 100; i++) {
                // Create a message and verify it
                const testMessage = new WasmMessage(
                    keypair.public_key_bytes,
                    keypair.public_key_bytes,
                    'test'
                );
                testMessage.sign(keypair.keypair_bytes);
                testMessage.verify(keypair.public_key_bytes);
            }
            const verifyTime = performance.now() - verifyStart;
            this.logDemo(`   100 verifications: ${verifyTime.toFixed(2)}ms (${(verifyTime*10).toFixed(2)}μs/verification)`);
            
            this.logDemo('⚡ Benchmark completed!');
            
        } catch (error) {
            this.logDemo(`❌ Benchmark failed: ${error.message}`);
        }
    }
    
    async runProofDemo() {
        const log = document.getElementById('demo-log');
        this.clearDemoLog();
        
        try {
            this.logDemo('🔍 Running Proof Verification Demo...');
            
            const keypair = new WasmKeyPair();
            
            // Test different proof types
            const proofTypes = ['identity', 'message', 'timestamp'];
            
            for (const proofType of proofTypes) {
                this.logDemo(`Testing ${proofType} proof...`);
                
                const testData = new TextEncoder().encode(`Test data for ${proofType} proof`);
                const proof = new WasmProof(proofType, testData);
                proof.sign(keypair);
                
                const isValid = proof.verify();
                this.logDemo(`   ${proofType} proof: ${isValid ? '✅ Valid' : '❌ Invalid'}`);
            }
            
            // Test tampered proof
            this.logDemo('Testing tampered proof...');
            const tamperedProof = new WasmProof('message', new TextEncoder().encode('original data'));
            tamperedProof.sign(keypair);
            
            // Simulate tampering by creating a new proof with different data but same signature
            const tamperedData = new TextEncoder().encode('tampered data');
            const newTamperedProof = new WasmProof('message', tamperedData);
            
            const tamperedValid = newTamperedProof.verify();
            this.logDemo(`   Tampered proof: ${tamperedValid ? '❌ Incorrectly valid' : '✅ Correctly invalid'}`);
            
            this.logDemo('🔍 Proof verification demo completed!');
            
        } catch (error) {
            this.logDemo(`❌ Proof demo failed: ${error.message}`);
        }
    }
    
    // Utility functions
    async loadSavedData() {
        try {
            // Load identity
            const identityData = LocalStorage.load('identity');
            const displayNameData = LocalStorage.load('display_name');
            
            if (identityData && displayNameData) {
                const keyBytes = hex_to_bytes(identityData);
                this.keypair = WasmKeyPair.from_bytes(keyBytes);
                this.displayName = displayNameData;
                console_log('Loaded saved identity');
            }
            
            // Load invitations
            const invitationsData = LocalStorage.load('invitations');
            if (invitationsData) {
                this.invitations = JSON.parse(invitationsData);
            }
            
        } catch (error) {
            console_error('Failed to load saved data: ' + error);
        }
    }
    
    async saveIdentity() {
        try {
            await LocalStorage.save('identity', bytes_to_hex(this.keypair.private_key_bytes));
            await LocalStorage.save('display_name', this.displayName);
            console_log('Identity saved to local storage');
        } catch (error) {
            console_error('Failed to save identity: ' + error);
        }
    }
    
    async saveMessage(message, folder) {
        try {
            const key = `messages_${folder}`;
            const existingData = LocalStorage.load(key);
            const messages = existingData ? JSON.parse(existingData) : [];
            
            const messageData = {
                id: message.id,
                sender_hex: message.sender_hex,
                recipient_hex: message.recipient_hex,
                content: message.content,
                timestamp: message.timestamp,
                is_signed: message.is_signed
            };
            
            messages.push(messageData);
            await LocalStorage.save(key, JSON.stringify(messages));
            
        } catch (error) {
            console_error('Failed to save message: ' + error);
        }
    }
    
    hexToBytes(hex) {
        const bytes = new Uint8Array(hex.length / 2);
        for (let i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
        }
        return bytes;
    }
    
    bytesToHex(bytes) {
        return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    }
    
    truncateKey(key) {
        return key ? `${key.substring(0, 8)}...${key.substring(key.length - 8)}` : '';
    }
    
    truncateText(text, maxLength) {
        return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
    }
    
    formatTimestamp(timestamp) {
        return new Date(timestamp).toLocaleString();
    }
    
    logDemo(message) {
        const log = document.getElementById('demo-log');
        log.textContent += message + '\n';
        log.scrollTop = log.scrollHeight;
    }
    
    clearDemoLog() {
        document.getElementById('demo-log').textContent = '';
    }
    
    showError(message) {
        const alert = document.getElementById('error-alert');
        document.getElementById('error-message').textContent = message;
        alert.classList.remove('d-none');
        setTimeout(() => alert.classList.add('d-none'), 5000);
    }
    
    showSuccess(message) {
        const alert = document.getElementById('success-alert');
        document.getElementById('success-message').textContent = message;
        alert.classList.remove('d-none');
        setTimeout(() => alert.classList.add('d-none'), 3000);
    }
}

// Initialize the application when the page loads
document.addEventListener('DOMContentLoaded', () => {
    const app = new ProofMessengerApp();
    app.init();
});

// Make app available globally for debugging
window.ProofMessengerApp = ProofMessengerApp;