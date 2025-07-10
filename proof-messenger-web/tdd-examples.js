/**
 * TDD-Ready WASM Crypto Usage Examples
 * 
 * This file demonstrates proper error handling, type safety,
 * and testing patterns for the WASM crypto bindings.
 */

import init, {
    WasmKeyPair,
    WasmMessage,
    WasmProof,
    bytes_to_hex,
    hex_to_bytes,
    generate_invite_code,
    validate_invite_code,
    validate_public_key,
    validate_signature,
    verify_signature,
    console_log,
    console_warn,
    console_error
} from './pkg/proof_messenger_web.js';

// D. Error Handling Patterns
class CryptoError extends Error {
    constructor(message, cause) {
        super(message);
        this.name = 'CryptoError';
        this.cause = cause;
    }
}

class ValidationError extends Error {
    constructor(message, field, value) {
        super(message);
        this.name = 'ValidationError';
        this.field = field;
        this.value = value;
    }
}

// E. Type-Safe Wrappers
class SafeKeyPair {
    constructor(wasmKeyPair) {
        if (!wasmKeyPair) {
            throw new CryptoError('Invalid keypair provided');
        }
        this._kp = wasmKeyPair;
        this._publicKeyHex = null;
        this._publicKeyBytes = null;
    }

    static generate() {
        try {
            const kp = new WasmKeyPair();
            return new SafeKeyPair(kp);
        } catch (error) {
            throw new CryptoError('Failed to generate keypair', error);
        }
    }

    static fromBytes(bytes) {
        if (!bytes || bytes.length !== 64) {
            throw new ValidationError('Invalid keypair bytes length', 'bytes', bytes?.length);
        }

        try {
            const kp = WasmKeyPair.from_bytes(bytes);
            return new SafeKeyPair(kp);
        } catch (error) {
            throw new CryptoError('Failed to create keypair from bytes', error);
        }
    }

    static fromHex(hex) {
        if (!hex || typeof hex !== 'string') {
            throw new ValidationError('Invalid hex string', 'hex', hex);
        }

        try {
            const bytes = hex_to_bytes(hex);
            return SafeKeyPair.fromBytes(bytes);
        } catch (error) {
            throw new CryptoError('Failed to create keypair from hex', error);
        }
    }

    get publicKeyHex() {
        if (!this._publicKeyHex) {
            this._publicKeyHex = this._kp.public_key_hex;
        }
        return this._publicKeyHex;
    }

    get publicKeyBytes() {
        if (!this._publicKeyBytes) {
            this._publicKeyBytes = this._kp.public_key_bytes;
        }
        return this._publicKeyBytes;
    }

    get privateKeyBytes() {
        return this._kp.private_key_bytes;
    }

    get keypairBytes() {
        return this._kp.keypair_bytes;
    }

    sign(data) {
        if (!data) {
            throw new ValidationError('Data to sign cannot be empty', 'data', data);
        }

        try {
            const dataBytes = typeof data === 'string' 
                ? new TextEncoder().encode(data)
                : data;
            return this._kp.sign(dataBytes);
        } catch (error) {
            throw new CryptoError('Failed to sign data', error);
        }
    }

    // Secure cleanup
    destroy() {
        if (this._kp) {
            this._kp.free();
            this._kp = null;
        }
        this._publicKeyHex = null;
        this._publicKeyBytes = null;
    }
}

// F. Message Handling with Error Recovery
class SafeMessage {
    constructor(senderBytes, recipientBytes, content) {
        this._validateInputs(senderBytes, recipientBytes, content);
        
        try {
            this._msg = new WasmMessage(senderBytes, recipientBytes, content);
        } catch (error) {
            throw new CryptoError('Failed to create message', error);
        }
    }

    static fromJson(json) {
        if (!json || typeof json !== 'string') {
            throw new ValidationError('Invalid JSON string', 'json', json);
        }

        try {
            const msg = WasmMessage.from_json(json);
            if (!msg) {
                throw new Error('Failed to parse message JSON');
            }
            return new SafeMessage._fromWasm(msg);
        } catch (error) {
            throw new CryptoError('Failed to create message from JSON', error);
        }
    }

    static _fromWasm(wasmMessage) {
        const instance = Object.create(SafeMessage.prototype);
        instance._msg = wasmMessage;
        return instance;
    }

    _validateInputs(senderBytes, recipientBytes, content) {
        if (!validate_public_key(senderBytes)) {
            throw new ValidationError('Invalid sender public key', 'senderBytes', senderBytes);
        }
        if (!validate_public_key(recipientBytes)) {
            throw new ValidationError('Invalid recipient public key', 'recipientBytes', recipientBytes);
        }
        if (!content || typeof content !== 'string') {
            throw new ValidationError('Invalid message content', 'content', content);
        }
        if (content.length > 10000) { // Reasonable limit
            throw new ValidationError('Message content too long', 'content.length', content.length);
        }
    }

    get id() { return this._msg.id; }
    get senderHex() { return this._msg.sender_hex; }
    get recipientHex() { return this._msg.recipient_hex; }
    get content() { return this._msg.content; }
    get timestamp() { return this._msg.timestamp; }
    get isSigned() { return this._msg.is_signed; }

    sign(keypair) {
        if (!(keypair instanceof SafeKeyPair)) {
            throw new ValidationError('Invalid keypair type', 'keypair', typeof keypair);
        }

        try {
            this._msg.sign(keypair.keypairBytes);
        } catch (error) {
            throw new CryptoError('Failed to sign message', error);
        }
    }

    verify(publicKeyBytes) {
        if (!validate_public_key(publicKeyBytes)) {
            throw new ValidationError('Invalid public key for verification', 'publicKeyBytes', publicKeyBytes);
        }

        try {
            return this._msg.verify(publicKeyBytes);
        } catch (error) {
            throw new CryptoError('Failed to verify message', error);
        }
    }

    toJson() {
        try {
            return this._msg.to_json();
        } catch (error) {
            throw new CryptoError('Failed to serialize message', error);
        }
    }

    destroy() {
        if (this._msg) {
            this._msg.free();
            this._msg = null;
        }
    }
}

// G. Utility Functions with Validation
class CryptoUtils {
    static async generateInviteCode() {
        try {
            return generate_invite_code();
        } catch (error) {
            throw new CryptoError('Failed to generate invite code', error);
        }
    }

    static validateInviteCode(code) {
        if (!code || typeof code !== 'string') {
            return false;
        }
        return validate_invite_code(code);
    }

    static bytesToHex(bytes) {
        if (!bytes || !(bytes instanceof Uint8Array)) {
            throw new ValidationError('Invalid bytes array', 'bytes', bytes);
        }
        return bytes_to_hex(bytes);
    }

    static hexToBytes(hex) {
        if (!hex || typeof hex !== 'string') {
            throw new ValidationError('Invalid hex string', 'hex', hex);
        }
        
        try {
            return hex_to_bytes(hex);
        } catch (error) {
            throw new CryptoError('Failed to decode hex string', error);
        }
    }

    static verifySignature(publicKeyBytes, message, signatureBytes) {
        if (!validate_public_key(publicKeyBytes)) {
            throw new ValidationError('Invalid public key', 'publicKeyBytes', publicKeyBytes);
        }
        if (!validate_signature(signatureBytes)) {
            throw new ValidationError('Invalid signature', 'signatureBytes', signatureBytes);
        }
        if (!message) {
            throw new ValidationError('Message cannot be empty', 'message', message);
        }

        try {
            const messageBytes = typeof message === 'string' 
                ? new TextEncoder().encode(message)
                : message;
            return verify_signature(publicKeyBytes, messageBytes, signatureBytes);
        } catch (error) {
            throw new CryptoError('Failed to verify signature', error);
        }
    }
}

// H. Test Utilities
class TestFramework {
    constructor() {
        this.tests = [];
        this.results = {
            passed: 0,
            failed: 0,
            total: 0
        };
    }

    test(name, testFn) {
        this.tests.push({ name, testFn });
    }

    async run() {
        console.log('🧪 Running test suite...');
        this.results = { passed: 0, failed: 0, total: 0 };

        for (const { name, testFn } of this.tests) {
            this.results.total++;
            try {
                await testFn();
                this.results.passed++;
                console.log(`✅ ${name}`);
            } catch (error) {
                this.results.failed++;
                console.error(`❌ ${name}: ${error.message}`);
            }
        }

        const passRate = ((this.results.passed / this.results.total) * 100).toFixed(1);
        console.log(`\n📊 Results: ${this.results.passed}/${this.results.total} passed (${passRate}%)`);
        
        return this.results;
    }

    assert(condition, message) {
        if (!condition) {
            throw new Error(message || 'Assertion failed');
        }
    }

    assertEqual(actual, expected, message) {
        if (actual !== expected) {
            throw new Error(message || `Expected ${expected}, got ${actual}`);
        }
    }

    assertThrows(fn, expectedError, message) {
        try {
            fn();
            throw new Error(message || 'Expected function to throw');
        } catch (error) {
            if (expectedError && !(error instanceof expectedError)) {
                throw new Error(message || `Expected ${expectedError.name}, got ${error.constructor.name}`);
            }
        }
    }
}

// I. Usage Examples
async function demonstrateUsage() {
    console.log('🚀 Demonstrating TDD-Ready WASM Crypto Usage');

    try {
        // Initialize WASM
        await init();
        console.log('✅ WASM initialized');

        // Example 1: Safe keypair generation
        const alice = SafeKeyPair.generate();
        const bob = SafeKeyPair.generate();
        console.log('✅ Generated keypairs for Alice and Bob');

        // Example 2: Message creation and signing
        const message = new SafeMessage(
            alice.publicKeyBytes,
            bob.publicKeyBytes,
            'Hello Bob! This is a secure message.'
        );
        
        message.sign(alice);
        console.log('✅ Message created and signed');

        // Example 3: Message verification
        const isValid = message.verify(alice.publicKeyBytes);
        console.log(`✅ Message verification: ${isValid ? 'Valid' : 'Invalid'}`);

        // Example 4: JSON serialization
        const json = message.toJson();
        const restored = SafeMessage.fromJson(json);
        console.log('✅ Message serialization/deserialization');

        // Example 5: Utility functions
        const inviteCode = await CryptoUtils.generateInviteCode();
        const isValidCode = CryptoUtils.validateInviteCode(inviteCode);
        console.log(`✅ Generated invite code: ${inviteCode} (valid: ${isValidCode})`);

        // Cleanup
        alice.destroy();
        bob.destroy();
        message.destroy();
        restored.destroy();
        
        console.log('✅ Resources cleaned up');

    } catch (error) {
        console.error('❌ Demo failed:', error);
    }
}

// J. Property-Based Test Examples
function createPropertyTests() {
    const framework = new TestFramework();

    framework.test('Keypair generation is deterministic', () => {
        const kp1 = SafeKeyPair.generate();
        const bytes = kp1.keypairBytes;
        const kp2 = SafeKeyPair.fromBytes(bytes);
        
        framework.assertEqual(kp1.publicKeyHex, kp2.publicKeyHex, 'Public keys should match');
        
        kp1.destroy();
        kp2.destroy();
    });

    framework.test('Sign-verify roundtrip always works', () => {
        const kp = SafeKeyPair.generate();
        const testData = 'Test message for signing';
        
        const signature = kp.sign(testData);
        const isValid = CryptoUtils.verifySignature(
            kp.publicKeyBytes,
            testData,
            signature
        );
        
        framework.assert(isValid, 'Signature should be valid');
        kp.destroy();
    });

    framework.test('Invalid inputs throw appropriate errors', () => {
        framework.assertThrows(
            () => SafeKeyPair.fromBytes(new Uint8Array(30)),
            ValidationError,
            'Should throw ValidationError for wrong length'
        );

        framework.assertThrows(
            () => CryptoUtils.hexToBytes('invalid_hex'),
            CryptoError,
            'Should throw CryptoError for invalid hex'
        );
    });

    framework.test('Message validation works correctly', () => {
        const kp = SafeKeyPair.generate();
        
        framework.assertThrows(
            () => new SafeMessage(
                new Uint8Array(30), // Invalid key length
                kp.publicKeyBytes,
                'test'
            ),
            ValidationError,
            'Should reject invalid sender key'
        );

        framework.assertThrows(
            () => new SafeMessage(
                kp.publicKeyBytes,
                kp.publicKeyBytes,
                '' // Empty content
            ),
            ValidationError,
            'Should reject empty content'
        );

        kp.destroy();
    });

    return framework;
}

// Export for use in other modules
export {
    SafeKeyPair,
    SafeMessage,
    CryptoUtils,
    TestFramework,
    CryptoError,
    ValidationError,
    demonstrateUsage,
    createPropertyTests
};

// Auto-run demo if this is the main module
if (typeof window !== 'undefined') {
    window.addEventListener('load', demonstrateUsage);
}