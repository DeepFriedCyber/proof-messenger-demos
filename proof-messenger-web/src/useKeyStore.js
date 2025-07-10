//! TDD Step 2: Secure and Testable State Management Implementation
//!
//! This Zustand store encapsulates the WasmKeyPair instance, exposing only safe
//! actions and derived state to React components. The private key never leaves
//! the WASM boundary, ensuring maximum security.

import { create } from 'zustand';
import { WasmKeyPair } from '../pkg/proof_messenger_web.js';

/**
 * Secure key store that manages WasmKeyPair instances without exposing private keys
 * 
 * Security principles:
 * 1. Private keys never leave the WASM boundary
 * 2. Only public keys and signatures are exposed to JavaScript
 * 3. All cryptographic operations happen within WASM
 * 4. State is designed to be serialization-safe (no sensitive data)
 */
export const useKeyStore = create((set, get) => ({
    // State properties
    keyPairInstance: null,      // The WASM keypair instance (encapsulates private key)
    publicKeyHex: null,         // Derived public key (safe to expose)
    status: 'uninitialized',    // 'uninitialized' | 'generating' | 'ready' | 'error'

    // Computed properties
    isReady: () => {
        const { status } = get();
        return status === 'ready';
    },

    // Actions
    
    /**
     * Generate and store a new keypair instance
     * 
     * This action:
     * 1. Creates a new WasmKeyPair instance (private key stays in WASM)
     * 2. Extracts only the public key for JavaScript access
     * 3. Updates status to track the operation state
     * 4. Handles errors gracefully without exposing sensitive information
     */
    generateAndStoreKeyPair: () => {
        set({ status: 'generating' });
        
        try {
            // Create new WASM keypair instance
            // The private key is generated and stored entirely within WASM
            const keyPairInstance = new WasmKeyPair();
            
            // Extract only the public key for JavaScript access
            const publicKeyHex = keyPairInstance.public_key_hex;
            
            // Update state with safe, non-sensitive data
            set({
                keyPairInstance,
                publicKeyHex,
                status: 'ready',
            });
            
        } catch (error) {
            // Handle errors without exposing sensitive information
            set({
                keyPairInstance: null,
                publicKeyHex: null,
                status: 'error'
            });
            
            console.error("Failed to generate keypair:", error);
        }
    },

    /**
     * Sign data using the stored keypair
     * 
     * This action:
     * 1. Validates that a keypair is available
     * 2. Delegates signing to the WASM instance (private key never exposed)
     * 3. Returns only the signature bytes
     * 
     * @param {Uint8Array} contextBytes - The data to sign
     * @returns {Uint8Array} The signature bytes
     * @throws {Error} If no keypair is initialized
     */
    sign: (contextBytes) => {
        const { keyPairInstance } = get();
        
        if (!keyPairInstance) {
            throw new Error('Cannot sign: Keypair not initialized.');
        }
        
        // Delegate to WASM - private key never leaves WASM boundary
        return keyPairInstance.sign(contextBytes);
    },

    /**
     * Reset the store to initial state
     * 
     * This action:
     * 1. Clears all state including the WASM instance
     * 2. Allows the WASM instance to be garbage collected
     * 3. Resets status to uninitialized
     */
    reset: () => {
        set({
            keyPairInstance: null,
            publicKeyHex: null,
            status: 'uninitialized',
        });
    },
}));

// Additional security measures and utilities

/**
 * Hook for React components that only need to know if a keypair is ready
 * This prevents components from accessing the full store unnecessarily
 */
export const useKeyStoreReady = () => {
    return useKeyStore((state) => state.isReady());
};

/**
 * Hook for React components that only need the public key
 * This prevents components from accessing the keypair instance
 */
export const usePublicKey = () => {
    return useKeyStore((state) => state.publicKeyHex);
};

/**
 * Hook for React components that need signing capability
 * This provides a safe signing function without exposing the keypair instance
 */
export const useSigningFunction = () => {
    return useKeyStore((state) => state.sign);
};

/**
 * Hook for React components that need to track keypair status
 * This provides status information for UI feedback
 */
export const useKeyStoreStatus = () => {
    return useKeyStore((state) => ({
        status: state.status,
        isReady: state.isReady(),
        hasKeyPair: state.keyPairInstance !== null,
    }));
};

/**
 * Utility function to validate that the store is in a secure state
 * This can be used in development/testing to ensure no sensitive data leaks
 */
export const validateStoreSecurityProperties = () => {
    const state = useKeyStore.getState();
    
    // Check that no sensitive properties are exposed
    const sensitiveProps = ['privateKey', 'secretKey', 'seed', 'entropy', 'privateKeyHex'];
    const violations = [];
    
    for (const prop of sensitiveProps) {
        if (state.hasOwnProperty(prop)) {
            violations.push(`Sensitive property '${prop}' found in store state`);
        }
    }
    
    // Check that serialization doesn't expose sensitive data
    try {
        const serialized = JSON.stringify(state);
        const sensitivePatterns = [/private/i, /secret/i, /seed/i, /entropy/i];
        
        for (const pattern of sensitivePatterns) {
            if (pattern.test(serialized)) {
                violations.push(`Sensitive data pattern '${pattern}' found in serialized state`);
            }
        }
    } catch (e) {
        // Serialization failure is actually good for security
        // (means WASM objects can't be accidentally serialized)
    }
    
    if (violations.length > 0) {
        console.warn('Security violations detected in key store:', violations);
        return false;
    }
    
    return true;
};

/**
 * Development helper to inspect store state safely
 * Only exposes non-sensitive information
 */
export const getStoreDiagnostics = () => {
    const state = useKeyStore.getState();
    
    return {
        status: state.status,
        hasKeyPair: state.keyPairInstance !== null,
        hasPublicKey: state.publicKeyHex !== null,
        publicKeyLength: state.publicKeyHex ? state.publicKeyHex.length : 0,
        isReady: state.isReady(),
        keyPairType: state.keyPairInstance ? state.keyPairInstance.constructor.name : null,
        // Explicitly exclude sensitive data
        sensitiveDataExcluded: [
            'privateKey', 'secretKey', 'seed', 'entropy', 
            'keyPairInstance (contains private key)'
        ],
    };
};