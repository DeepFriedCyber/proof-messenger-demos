/**
 * Biometric Proof Generator - Frontend Implementation
 * 
 * This module handles biometric authentication using the WebAuthn API.
 * The backend server has ZERO knowledge of biometric methods - all biometric
 * interaction happens on the client device to unlock the secure key store.
 * 
 * The server only receives the resulting cryptographic proof.
 */

/**
 * Converts transaction context to WebAuthn challenge format
 * @param {Object} transactionContext - The transaction details
 * @returns {Uint8Array} - Challenge bytes for WebAuthn
 */
function contextToChallenge(transactionContext) {
    const contextJson = JSON.stringify(transactionContext);
    const encoder = new TextEncoder();
    return encoder.encode(contextJson);
}

/**
 * Converts string to base64url format (WebAuthn standard)
 * @param {string} str - Input string
 * @returns {string} - Base64url encoded string
 */
function toBase64url(str) {
    return btoa(str)
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=/g, '');
}

/**
 * Converts Uint8Array to base64url format
 * @param {Uint8Array} buffer - Input buffer
 * @returns {string} - Base64url encoded string
 */
function bufferToBase64url(buffer) {
    const str = String.fromCharCode(...new Uint8Array(buffer));
    return toBase64url(str);
}

/**
 * Triggers biometric authentication for proof generation
 * This function calls the browser's WebAuthn API with the transaction context as challenge
 * 
 * @param {Object} transactionContext - Transaction details to be signed
 * @param {string} transactionContext.action - Type of action (e.g., "wire_transfer")
 * @param {number} transactionContext.amount - Transaction amount
 * @param {string} transactionContext.destination - Destination account/entity
 * @param {string} transactionContext.timestamp - ISO timestamp
 * @param {string} transactionContext.user_id - User identifier
 * @returns {Promise<Object>} - WebAuthn assertion result
 */
async function triggerProofGeneration(transactionContext) {
    // Validate input
    if (!transactionContext || typeof transactionContext !== 'object') {
        throw new Error('Transaction context is required');
    }

    // Required fields validation
    const requiredFields = ['action', 'amount', 'destination', 'timestamp', 'user_id'];
    for (const field of requiredFields) {
        if (!transactionContext[field]) {
            throw new Error(`Missing required field: ${field}`);
        }
    }

    // Convert transaction context to WebAuthn challenge
    const challengeBytes = contextToChallenge(transactionContext);
    
    // WebAuthn API call - this triggers biometric authentication on the device
    const webAuthnOptions = {
        publicKey: {
            challenge: challengeBytes,
            allowCredentials: [], // Allow any registered credential
            userVerification: 'required', // Require biometric/PIN verification
            timeout: 60000, // 60 second timeout
            rpId: window.location.hostname, // Relying party ID
        }
    };

    try {
        // This is where the biometric magic happens:
        // 1. Browser prompts user for biometric authentication (fingerprint, face, etc.)
        // 2. Device's secure enclave/TPM unlocks the private key
        // 3. Private key signs the challenge (our transaction context)
        // 4. Returns cryptographic proof - NO biometric data leaves the device
        const assertion = await navigator.credentials.get(webAuthnOptions);
        
        if (!assertion) {
            throw new Error('Biometric authentication was cancelled or failed');
        }

        // Extract the cryptographic proof components
        const proof = {
            credentialId: bufferToBase64url(assertion.rawId),
            authenticatorData: bufferToBase64url(assertion.response.authenticatorData),
            signature: bufferToBase64url(assertion.response.signature),
            userHandle: assertion.response.userHandle ? bufferToBase64url(assertion.response.userHandle) : null,
            clientDataJSON: bufferToBase64url(assertion.response.clientDataJSON),
            transactionContext: transactionContext, // Include original context for server verification
        };

        return proof;
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

/**
 * Registers a new biometric credential for the user
 * This should be called during user onboarding or when adding a new device
 * 
 * @param {string} userId - User identifier
 * @param {string} userName - User display name
 * @returns {Promise<Object>} - Registration result
 */
async function registerBiometricCredential(userId, userName) {
    // Generate a random challenge for registration
    const challenge = new Uint8Array(32);
    crypto.getRandomValues(challenge);

    const registrationOptions = {
        publicKey: {
            challenge: challenge,
            rp: {
                name: "Proof Messenger",
                id: window.location.hostname,
            },
            user: {
                id: new TextEncoder().encode(userId),
                name: userName,
                displayName: userName,
            },
            pubKeyCredParams: [
                { alg: -7, type: "public-key" }, // ES256
                { alg: -257, type: "public-key" }, // RS256
            ],
            authenticatorSelection: {
                authenticatorAttachment: "platform", // Prefer built-in authenticators
                userVerification: "required", // Require biometric verification
                requireResidentKey: false,
            },
            timeout: 60000,
            attestation: "direct",
        }
    };

    try {
        const credential = await navigator.credentials.create(registrationOptions);
        
        if (!credential) {
            throw new Error('Biometric credential registration was cancelled');
        }

        // Return registration data for server storage
        return {
            credentialId: bufferToBase64url(credential.rawId),
            publicKey: bufferToBase64url(credential.response.publicKey),
            attestationObject: bufferToBase64url(credential.response.attestationObject),
            clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
            userId: userId,
        };
    } catch (error) {
        throw new Error(`Biometric credential registration failed: ${error.message}`);
    }
}

/**
 * Checks if biometric authentication is available on this device
 * @returns {Promise<boolean>} - True if biometric auth is available
 */
async function isBiometricAvailable() {
    if (!window.PublicKeyCredential) {
        return false;
    }

    try {
        const available = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
        return available;
    } catch (error) {
        console.warn('Error checking biometric availability:', error);
        return false;
    }
}

/**
 * High-level function to create a proof with biometric approval
 * This combines the transaction context preparation with biometric authentication
 * 
 * @param {Object} transactionData - Raw transaction data
 * @returns {Promise<Object>} - Complete proof ready for server submission
 */
async function createBiometricApprovedProof(transactionData) {
    // Check if biometric authentication is available
    const biometricAvailable = await isBiometricAvailable();
    if (!biometricAvailable) {
        throw new Error('Biometric authentication is not available on this device');
    }

    // Prepare transaction context with timestamp and user info
    const transactionContext = {
        action: transactionData.action,
        amount: transactionData.amount,
        destination: transactionData.destination,
        timestamp: new Date().toISOString(),
        user_id: transactionData.user_id,
        request_id: transactionData.request_id || `req-${Date.now()}`,
    };

    // Trigger biometric authentication
    const biometricProof = await triggerProofGeneration(transactionContext);

    // Combine with any additional proof data
    const completeProof = {
        type: 'biometric_webauthn',
        biometric_proof: biometricProof,
        transaction_context: transactionContext,
        device_info: {
            user_agent: navigator.userAgent,
            platform: navigator.platform,
            timestamp: new Date().toISOString(),
        },
    };

    return completeProof;
}

// Export functions for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    // Node.js environment (for testing)
    module.exports = {
        triggerProofGeneration,
        registerBiometricCredential,
        isBiometricAvailable,
        createBiometricApprovedProof,
        contextToChallenge,
        bufferToBase64url,
    };
} else {
    // Browser environment
    window.BiometricProofGenerator = {
        triggerProofGeneration,
        registerBiometricCredential,
        isBiometricAvailable,
        createBiometricApprovedProof,
    };
}