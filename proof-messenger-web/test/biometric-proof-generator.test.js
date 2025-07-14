/**
 * TDD Test Suite for Biometric Proof Generator
 * 
 * These tests verify that the frontend correctly calls the WebAuthn API
 * with the transaction context as the challenge. The backend has ZERO
 * knowledge of biometric methods - this is purely client-side testing.
 */

const { 
    triggerProofGeneration, 
    registerBiometricCredential,
    isBiometricAvailable,
    createBiometricApprovedProof,
    contextToChallenge,
    bufferToBase64url,
} = require('../src/biometric-proof-generator');

// Mock the browser's WebAuthn API
const mockCredentialsGet = jest.fn();
const mockCredentialsCreate = jest.fn();
const mockIsUserVerifyingPlatformAuthenticatorAvailable = jest.fn();

// Setup global mocks for browser APIs in beforeEach

describe('Biometric Proof Generator', () => {
    beforeEach(() => {
        // Reset all mocks before each test
        jest.clearAllMocks();
        
        // Setup global mocks for browser APIs
        global.navigator = {
            credentials: {
                get: mockCredentialsGet,
                create: mockCredentialsCreate,
            },
            userAgent: 'Mozilla/5.0 (Test Browser)',
            platform: 'Test Platform',
        };

        global.window = {
            location: {
                hostname: 'proof-messenger.example.com',
            },
            PublicKeyCredential: {
                isUserVerifyingPlatformAuthenticatorAvailable: mockIsUserVerifyingPlatformAuthenticatorAvailable,
            },
        };
    });

    describe('triggerProofGeneration', () => {
        /**
         * TDD Test Case 1: Core requirement - WebAuthn API called with transaction context as challenge
         * This test verifies the corrected architectural approach
         */
        test('should call WebAuthn API with transaction context as challenge', async () => {
            // ARRANGE: Set up transaction context and mock WebAuthn response
            const transactionContext = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "user-123",
            };

            const mockAssertion = {
                rawId: new ArrayBuffer(32),
                response: {
                    authenticatorData: new ArrayBuffer(64),
                    signature: new ArrayBuffer(72),
                    userHandle: new ArrayBuffer(16),
                    clientDataJSON: new ArrayBuffer(128),
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Trigger biometric proof generation
            await triggerProofGeneration(transactionContext);

            // ASSERT: Verify WebAuthn API was called with correct parameters
            expect(mockCredentialsGet).toHaveBeenCalledTimes(1);
            
            const callArgs = mockCredentialsGet.mock.calls[0][0];
            expect(callArgs).toHaveProperty('publicKey');
            expect(callArgs.publicKey).toHaveProperty('challenge');
            expect(callArgs.publicKey).toHaveProperty('userVerification', 'required');
            expect(callArgs.publicKey).toHaveProperty('timeout', 60000);
            expect(callArgs.publicKey).toHaveProperty('rpId', 'proof-messenger.example.com');

            // Verify the challenge contains our transaction context
            const challengeBytes = callArgs.publicKey.challenge;
            const challengeString = new TextDecoder().decode(challengeBytes);
            const challengeObject = JSON.parse(challengeString);
            
            expect(challengeObject).toEqual(transactionContext);
        });

        /**
         * TDD Test Case 2: Verify transaction context validation
         */
        test('should validate required transaction context fields', async () => {
            // ARRANGE: Invalid transaction contexts
            const invalidContexts = [
                null,
                undefined,
                {},
                { action: "transfer" }, // Missing required fields
                { action: "transfer", amount: 1000 }, // Missing destination, timestamp, user_id
            ];

            // ACT & ASSERT: Each invalid context should throw an error
            for (const invalidContext of invalidContexts) {
                await expect(triggerProofGeneration(invalidContext))
                    .rejects
                    .toThrow();
            }

            // Verify WebAuthn API was never called for invalid inputs
            expect(mockCredentialsGet).not.toHaveBeenCalled();
        });

        /**
         * TDD Test Case 3: Verify proper error handling for WebAuthn failures
         */
        test('should handle WebAuthn API errors correctly', async () => {
            // ARRANGE: Valid transaction context
            const transactionContext = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "user-123",
            };

            // Test different WebAuthn error scenarios
            const errorScenarios = [
                { 
                    error: { name: 'NotAllowedError', message: 'User denied' },
                    expectedMessage: 'Biometric authentication was denied by user'
                },
                { 
                    error: { name: 'NotSupportedError', message: 'Not supported' },
                    expectedMessage: 'Biometric authentication is not supported on this device'
                },
                { 
                    error: { name: 'SecurityError', message: 'Security error' },
                    expectedMessage: 'Biometric authentication failed due to security restrictions'
                },
                { 
                    error: { name: 'UnknownError', message: 'Unknown error' },
                    expectedMessage: 'Biometric authentication failed: Unknown error'
                },
            ];

            for (const scenario of errorScenarios) {
                // ARRANGE: Mock WebAuthn to throw specific error
                mockCredentialsGet.mockRejectedValue(scenario.error);

                // ACT & ASSERT: Verify proper error handling
                await expect(triggerProofGeneration(transactionContext))
                    .rejects
                    .toThrow(scenario.expectedMessage);
                
                // Reset mock for next iteration
                mockCredentialsGet.mockReset();
            }
        });

        /**
         * TDD Test Case 4: Verify proof structure returned to backend
         */
        test('should return properly structured proof for backend verification', async () => {
            // ARRANGE: Transaction context and mock WebAuthn response
            const transactionContext = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "user-123",
            };

            const mockAssertion = {
                rawId: new Uint8Array([1, 2, 3, 4]).buffer,
                response: {
                    authenticatorData: new Uint8Array([5, 6, 7, 8]).buffer,
                    signature: new Uint8Array([9, 10, 11, 12]).buffer,
                    userHandle: new Uint8Array([13, 14, 15, 16]).buffer,
                    clientDataJSON: new Uint8Array([17, 18, 19, 20]).buffer,
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Generate proof
            const proof = await triggerProofGeneration(transactionContext);

            // ASSERT: Verify proof structure
            expect(proof).toHaveProperty('credentialId');
            expect(proof).toHaveProperty('authenticatorData');
            expect(proof).toHaveProperty('signature');
            expect(proof).toHaveProperty('userHandle');
            expect(proof).toHaveProperty('clientDataJSON');
            expect(proof).toHaveProperty('transactionContext', transactionContext);

            // Verify all fields are base64url encoded strings
            expect(typeof proof.credentialId).toBe('string');
            expect(typeof proof.authenticatorData).toBe('string');
            expect(typeof proof.signature).toBe('string');
            expect(typeof proof.clientDataJSON).toBe('string');
        });

        /**
         * TDD Test Case 5: Verify challenge encoding is correct
         */
        test('should correctly encode transaction context as WebAuthn challenge', async () => {
            // ARRANGE: Complex transaction context
            const transactionContext = {
                action: "multi_signature_approval",
                amount: 5000000,
                destination: "International Bank Ltd",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "executive-user-456",
                metadata: {
                    approval_level: "executive",
                    risk_score: "high",
                    compliance_check: "passed",
                },
            };

            const mockAssertion = {
                rawId: new ArrayBuffer(32),
                response: {
                    authenticatorData: new ArrayBuffer(64),
                    signature: new ArrayBuffer(72),
                    userHandle: null, // Test null userHandle
                    clientDataJSON: new ArrayBuffer(128),
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Trigger proof generation
            await triggerProofGeneration(transactionContext);

            // ASSERT: Verify challenge encoding
            const callArgs = mockCredentialsGet.mock.calls[0][0];
            const challengeBytes = callArgs.publicKey.challenge;
            
            // Decode challenge and verify it matches original context
            const decodedChallenge = new TextDecoder().decode(challengeBytes);
            const parsedChallenge = JSON.parse(decodedChallenge);
            
            expect(parsedChallenge).toEqual(transactionContext);
        });
    });

    describe('registerBiometricCredential', () => {
        /**
         * TDD Test Case 6: Verify biometric credential registration
         */
        test('should call WebAuthn create API for credential registration', async () => {
            // ARRANGE: User registration data
            const userId = "user-789";
            const userName = "John Doe";

            const mockCredential = {
                rawId: new Uint8Array([21, 22, 23, 24]).buffer,
                response: {
                    publicKey: new Uint8Array([25, 26, 27, 28]).buffer,
                    attestationObject: new Uint8Array([29, 30, 31, 32]).buffer,
                    clientDataJSON: new Uint8Array([33, 34, 35, 36]).buffer,
                },
            };

            mockCredentialsCreate.mockResolvedValue(mockCredential);

            // ACT: Register biometric credential
            const result = await registerBiometricCredential(userId, userName);

            // ASSERT: Verify WebAuthn create API was called correctly
            expect(mockCredentialsCreate).toHaveBeenCalledTimes(1);
            
            const callArgs = mockCredentialsCreate.mock.calls[0][0];
            expect(callArgs).toHaveProperty('publicKey');
            expect(callArgs.publicKey).toHaveProperty('challenge');
            expect(callArgs.publicKey).toHaveProperty('rp');
            expect(callArgs.publicKey).toHaveProperty('user');
            expect(callArgs.publicKey.user.name).toBe(userName);
            expect(callArgs.publicKey.authenticatorSelection.userVerification).toBe('required');

            // Verify returned registration data
            expect(result).toHaveProperty('credentialId');
            expect(result).toHaveProperty('publicKey');
            expect(result).toHaveProperty('attestationObject');
            expect(result).toHaveProperty('clientDataJSON');
            expect(result).toHaveProperty('userId', userId);
        });
    });

    describe('isBiometricAvailable', () => {
        /**
         * TDD Test Case 7: Verify biometric availability detection
         */
        test('should correctly detect biometric availability', async () => {
            // ARRANGE: Mock platform authenticator availability
            mockIsUserVerifyingPlatformAuthenticatorAvailable.mockResolvedValue(true);

            // ACT: Check availability
            const available = await isBiometricAvailable();

            // ASSERT: Verify correct API call and result
            expect(mockIsUserVerifyingPlatformAuthenticatorAvailable).toHaveBeenCalledTimes(1);
            expect(available).toBe(true);
        });

        test('should return false when WebAuthn is not supported', async () => {
            // ARRANGE: Remove WebAuthn support
            delete global.window.PublicKeyCredential;

            // ACT: Check availability
            const available = await isBiometricAvailable();

            // ASSERT: Should return false
            expect(available).toBe(false);

            // Restore for other tests
            global.window.PublicKeyCredential = {
                isUserVerifyingPlatformAuthenticatorAvailable: mockIsUserVerifyingPlatformAuthenticatorAvailable,
            };
        });
    });

    describe('createBiometricApprovedProof', () => {
        /**
         * TDD Test Case 8: Integration test for complete biometric approval flow
         */
        test('should create complete biometric-approved proof', async () => {
            // ARRANGE: Mock biometric availability and WebAuthn response
            mockIsUserVerifyingPlatformAuthenticatorAvailable.mockResolvedValue(true);
            
            const mockAssertion = {
                rawId: new ArrayBuffer(32),
                response: {
                    authenticatorData: new ArrayBuffer(64),
                    signature: new ArrayBuffer(72),
                    userHandle: new ArrayBuffer(16),
                    clientDataJSON: new ArrayBuffer(128),
                },
            };
            
            mockCredentialsGet.mockResolvedValue(mockAssertion);

            const transactionData = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                user_id: "user-123",
                request_id: "req-456",
            };

            // ACT: Create biometric-approved proof
            const proof = await createBiometricApprovedProof(transactionData);

            // ASSERT: Verify complete proof structure
            expect(proof).toHaveProperty('type', 'biometric_webauthn');
            expect(proof).toHaveProperty('biometric_proof');
            expect(proof).toHaveProperty('transaction_context');
            expect(proof).toHaveProperty('device_info');

            // Verify biometric proof contains WebAuthn assertion
            expect(proof.biometric_proof).toHaveProperty('credentialId');
            expect(proof.biometric_proof).toHaveProperty('signature');
            expect(proof.biometric_proof).toHaveProperty('authenticatorData');

            // Verify transaction context includes timestamp
            expect(proof.transaction_context).toHaveProperty('timestamp');
            expect(proof.transaction_context.action).toBe(transactionData.action);
            expect(proof.transaction_context.amount).toBe(transactionData.amount);

            // Verify device info
            expect(proof.device_info).toHaveProperty('user_agent');
            expect(proof.device_info).toHaveProperty('platform');
            expect(proof.device_info).toHaveProperty('timestamp');
        });

        test('should throw error when biometric is not available', async () => {
            // ARRANGE: Mock biometric not available
            mockIsUserVerifyingPlatformAuthenticatorAvailable.mockResolvedValue(false);

            const transactionData = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                user_id: "user-123",
            };

            // ACT & ASSERT: Should throw error
            await expect(createBiometricApprovedProof(transactionData))
                .rejects
                .toThrow('Biometric authentication is not available on this device');
        });
    });

    describe('Utility Functions', () => {
        /**
         * TDD Test Case 9: Verify utility functions work correctly
         */
        test('contextToChallenge should convert object to Uint8Array', () => {
            // ARRANGE: Test context
            const context = { action: "test", amount: 100 };

            // ACT: Convert to challenge
            const challenge = contextToChallenge(context);

            // ASSERT: Verify conversion
            expect(challenge).toBeInstanceOf(Uint8Array);
            
            // Verify round-trip conversion
            const decoded = new TextDecoder().decode(challenge);
            const parsed = JSON.parse(decoded);
            expect(parsed).toEqual(context);
        });

        test('bufferToBase64url should encode buffer correctly', () => {
            // ARRANGE: Test buffer
            const buffer = new Uint8Array([72, 101, 108, 108, 111]).buffer; // "Hello"

            // ACT: Convert to base64url
            const encoded = bufferToBase64url(buffer);

            // ASSERT: Verify encoding (base64url format)
            expect(typeof encoded).toBe('string');
            expect(encoded).not.toContain('+');
            expect(encoded).not.toContain('/');
            expect(encoded).not.toContain('=');
        });
    });

    describe('Security Properties', () => {
        /**
         * TDD Test Case 10: Verify security properties of the implementation
         */
        test('should never expose biometric data to backend', async () => {
            // ARRANGE: Transaction context
            const transactionContext = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "user-123",
            };

            const mockAssertion = {
                rawId: new ArrayBuffer(32),
                response: {
                    authenticatorData: new ArrayBuffer(64),
                    signature: new ArrayBuffer(72),
                    userHandle: new ArrayBuffer(16),
                    clientDataJSON: new ArrayBuffer(128),
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Generate proof
            const proof = await triggerProofGeneration(transactionContext);

            // ASSERT: Verify no biometric data in proof
            const proofString = JSON.stringify(proof);
            
            // These terms should never appear in the proof sent to backend
            const forbiddenTerms = [
                'fingerprint', 'biometric', 'face', 'iris', 'voice',
                'template', 'minutiae', 'enrollment', 'scan'
            ];

            for (const term of forbiddenTerms) {
                expect(proofString.toLowerCase()).not.toContain(term);
            }

            // Proof should only contain cryptographic signatures and metadata
            expect(proof).toHaveProperty('signature'); // Cryptographic signature
            expect(proof).toHaveProperty('credentialId'); // Public credential ID
            expect(proof).toHaveProperty('authenticatorData'); // Authenticator metadata
            expect(proof).not.toHaveProperty('biometricTemplate');
            expect(proof).not.toHaveProperty('biometricData');
        });

        test('should include transaction context in challenge for non-repudiation', async () => {
            // ARRANGE: High-value transaction
            const transactionContext = {
                action: "executive_approval",
                amount: 10000000,
                destination: "Offshore Account XYZ",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "ceo-user-001",
            };

            const mockAssertion = {
                rawId: new ArrayBuffer(32),
                response: {
                    authenticatorData: new ArrayBuffer(64),
                    signature: new ArrayBuffer(72),
                    userHandle: new ArrayBuffer(16),
                    clientDataJSON: new ArrayBuffer(128),
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Generate proof
            await triggerProofGeneration(transactionContext);

            // ASSERT: Verify transaction context is in the challenge
            const callArgs = mockCredentialsGet.mock.calls[0][0];
            const challengeBytes = callArgs.publicKey.challenge;
            const challengeString = new TextDecoder().decode(challengeBytes);
            const challengeObject = JSON.parse(challengeString);

            // The signature will be over this exact transaction context
            // This provides non-repudiation - user cannot claim they signed something else
            expect(challengeObject.action).toBe("executive_approval");
            expect(challengeObject.amount).toBe(10000000);
            expect(challengeObject.destination).toBe("Offshore Account XYZ");
            expect(challengeObject.user_id).toBe("ceo-user-001");
        });
    });
});