//! TDD Step 3: Tests for Enhanced Key Store with Persistence
//!
//! This test suite verifies that the persistent key store maintains all security
//! properties of the original key store while adding secure persistence capabilities.

import { describe, it, expect, beforeAll, beforeEach, afterEach, vi } from 'vitest';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';
import init, { WasmKeyPair } from '../pkg/proof_messenger_web.js';
import { 
    usePersistentKeyStore,
    usePersistentKeyStoreReady,
    usePersistentPublicKey,
    usePersistentSigningFunction,
    usePersistentKeyStoreStatus,
    usePersistenceOperations,
    validatePersistentStoreSecurityProperties,
    getPersistentStoreDiagnostics
} from '../src/persistent-key-store.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Initialize WASM once for all tests
beforeAll(async () => {
    const wasmPath = path.join(__dirname, '../pkg/proof_messenger_web_bg.wasm');
    const wasmBytes = readFileSync(wasmPath);
    await init(wasmBytes);
});

// Mock localStorage for Node.js testing
const localStorageMock = {
    store: {},
    getItem: vi.fn((key) => localStorageMock.store[key] || null),
    setItem: vi.fn((key, value) => {
        localStorageMock.store[key] = value;
    }),
    removeItem: vi.fn((key) => {
        delete localStorageMock.store[key];
    }),
    clear: vi.fn(() => {
        localStorageMock.store = {};
    }),
    key: vi.fn((index) => {
        const keys = Object.keys(localStorageMock.store);
        return keys[index] || null;
    }),
    get length() {
        return Object.keys(localStorageMock.store).length;
    }
};

// Setup localStorage mock
beforeEach(() => {
    global.localStorage = localStorageMock;
    localStorageMock.clear();
    vi.clearAllMocks();
    usePersistentKeyStore.getState().reset();
});

afterEach(() => {
    localStorageMock.clear();
});

describe('PersistentKeyStore - Basic Functionality', () => {
    it('should maintain all original key store functionality', () => {
        // ARRANGE: Get the initial state
        const initialState = usePersistentKeyStore.getState();
        expect(initialState.keyPairInstance).toBeNull();
        expect(initialState.publicKeyHex).toBeNull();
        expect(initialState.status).toBe('uninitialized');

        // ACT: Generate a keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();

        // ASSERT: Check the new state
        const newState = usePersistentKeyStore.getState();
        expect(newState.keyPairInstance).toBeInstanceOf(WasmKeyPair);
        expect(newState.publicKeyHex).toMatch(/^[0-9a-f]{64}$/);
        expect(newState.status).toBe('ready');
        expect(newState.isReady()).toBe(true);
    });

    it('should be able to sign data like the original store', () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const context = new TextEncoder().encode('test context for signing');

        // ACT: Sign data
        const signature = usePersistentKeyStore.getState().sign(context);

        // ASSERT: Valid signature
        expect(signature).toBeInstanceOf(Uint8Array);
        expect(signature.length).toBe(64);
    });

    it('should throw error when trying to sign without a keypair', () => {
        // ARRANGE: Ensure no keypair is generated
        const context = new TextEncoder().encode('test context');

        // ACT & ASSERT: Should throw error
        expect(() => {
            usePersistentKeyStore.getState().sign(context);
        }).toThrow('Cannot sign: Keypair not initialized');
    });

    it('should reset state properly including persistence info', () => {
        // ARRANGE: Generate keypair and set some persistence state
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        usePersistentKeyStore.setState({ 
            isPersistedSession: true, 
            lastSaveTime: Date.now(),
            storageKey: 'test-key'
        });

        // ACT: Reset state
        usePersistentKeyStore.getState().reset();

        // ASSERT: All state is reset
        const state = usePersistentKeyStore.getState();
        expect(state.keyPairInstance).toBeNull();
        expect(state.publicKeyHex).toBeNull();
        expect(state.status).toBe('uninitialized');
        expect(state.isPersistedSession).toBe(false);
        expect(state.lastSaveTime).toBeNull();
        expect(state.storageKey).toBeNull();
    });
});

describe('PersistentKeyStore - Persistence Operations', () => {
    it('should save keypair to storage with encryption', async () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'TestSavePassword123!';
        const storageKey = 'test-save-keypair';

        // ACT: Save keypair
        const success = await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);

        // ASSERT: Save was successful
        expect(success).toBe(true);
        
        // Check state updates
        const state = usePersistentKeyStore.getState();
        expect(state.status).toBe('ready');
        expect(state.isPersistedSession).toBe(true);
        expect(state.lastSaveTime).toBeGreaterThan(0);
        expect(state.storageKey).toBe(storageKey);

        // Check that data was actually stored
        expect(localStorage.setItem).toHaveBeenCalledWith(storageKey, expect.any(String));
    });

    it('should load keypair from storage with decryption', async () => {
        // ARRANGE: Generate and save a keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const originalPublicKey = usePersistentKeyStore.getState().publicKeyHex;
        const password = 'TestLoadPassword123!';
        const storageKey = 'test-load-keypair';

        await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);

        // Reset store to simulate fresh session
        usePersistentKeyStore.getState().reset();
        expect(usePersistentKeyStore.getState().keyPairInstance).toBeNull();

        // ACT: Load keypair
        const success = await usePersistentKeyStore.getState().loadKeypairFromStorage(password, storageKey);

        // ASSERT: Load was successful
        expect(success).toBe(true);

        // Check state updates
        const state = usePersistentKeyStore.getState();
        expect(state.status).toBe('ready');
        expect(state.isPersistedSession).toBe(true);
        expect(state.storageKey).toBe(storageKey);
        expect(state.keyPairInstance).not.toBeNull();
        expect(state.publicKeyHex).toBe(originalPublicKey);
    });

    it('should return false when loading non-existent keypair', async () => {
        // ACT: Try to load non-existent keypair
        const success = await usePersistentKeyStore.getState().loadKeypairFromStorage(
            'TestPassword123!', 
            'non-existent-key'
        );

        // ASSERT: Load failed gracefully
        expect(success).toBe(false);

        // State should remain uninitialized
        const state = usePersistentKeyStore.getState();
        expect(state.status).toBe('uninitialized');
        expect(state.keyPairInstance).toBeNull();
        expect(state.isPersistedSession).toBe(false);
    });

    it('should handle wrong password gracefully', async () => {
        // ARRANGE: Save keypair with one password
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const correctPassword = 'CorrectPassword123!';
        const wrongPassword = 'WrongPassword456!';
        const storageKey = 'test-wrong-password';

        await usePersistentKeyStore.getState().saveKeypairToStorage(correctPassword, storageKey);
        usePersistentKeyStore.getState().reset();

        // ACT & ASSERT: Should throw error with wrong password
        await expect(
            usePersistentKeyStore.getState().loadKeypairFromStorage(wrongPassword, storageKey)
        ).rejects.toThrow();

        // State should be in error state
        const state = usePersistentKeyStore.getState();
        expect(state.status).toBe('error');
        expect(state.keyPairInstance).toBeNull();
    });

    it('should check if keypair exists in storage', () => {
        // ARRANGE: Put some data in storage
        localStorageMock.store['existing-keypair'] = 'encrypted-data';

        // ACT & ASSERT: Check existence
        expect(usePersistentKeyStore.getState().hasKeypairInStorage('existing-keypair')).toBe(true);
        expect(usePersistentKeyStore.getState().hasKeypairInStorage('non-existent-keypair')).toBe(false);
    });

    it('should remove keypair from storage', async () => {
        // ARRANGE: Save a keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'RemoveTestPassword123!';
        const storageKey = 'keypair-to-remove';

        await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);
        expect(usePersistentKeyStore.getState().hasKeypairInStorage(storageKey)).toBe(true);

        // ACT: Remove keypair
        usePersistentKeyStore.getState().removeKeypairFromStorage(storageKey);

        // ASSERT: Keypair is removed
        expect(localStorage.removeItem).toHaveBeenCalledWith(storageKey);
        expect(usePersistentKeyStore.getState().hasKeypairInStorage(storageKey)).toBe(false);

        // State should be updated if it was the current keypair
        const state = usePersistentKeyStore.getState();
        expect(state.isPersistedSession).toBe(false);
        expect(state.storageKey).toBeNull();
    });

    it('should list stored keypairs', () => {
        // ARRANGE: Put multiple keypairs in storage
        localStorageMock.store['keypair-user1'] = 'encrypted-data-1';
        localStorageMock.store['keypair-user2'] = 'encrypted-data-2';
        localStorageMock.store['other-data'] = 'not-a-keypair';

        // ACT: List keypairs with prefix
        const keypairKeys = usePersistentKeyStore.getState().listStoredKeypairs('keypair-');

        // ASSERT: Should return only keypair keys
        expect(keypairKeys).toEqual(['keypair-user1', 'keypair-user2']);
        expect(keypairKeys).not.toContain('other-data');
    });
});

describe('PersistentKeyStore - Auto-save and Convenience Methods', () => {
    it('should auto-save using current storage key', async () => {
        // ARRANGE: Generate keypair and save it once to set storage key
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'AutoSavePassword123!';
        const storageKey = 'auto-save-test';

        await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);

        // Simulate some changes (generate new keypair)
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        expect(usePersistentKeyStore.getState().isPersistedSession).toBe(false);

        // ACT: Auto-save
        const success = await usePersistentKeyStore.getState().autoSave(password);

        // ASSERT: Auto-save used the previous storage key
        expect(success).toBe(true);
        expect(usePersistentKeyStore.getState().storageKey).toBe(storageKey);
        expect(usePersistentKeyStore.getState().isPersistedSession).toBe(true);
    });

    it('should provide persistence status information', async () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();

        // ACT: Get initial status
        let status = usePersistentKeyStore.getState().getPersistenceStatus();

        // ASSERT: Initial status
        expect(status.isPersistedSession).toBe(false);
        expect(status.canSave).toBe(true);
        expect(status.hasUnsavedChanges).toBe(true);
        expect(status.lastSaveTime).toBeNull();
        expect(status.storageKey).toBeNull();

        // ARRANGE: Save keypair
        const password = 'StatusTestPassword123!';
        const storageKey = 'status-test';
        await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);

        // ACT: Get status after save
        status = usePersistentKeyStore.getState().getPersistenceStatus();

        // ASSERT: Status after save
        expect(status.isPersistedSession).toBe(true);
        expect(status.canSave).toBe(true);
        expect(status.hasUnsavedChanges).toBe(false);
        expect(status.lastSaveTime).toBeGreaterThan(0);
        expect(status.storageKey).toBe(storageKey);
    });
});

describe('PersistentKeyStore - Status Management', () => {
    it('should track loading status during load operation', async () => {
        // ARRANGE: Save a keypair first
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'LoadingStatusPassword123!';
        const storageKey = 'loading-status-test';
        await usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);
        usePersistentKeyStore.getState().reset();

        // ACT: Start loading (don't await immediately)
        const loadPromise = usePersistentKeyStore.getState().loadKeypairFromStorage(password, storageKey);

        // ASSERT: Should be in loading state
        expect(usePersistentKeyStore.getState().status).toBe('loading');
        expect(usePersistentKeyStore.getState().isLoading()).toBe(true);

        // Wait for completion
        await loadPromise;

        // ASSERT: Should be ready after loading
        expect(usePersistentKeyStore.getState().status).toBe('ready');
        expect(usePersistentKeyStore.getState().isLoading()).toBe(false);
    });

    it('should track saving status during save operation', async () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'SavingStatusPassword123!';
        const storageKey = 'saving-status-test';

        // ACT: Start saving (don't await immediately)
        const savePromise = usePersistentKeyStore.getState().saveKeypairToStorage(password, storageKey);

        // ASSERT: Should be in saving state
        expect(usePersistentKeyStore.getState().status).toBe('saving');
        expect(usePersistentKeyStore.getState().isLoading()).toBe(true);

        // Wait for completion
        await savePromise;

        // ASSERT: Should be ready after saving
        expect(usePersistentKeyStore.getState().status).toBe('ready');
        expect(usePersistentKeyStore.getState().isLoading()).toBe(false);
    });

    it('should handle save errors gracefully', async () => {
        // ARRANGE: Generate keypair but mock storage to fail
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const originalSetItem = localStorage.setItem;
        localStorage.setItem = vi.fn(() => {
            throw new Error('Storage quota exceeded');
        });

        // ACT & ASSERT: Should throw error but maintain state
        await expect(
            usePersistentKeyStore.getState().saveKeypairToStorage('TestPassword123!', 'failing-save')
        ).rejects.toThrow();

        // State should return to ready (not error)
        expect(usePersistentKeyStore.getState().status).toBe('ready');
        expect(usePersistentKeyStore.getState().keyPairInstance).not.toBeNull();

        // Restore original method
        localStorage.setItem = originalSetItem;
    });
});

describe('PersistentKeyStore - React Hooks', () => {
    it('should export hook functions', () => {
        // ASSERT: Hook functions are exported and are functions
        expect(typeof usePersistentKeyStoreReady).toBe('function');
        expect(typeof usePersistentPublicKey).toBe('function');
        expect(typeof usePersistentSigningFunction).toBe('function');
        expect(typeof usePersistentKeyStoreStatus).toBe('function');
        expect(typeof usePersistenceOperations).toBe('function');
    });

    it.skip('should provide ready status hook', () => {
        // NOTE: React hooks can only be tested within React components
        // This test is skipped as it requires a React testing environment
        // In a real application, these would be tested with @testing-library/react
    });

    it.skip('should provide public key hook', () => {
        // NOTE: React hooks can only be tested within React components
    });

    it.skip('should provide signing function hook', () => {
        // NOTE: React hooks can only be tested within React components
    });

    it.skip('should provide comprehensive status hook', () => {
        // NOTE: React hooks can only be tested within React components
    });

    it.skip('should provide persistence operations hook', () => {
        // NOTE: React hooks can only be tested within React components
    });
});

describe('PersistentKeyStore - Security Properties', () => {
    it('should maintain security properties validation', () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();

        // ACT: Validate security properties
        const isSecure = validatePersistentStoreSecurityProperties();

        // ASSERT: Security properties maintained
        expect(isSecure).toBe(true);
    });

    it('should not expose sensitive data in diagnostics', () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();

        // ACT: Get diagnostics
        const diagnostics = getPersistentStoreDiagnostics();

        // ASSERT: No sensitive data exposed
        expect(diagnostics).not.toHaveProperty('privateKey');
        expect(diagnostics).not.toHaveProperty('password');
        expect(diagnostics.sensitiveDataExcluded).toContain('password');
        expect(diagnostics.hasKeyPair).toBe(true);
        expect(diagnostics.persistenceStatus).toBeDefined();
    });

    it('should not leak passwords in state', async () => {
        // ARRANGE: Generate and save keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'SecretPassword123!';
        
        // ACT: Save keypair
        await usePersistentKeyStore.getState().saveKeypairToStorage(password, 'security-test');

        // ASSERT: Password not stored in state
        const state = usePersistentKeyStore.getState();
        const stateString = JSON.stringify(state);
        
        expect(stateString).not.toContain(password);
        expect(stateString).not.toContain('SecretPassword');
        expect(state).not.toHaveProperty('password');
    });

    it('should handle concurrent operations safely', async () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const password = 'ConcurrentPassword123!';

        // ACT: Perform concurrent save operations
        const operations = [
            usePersistentKeyStore.getState().saveKeypairToStorage(password, 'concurrent-1'),
            usePersistentKeyStore.getState().saveKeypairToStorage(password, 'concurrent-2'),
            usePersistentKeyStore.getState().saveKeypairToStorage(password, 'concurrent-3'),
        ];

        // ASSERT: All operations should complete
        const results = await Promise.all(operations);
        results.forEach(result => {
            expect(result).toBe(true);
        });

        // Final state should be consistent
        const finalState = usePersistentKeyStore.getState();
        expect(finalState.status).toBe('ready');
        expect(finalState.keyPairInstance).not.toBeNull();
    });
});

describe('PersistentKeyStore - Error Handling', () => {
    it('should handle save without keypair', async () => {
        // ARRANGE: No keypair generated
        expect(usePersistentKeyStore.getState().keyPairInstance).toBeNull();

        // ACT & ASSERT: Should throw error
        await expect(
            usePersistentKeyStore.getState().saveKeypairToStorage('TestPassword123!', 'no-keypair-test')
        ).rejects.toThrow('Cannot save: No keypair initialized');
    });

    it('should handle invalid storage operations gracefully', async () => {
        // ACT & ASSERT: Invalid storage key
        await expect(
            usePersistentKeyStore.getState().saveKeypairToStorage('TestPassword123!', '')
        ).rejects.toThrow();

        // Invalid password
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        await expect(
            usePersistentKeyStore.getState().saveKeypairToStorage('weak', 'test-key')
        ).rejects.toThrow();
    });

    it('should maintain state consistency after errors', async () => {
        // ARRANGE: Generate keypair
        usePersistentKeyStore.getState().generateAndStoreKeyPair();
        const originalState = {
            publicKeyHex: usePersistentKeyStore.getState().publicKeyHex,
            status: usePersistentKeyStore.getState().status,
        };

        // ACT: Trigger save error
        try {
            await usePersistentKeyStore.getState().saveKeypairToStorage('weak', 'error-test');
        } catch (error) {
            // Expected to fail
        }

        // ASSERT: State should be preserved
        const currentState = usePersistentKeyStore.getState();
        expect(currentState.publicKeyHex).toBe(originalState.publicKeyHex);
        expect(currentState.status).toBe('ready'); // Should return to ready state
        expect(currentState.keyPairInstance).not.toBeNull();
    });
});