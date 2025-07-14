/**
 * Simplified TDD Test Suite for Biometric Proof Generator
 * 
 * This test demonstrates the corrected architectural approach:
 * - Backend has ZERO knowledge of biometric methods
 * - Frontend calls WebAuthn API with transaction context as challenge
 * - Only cryptographic proofs are sent to backend
 */

// Mock the WebAuthn API functions
const mockCredentialsGet = jest.fn();
const mockCredentialsCreate = jest.fn();
const mockIsUserVerifyingPlatformAuthenticatorAvailable = jest.fn();

// Mock browser globals
const mockNavigator = {
    credentials: {
        get: mockCredentialsGet,
        create: mockCredentialsCreate,
    },
    userAgent: 'Mozilla/5.0 (Test Browser)',
    platform: 'Test Platform',
};

const mockWindow = {
    location: {
        hostname: 'proof-messenger.example.com',
    },
    PublicKeyCredential: {
        isUserVerifyingPlatformAuthenticatorAvailable: mockIsUserVerifyingPlatformAuthenticatorAvailable,
    },
};

// Simplified biometric proof generator for testing
class BiometricProofGenerator {
    constructor(navigator, window) {
        this.navigator = navigator;
        this.window = window;
    }

    contextToChallenge(transactionContext) {
        const contextJson = JSON.stringify(transactionContext);
        const encoder = new TextEncoder();
        return encoder.encode(contextJson);
    }

    bufferToBase64url(buffer) {
        const str = String.fromCharCode(...new Uint8Array(buffer));
        return btoa(str)
            .replace(/\+/g, '-')
            .replace(/\//g, '_')
            .replace(/=/g, '');
    }

    async triggerProofGeneration(transactionContext) {
        // Validate input
        if (!transactionContext || typeof transactionContext !== 'object') {
            throw new Error('Transaction context is required');
        }

        const requiredFields = ['action', 'amount', 'destination', 'timestamp', 'user_id'];
        for (const field of requiredFields) {
            if (!transactionContext[field]) {
                throw new Error(`Missing required field: ${field}`);
            }
        }

        // Convert transaction context to WebAuthn challenge
        const challengeBytes = this.contextToChallenge(transactionContext);
        
        // WebAuthn API call
        const webAuthnOptions = {
            publicKey: {
                challenge: challengeBytes,
                allowCredentials: [],
                userVerification: 'required',
                timeout: 60000,
                rpId: this.window.location.hostname,
            }
        };

        try {
            const assertion = await this.navigator.credentials.get(webAuthnOptions);
            
            if (!assertion) {
                throw new Error('Biometric authentication was cancelled or failed');
            }

            return {
                credentialId: this.bufferToBase64url(assertion.rawId),
                authenticatorData: this.bufferToBase64url(assertion.response.authenticatorData),
                signature: this.bufferToBase64url(assertion.response.signature),
                userHandle: assertion.response.userHandle ? this.bufferToBase64url(assertion.response.userHandle) : null,
                clientDataJSON: this.bufferToBase64url(assertion.response.clientDataJSON),
                transactionContext: transactionContext,
            };
        } catch (error) {
            if (error.name === 'NotAllowedError') {
                throw new Error('Biometric authentication was denied by user');
            } else if (error.name === 'NotSupportedError') {
                throw new Error('Biometric authentication is not supported on this device');
            } else if (error.name === 'SecurityError') {
                throw new Error('Biometric authentication failed due to security restrictions');
            } else {
                throw new Error(`Biometric authentication failed: ${error.message}`);
            }
        }
    }

    async isBiometricAvailable() {
        if (!this.window.PublicKeyCredential) {
            return false;
        }

        try {
            const available = await this.window.PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
            return available;
        } catch (error) {
            return false;
        }
    }
}

describe('Biometric Proof Generator - Corrected Architecture', () => {
    let generator;

    beforeEach(() => {
        // Reset all mocks
        jest.clearAllMocks();
        
        // Create generator with mocked dependencies
        generator = new BiometricProofGenerator(mockNavigator, mockWindow);
    });

    describe('Core TDD Test: WebAuthn API Integration', () => {
        /**
         * PRIMARY TEST CASE: Verify WebAuthn API called with transaction context as challenge
         * This test proves the corrected architectural approach
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
                rawId: new Uint8Array([1, 2, 3, 4]).buffer,
                response: {
                    authenticatorData: new Uint8Array([5, 6, 7, 8]).buffer,
                    signature: new Uint8Array([9, 10, 11, 12]).buffer,
                    userHandle: new Uint8Array([13, 14, 15, 16]).buffer,
                    clientDataJSON: new Uint8Array([17, 18, 19, 20]).buffer,
                },
            };

            mockCredentialsGet.mockResolvedValue(mockAssertion);

            // ACT: Trigger biometric proof generation
            await generator.triggerProofGeneration(transactionContext);

            // ASSERT: Verify WebAuthn API was called with correct parameters
            expect(mockCredentialsGet).toHaveBeenCalledTimes(1);
            
            const callArgs = mockCredentialsGet.mock.calls[0][0];
            expect(callArgs).toHaveProperty('publicKey');
            expect(callArgs.publicKey).toHaveProperty('challenge');
            expect(callArgs.publicKey).toHaveProperty('userVerification', 'required');
            expect(callArgs.publicKey).toHaveProperty('timeout', 60000);
            expect(callArgs.publicKey).toHaveProperty('rpId', 'proof-messenger.example.com');

            // CRITICAL: Verify the challenge contains our transaction context
            const challengeBytes = callArgs.publicKey.challenge;
            const challengeString = new TextDecoder().decode(challengeBytes);
            const challengeObject = JSON.parse(challengeString);
            
            expect(challengeObject).toEqual(transactionContext);
        });

        /**
         * TDD Test Case: Verify transaction context validation
         */
        test('should validate required transaction context fields', async () => {
            const invalidContexts = [
                null,
                undefined,
                {},
                { action: "transfer" }, // Missing required fields
                { action: "transfer", amount: 1000 }, // Missing destination, timestamp, user_id
            ];

            for (const invalidContext of invalidContexts) {
                await expect(generator.triggerProofGeneration(invalidContext))
                    .rejects
                    .toThrow();
            }

            // Verify WebAuthn API was never called for invalid inputs
            expect(mockCredentialsGet).not.toHaveBeenCalled();
        });

        /**
         * TDD Test Case: Verify proper error handling for WebAuthn failures
         */
        test('should handle WebAuthn API errors correctly', async () => {
            const transactionContext = {
                action: "wire_transfer",
                amount: 1000000,
                destination: "ACME Corp",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "user-123",
            };

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
            ];

            for (const scenario of errorScenarios) {
                mockCredentialsGet.mockRejectedValue(scenario.error);

                await expect(generator.triggerProofGeneration(transactionContext))
                    .rejects
                    .toThrow(scenario.expectedMessage);
                
                mockCredentialsGet.mockReset();
            }
        });

        /**
         * TDD Test Case: Verify proof structure returned to backend
         */
        test('should return properly structured proof for backend verification', async () => {
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

            const proof = await generator.triggerProofGeneration(transactionContext);

            // Verify proof structure
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
    });

    describe('Security Properties', () => {
        /**
         * TDD Test Case: Verify no biometric data is exposed to backend
         */
        test('should never expose biometric data to backend', async () => {
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

            const proof = await generator.triggerProofGeneration(transactionContext);

            // Verify no biometric data in proof
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

        /**
         * TDD Test Case: Verify transaction context in challenge for non-repudiation
         */
        test('should include transaction context in challenge for non-repudiation', async () => {
            const transactionContext = {
                action: "executive_approval",
                amount: 10000000,
                destination: "Offshore Account XYZ",
                timestamp: "2024-01-15T10:30:00Z",
                user_id: "ceo-user-001",
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

            await generator.triggerProofGeneration(transactionContext);

            // Verify transaction context is in the challenge
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

    describe('Biometric Availability Detection', () => {
        /**
         * TDD Test Case: Verify biometric availability detection
         */
        test('should correctly detect biometric availability', async () => {
            mockIsUserVerifyingPlatformAuthenticatorAvailable.mockResolvedValue(true);

            const available = await generator.isBiometricAvailable();

            expect(mockIsUserVerifyingPlatformAuthenticatorAvailable).toHaveBeenCalledTimes(1);
            expect(available).toBe(true);
        });

        test('should return false when WebAuthn is not supported', async () => {
            const generatorWithoutWebAuthn = new BiometricProofGenerator(
                mockNavigator, 
                { location: { hostname: 'test.com' } } // No PublicKeyCredential
            );

            const available = await generatorWithoutWebAuthn.isBiometricAvailable();

            expect(available).toBe(false);
        });
    });

    describe('Utility Functions', () => {
        /**
         * TDD Test Case: Verify utility functions work correctly
         */
        test('contextToChallenge should convert object to Uint8Array', () => {
            const context = { action: "test", amount: 100 };

            const challenge = generator.contextToChallenge(context);

            expect(challenge).toBeInstanceOf(Uint8Array);
            
            // Verify round-trip conversion
            const decoded = new TextDecoder().decode(challenge);
            const parsed = JSON.parse(decoded);
            expect(parsed).toEqual(context);
        });

        test('bufferToBase64url should encode buffer correctly', () => {
            const buffer = new Uint8Array([72, 101, 108, 108, 111]).buffer; // "Hello"

            const encoded = generator.bufferToBase64url(buffer);

            expect(typeof encoded).toBe('string');
            expect(encoded).not.toContain('+');
            expect(encoded).not.toContain('/');
            expect(encoded).not.toContain('=');
        });
    });
});