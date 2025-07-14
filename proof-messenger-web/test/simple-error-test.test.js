// Simple Error Handling Test for WASM Integration
import { expect, test, describe, beforeAll } from 'vitest';

let wasm;

beforeAll(async () => {
    // Import the CommonJS module
    wasm = await import('../pkg/proof_messenger_web.js');
});

describe('Basic Rich Error Propagation', () => {
    
    test('verify_proof_wasm should throw enhanced error for invalid public key', async () => {
        const invalidPublicKey = new Uint8Array(31); // Wrong length
        const context = new TextEncoder().encode("test context");
        const signature = new Uint8Array(64);
        
        try {
            wasm.verify_proof_wasm(invalidPublicKey, context, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Invalid public key format');
            expect(error.errorType).toBe('InvalidPublicKey');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('verify_proof_wasm should throw enhanced error for invalid signature', async () => {
        const keypair = new wasm.WasmKeyPair();
        const publicKey = keypair.public_key_bytes;
        const context = new TextEncoder().encode("test context");
        const invalidSignature = new Uint8Array(63); // Wrong length
        
        try {
            wasm.verify_proof_wasm(publicKey, context, invalidSignature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Invalid signature format');
            expect(error.errorType).toBe('InvalidSignature');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('WasmKeyPair should work correctly with valid inputs', async () => {
        const keypair = new wasm.WasmKeyPair();
        const context = new TextEncoder().encode("valid test context");
        
        // Sign the context
        const signature = keypair.sign(context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Verify the signature
        const publicKey = keypair.public_key_bytes;
        const isValid = wasm.verify_proof_wasm(publicKey, context, signature);
        expect(isValid).toBe(true);
    });
    
    test('make_secure_proof_wasm should work with valid inputs', async () => {
        const keypair = new wasm.WasmKeyPair();
        const keypairBytes = keypair.keypair_bytes;
        const context = new TextEncoder().encode("secure proof test");
        
        // Create secure proof
        const signature = wasm.make_secure_proof_wasm(keypairBytes, context);
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
        
        // Verify secure proof
        const publicKey = keypair.public_key_bytes;
        const isValid = wasm.verify_proof_secure_wasm(publicKey, context, signature);
        expect(isValid).toBe(true);
    });
    
    test('make_secure_proof_strict_wasm should throw empty context error', async () => {
        const keypair = new wasm.WasmKeyPair();
        const keypairBytes = keypair.keypair_bytes;
        const emptyContext = new Uint8Array(0);
        
        try {
            wasm.make_secure_proof_strict_wasm(keypairBytes, emptyContext);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Context data cannot be empty');
            expect(error.errorType).toBe('EmptyContext');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('verify_proof_strict_wasm should throw empty context error', async () => {
        const keypair = new wasm.WasmKeyPair();
        const publicKey = keypair.public_key_bytes;
        const emptyContext = new Uint8Array(0);
        const signature = new Uint8Array(64);
        
        try {
            wasm.verify_proof_strict_wasm(publicKey, emptyContext, signature);
            expect.fail('Should have thrown an error');
        } catch (error) {
            expect(error.message).toContain('Context data cannot be empty');
            expect(error.errorType).toBe('EmptyContext');
            expect(error.isProofMessengerError).toBe(true);
        }
    });
    
    test('error objects should have consistent structure', async () => {
        const invalidPublicKey = new Uint8Array(31);
        const context = new TextEncoder().encode("test");
        const signature = new Uint8Array(64);
        
        try {
            wasm.verify_proof_wasm(invalidPublicKey, context, signature);
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
});