//! TDD Step 3: Enhanced Key Store with Persistence
//!
//! This module extends the existing useKeyStore with secure persistence capabilities
//! using the SecureStorage module. It maintains all security properties while adding
//! the ability to save and restore keypairs across browser sessions.

import { create } from 'zustand';
import { WasmKeyPair } from '../pkg/proof_messenger_web.js';
import { SecureStorage } from './secure-storage.js';

/**
 * Enhanced key store with secure persistence capabilities
 * 
 * Security principles:
 * 1. Private keys never leave the WASM boundary
 * 2. Stored keypairs are encrypted with user-provided passwords
 * 3. All cryptographic operations happen within WASM
 * 4. State is designed to be serialization-safe (no sensitive data)
 * 5. Password is never stored in memory longer than necessary
 */
export const usePersistentKeyStore = create((set, get) => ({
    // State properties
    keyPairInstance: null,      // The WASM keypair instance (encapsulates private key)
    publicKeyHex: null,         // Derived public key (safe to expose)
    status: 'uninitialized',    // 'uninitialized' | 'generating' | 'ready' | 'error' | 'loading' | 'saving'
    isPersistedSession: false,  // Whether current keypair was loaded from storage
    lastSaveTime: null,         // Timestamp of last successful save
    storageKey: null,           // Current storage key being used

    // Computed properties
    isReady: () => {
        const { status } = get();
        return status === 'ready';
    },

    isLoading: () => {
        const { status } = get();
        return status === 'loading' || status === 'saving';
    },

    // Actions

    /**
     * Generate and store a new keypair instance
     * Same as original useKeyStore but updates persistence status
     */
    generateAndStoreKeyPair: () => {
        set({ status: 'generating' });
        
        try {
            const keyPairInstance = new WasmKeyPair();
            const publicKeyHex = keyPairInstance.public_key_hex;
            
            // Preserve storage key if it exists (for auto-save functionality)
            const { storageKey: currentStorageKey } = get();
            
            set({
                keyPairInstance,
                publicKeyHex,
                status: 'ready',
                isPersistedSession: false, // New keypair, not from storage
                lastSaveTime: null,
                storageKey: currentStorageKey, // Preserve existing storage key
            });
            
        } catch (error) {
            set({
                keyPairInstance: null,
                publicKeyHex: null,
                status: 'error',
                isPersistedSession: false,
                lastSaveTime: null,
                storageKey: null,
            });
            
            console.error("Failed to generate keypair:", error);
        }
    },

    /**
     * Save current keypair to encrypted storage
     * @param {string} password - Encryption password
     * @param {string} storageKey - Storage key (optional, defaults to 'default-keypair')
     * @returns {Promise<boolean>} Success status
     */
    saveKeypairToStorage: async (password, storageKey = 'default-keypair') => {
        const { keyPairInstance } = get();
        
        if (!keyPairInstance) {
            throw new Error('Cannot save: No keypair initialized.');
        }

        set({ status: 'saving' });

        try {
            await SecureStorage.saveKeypairToStorage(keyPairInstance, password, storageKey);
            
            set({
                status: 'ready',
                isPersistedSession: true,
                lastSaveTime: Date.now(),
                storageKey: storageKey,
            });

            return true;

        } catch (error) {
            set({ status: 'ready' }); // Return to ready state, but don't change persistence status
            console.error("Failed to save keypair:", error);
            throw error; // Re-throw for caller to handle
        }
    },

    /**
     * Load keypair from encrypted storage
     * @param {string} password - Decryption password
     * @param {string} storageKey - Storage key (optional, defaults to 'default-keypair')
     * @returns {Promise<boolean>} Success status
     */
    loadKeypairFromStorage: async (password, storageKey = 'default-keypair') => {
        set({ status: 'loading' });

        try {
            const keyPairInstance = await SecureStorage.loadKeypairFromStorage(password, storageKey);
            
            if (!keyPairInstance) {
                // No keypair found in storage
                set({
                    status: 'uninitialized',
                    keyPairInstance: null,
                    publicKeyHex: null,
                    isPersistedSession: false,
                    lastSaveTime: null,
                    storageKey: null,
                });
                return false;
            }

            // Successfully loaded keypair
            set({
                keyPairInstance,
                publicKeyHex: keyPairInstance.public_key_hex,
                status: 'ready',
                isPersistedSession: true,
                lastSaveTime: null, // We don't know when it was last saved
                storageKey: storageKey,
            });

            return true;

        } catch (error) {
            set({
                status: 'error',
                keyPairInstance: null,
                publicKeyHex: null,
                isPersistedSession: false,
                lastSaveTime: null,
                storageKey: null,
            });
            
            console.error("Failed to load keypair:", error);
            throw error; // Re-throw for caller to handle
        }
    },

    /**
     * Check if a keypair exists in storage
     * @param {string} storageKey - Storage key to check
     * @returns {boolean} True if keypair exists
     */
    hasKeypairInStorage: (storageKey = 'default-keypair') => {
        return SecureStorage.hasKeypairInStorage(storageKey);
    },

    /**
     * Remove keypair from storage
     * @param {string} storageKey - Storage key to remove
     */
    removeKeypairFromStorage: (storageKey = 'default-keypair') => {
        SecureStorage.removeKeypairFromStorage(storageKey);
        
        // If we just removed the currently loaded keypair, update state
        const { storageKey: currentStorageKey } = get();
        if (currentStorageKey === storageKey) {
            set({
                isPersistedSession: false,
                lastSaveTime: null,
                storageKey: null,
            });
        }
    },

    /**
     * List all stored keypairs
     * @param {string} prefix - Prefix to filter storage keys
     * @returns {string[]} Array of storage keys
     */
    listStoredKeypairs: (prefix = '') => {
        return SecureStorage.listStoredKeypairs(prefix);
    },

    /**
     * Sign data using the stored keypair
     * Same as original useKeyStore
     */
    sign: (contextBytes) => {
        const { keyPairInstance } = get();
        
        if (!keyPairInstance) {
            throw new Error('Cannot sign: Keypair not initialized.');
        }
        
        return keyPairInstance.sign(contextBytes);
    },

    /**
     * Reset the store to initial state
     * Enhanced to clear persistence information
     */
    reset: () => {
        set({
            keyPairInstance: null,
            publicKeyHex: null,
            status: 'uninitialized',
            isPersistedSession: false,
            lastSaveTime: null,
            storageKey: null,
        });
    },

    /**
     * Auto-save current keypair (convenience method)
     * @param {string} password - Encryption password
     * @returns {Promise<boolean>} Success status
     */
    autoSave: async (password) => {
        const { storageKey } = get();
        const keyToUse = storageKey || 'default-keypair';
        return await get().saveKeypairToStorage(password, keyToUse);
    },

    /**
     * Get persistence status information
     * @returns {Object} Persistence status details
     */
    getPersistenceStatus: () => {
        const { isPersistedSession, lastSaveTime, storageKey, status } = get();
        return {
            isPersistedSession,
            lastSaveTime,
            storageKey,
            canSave: status === 'ready',
            hasUnsavedChanges: status === 'ready' && !isPersistedSession,
        };
    },
}));

// Enhanced hooks for React components

/**
 * Hook for React components that only need to know if a keypair is ready
 */
export const usePersistentKeyStoreReady = () => {
    return usePersistentKeyStore((state) => state.isReady());
};

/**
 * Hook for React components that only need the public key
 */
export const usePersistentPublicKey = () => {
    return usePersistentKeyStore((state) => state.publicKeyHex);
};

/**
 * Hook for React components that need signing capability
 */
export const usePersistentSigningFunction = () => {
    return usePersistentKeyStore((state) => state.sign);
};

/**
 * Hook for React components that need to track keypair status
 */
export const usePersistentKeyStoreStatus = () => {
    return usePersistentKeyStore((state) => ({
        status: state.status,
        isReady: state.isReady(),
        isLoading: state.isLoading(),
        hasKeyPair: state.keyPairInstance !== null,
        persistenceStatus: state.getPersistenceStatus(),
    }));
};

/**
 * Hook for React components that need persistence operations
 */
export const usePersistenceOperations = () => {
    return usePersistentKeyStore((state) => ({
        saveKeypair: state.saveKeypairToStorage,
        loadKeypair: state.loadKeypairFromStorage,
        hasKeypairInStorage: state.hasKeypairInStorage,
        removeKeypair: state.removeKeypairFromStorage,
        listKeypairs: state.listStoredKeypairs,
        autoSave: state.autoSave,
    }));
};

/**
 * Utility function to validate that the enhanced store maintains security properties
 */
export const validatePersistentStoreSecurityProperties = () => {
    const state = usePersistentKeyStore.getState();
    
    // Check that no sensitive properties are exposed
    const sensitiveProps = ['privateKey', 'secretKey', 'seed', 'entropy', 'privateKeyHex', 'password'];
    const violations = [];
    
    for (const prop of sensitiveProps) {
        if (state.hasOwnProperty(prop)) {
            violations.push(`Sensitive property '${prop}' found in store state`);
        }
    }
    
    // Check that serialization doesn't expose sensitive data
    try {
        const serialized = JSON.stringify(state);
        const sensitivePatterns = [/private/i, /secret/i, /seed/i, /entropy/i, /password/i];
        
        for (const pattern of sensitivePatterns) {
            if (pattern.test(serialized)) {
                violations.push(`Sensitive data pattern '${pattern}' found in serialized state`);
            }
        }
    } catch (e) {
        // Serialization failure is actually good for security
    }
    
    if (violations.length > 0) {
        console.warn('Security violations detected in persistent key store:', violations);
        return false;
    }
    
    return true;
};

/**
 * Development helper to inspect persistent store state safely
 */
export const getPersistentStoreDiagnostics = () => {
    const state = usePersistentKeyStore.getState();
    
    return {
        status: state.status,
        hasKeyPair: state.keyPairInstance !== null,
        hasPublicKey: state.publicKeyHex !== null,
        publicKeyLength: state.publicKeyHex ? state.publicKeyHex.length : 0,
        isReady: state.isReady(),
        isLoading: state.isLoading(),
        persistenceStatus: state.getPersistenceStatus(),
        keyPairType: state.keyPairInstance ? state.keyPairInstance.constructor.name : null,
        // Explicitly exclude sensitive data
        sensitiveDataExcluded: [
            'privateKey', 'secretKey', 'seed', 'entropy', 
            'keyPairInstance (contains private key)', 'password'
        ],
    };
};