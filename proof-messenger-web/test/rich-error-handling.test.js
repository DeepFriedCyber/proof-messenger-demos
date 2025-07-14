// Rich Error Handling Tests for WASM Integration
import { expect, test, describe, beforeAll } from 'vitest';

let wasm;

beforeAll(async () => {
    wasm = await import('../pkg/proof_messenger_web.js');
});

describe('Rich Error Propagation from Rust to JavaScript', () => {
    
    test('verify_proof_secure_wasm should throw specific error for invalid public key', async () => {
        const invalidPublicKey = new Uint8Array(31); // Wrong length
        const context = new TextEncoder().encode("test context");
        const signature = new Uint8Array(64);
        
        try {
            await wasm.verify_proof_secure_wasm(invalidPublicKey, context, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Invalid public key format');
            expect(error.message).toContain('Failed to parse public key');
            expect(error.errorType).toBe('InvalidPublicKey');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('verify_proof_secure_wasm should throw specific error for invalid signature', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const publicKey = keypair.public_key_bytes();
        const context = new TextEncoder().encode("test context");
        const invalidSignature = new Uint8Array(63); // Wrong length
        
        try {
            await wasm.verify_proof_secure_wasm(publicKey, context, invalidSignature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Invalid signature format');
            expect(error.message).toContain('Failed to parse signature');
            expect(error.errorType).toBe('InvalidSignature');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('verify_proof_secure_wasm should throw verification failed error for wrong signature', async () => {
        const keypair1 = new wasm.WasmSecureKeyPair();
        const keypair2 = new wasm.WasmSecureKeyPair();
        const context = new TextEncoder().encode("test context");
        
        // Sign with keypair1
        const signature = keypair1.sign(context);
        
        // Try to verify with keypair2's public key
        const wrongPublicKey = keypair2.public_key_bytes();
        
        try {
            await wasm.verify_proof_secure_wasm(wrongPublicKey, context, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Signature verification failed');
            expect(error.errorType).toBe('VerificationFailed');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('make_secure_proof_strict_wasm should throw empty context error', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const keypairBytes = keypair.keypair_bytes();
        const emptyContext = new Uint8Array(0);
        
        try {
            await wasm.make_secure_proof_strict_wasm(keypairBytes, emptyContext);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Context data cannot be empty');
            expect(error.errorType).toBe('EmptyContext');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('make_secure_proof_wasm should throw context too large error', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const keypairBytes = keypair.keypair_bytes();
        
        // Create oversized context (1MB + 1 byte)
        const oversizedContext = new Uint8Array(1024 * 1024 + 1);
        
        try {
            await wasm.make_secure_proof_wasm(keypairBytes, oversizedContext);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Context data is too large');
            expect(error.message).toContain('1048577 bytes');
            expect(error.message).toContain('1048576 bytes');
            expect(error.errorType).toBe('ContextTooLarge');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('WasmSecureKeyPair.from_bytes should throw invalid private key error', async () => {
        const invalidKeypairBytes = new Uint8Array(63); // Wrong length
        
        try {
            wasm.WasmSecureKeyPair.from_bytes(invalidKeypairBytes);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Invalid private key format');
            expect(error.message).toContain('Failed to parse secure keypair');
            expect(error.errorType).toBe('InvalidPrivateKey');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('verify_proof_strict_wasm should throw empty context error', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const publicKey = keypair.public_key_bytes();
        const emptyContext = new Uint8Array(0);
        const signature = new Uint8Array(64);
        
        try {
            await wasm.verify_proof_strict_wasm(publicKey, emptyContext, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Context data cannot be empty');
            expect(error.errorType).toBe('EmptyContext');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
});

describe('Successful Operations with Rich Error Handling', () => {
    
    test('WasmSecureKeyPair should work correctly with valid inputs', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const context = new TextEncoder().encode("valid test context");
        
        // Sign the context
        const signature = keypair.sign(context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Verify the signature
        const isValid = await keypair.verify(context, signature);
        expect(isValid).toBe(true);
    });
    
    test('make_secure_proof_wasm and verify_proof_secure_wasm should work together', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const keypairBytes = keypair.keypair_bytes();
        const publicKey = keypair.public_key_bytes();
        const context = new TextEncoder().encode("integration test");
        
        // Create secure proof
        const signature = await wasm.make_secure_proof_wasm(keypairBytes, context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Verify secure proof
        const isValid = await wasm.verify_proof_secure_wasm(publicKey, context, signature);
        expect(isValid).toBe(true);
    });
    
    test('strict validation should work with non-empty context', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const keypairBytes = keypair.keypair_bytes();
        const publicKey = keypair.public_key_bytes();
        const context = new TextEncoder().encode("non-empty context");
        
        // Create strict proof
        const signature = await wasm.make_secure_proof_strict_wasm(keypairBytes, context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Verify strict proof
        const isValid = await wasm.verify_proof_strict_wasm(publicKey, context, signature);
        expect(isValid).toBe(true);
    });
    
    test('deterministic keypair generation should work', async () => {
        const seed = 12345;
        const keypair1 = wasm.WasmSecureKeyPair.from_seed(seed);
        const keypair2 = wasm.WasmSecureKeyPair.from_seed(seed);
        
        // Should generate identical keypairs
        expect(keypair1.public_key_hex()).toBe(keypair2.public_key_hex());
        
        const context = new TextEncoder().encode("deterministic test");
        const signature1 = keypair1.sign(context);
        const signature2 = keypair2.sign(context);
        
        // Should produce identical signatures
        expect(Array.from(signature1)).toEqual(Array.from(signature2));
    });
});

describe('Error Object Properties', () => {
    
    test('error objects should have consistent structure', async () => {
        const keypair = new wasm.WasmSecureKeyPair();
        const emptyContext = new Uint8Array(0);
        
        try {
            await keypair.sign_strict(emptyContext);
            expect.fail('Should have thrown an error');
        } catch (error) {
            // Check that error has all expected properties
            expect(error).toHaveProperty('message');
            expect(error).toHaveProperty('errorType');
            expect(error).toHaveProperty('isProofMessengerError');
            
            expect(typeof error.message).toBe('string');
            expect(typeof error.errorType).toBe('string');
            expect(error.isProofMessengerError).toBe(true);
            
            // Check that it's still a proper Error object
            expect(error).toBeInstanceOf(Error);
        }
    });
    
    test('different error types should have different errorType values', async () => {
        const errorTypes = new Set();
        
        // Test InvalidPublicKey
        try {
            const invalidPublicKey = new Uint8Array(31);
            const context = new TextEncoder().encode("test");
            const signature = new Uint8Array(64);
            await wasm.verify_proof_secure_wasm(invalidPublicKey, context, signature);
        } catch (error) {
            errorTypes.add(error.errorType);
        }
        
        // Test EmptyContext
        try {
            const keypair = new wasm.WasmSecureKeyPair();
            const emptyContext = new Uint8Array(0);
            await keypair.sign_strict(emptyContext);
        } catch (error) {
            errorTypes.add(error.errorType);
        }
        
        // Test InvalidSignature
        try {
            const keypair = new wasm.WasmSecureKeyPair();
            const publicKey = keypair.public_key_bytes();
            const context = new TextEncoder().encode("test");
            const invalidSignature = new Uint8Array(63);
            await wasm.verify_proof_secure_wasm(publicKey, context, invalidSignature);
        } catch (error) {
            errorTypes.add(error.errorType);
        }
        
        // Should have collected different error types
        expect(errorTypes.size).toBeGreaterThan(1);
        expect(errorTypes.has('InvalidPublicKey')).toBe(true);
        expect(errorTypes.has('EmptyContext')).toBe(true);
        expect(errorTypes.has('InvalidSignature')).toBe(true);
    });
});

describe('Backward Compatibility', () => {
    
    test('legacy WasmKeyPair should still work but with enhanced errors', async () => {
        const keypair = new wasm.WasmKeyPair();
        const context = new TextEncoder().encode("legacy test");
        
        // Should work with basic signing
        const signature = await keypair.sign(context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Should work with secure signing
        const secureSignature = await keypair.sign_secure(context);
        expect(secureSignature).toBeInstanceOf(Uint8Array);
        expect(secureSignature.length).toBe(64);
    });
    
    test('legacy verify_proof_wasm should still work with enhanced errors', async () => {
        const keypair = new wasm.WasmKeyPair();
        const publicKey = keypair.public_key_bytes();
        const context = new TextEncoder().encode("legacy verify test");
        const signature = await keypair.sign(context);
        
        // Should verify successfully
        const isValid = await wasm.verify_proof_wasm(publicKey, context, signature);
        expect(isValid).toBe(true);
        
        // Should throw enhanced error for invalid input
        try {
            const invalidPublicKey = new Uint8Array(31);
            await wasm.verify_proof_wasm(invalidPublicKey, context, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.errorType).toBe('InvalidPublicKey');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
});