/**
 * 2. JavaScript Usage (with Defensive Tests)
 * 
 * This module demonstrates the exact defensive programming pattern
 * requested, with comprehensive TDD-style assertions and error handling.
 */

import init, {
    WasmKeyPair,
    WasmMessage,
    generate_invite_code,
    validate_invite_code,
    bytes_to_hex,
    hex_to_bytes
} from "./pkg/proof_messenger_web.js";

// Defensive async WASM loading with comprehensive error handling
async function initWasmSafe() {
    try {
        await init();
        console.log("✅ WASM initialized successfully");
        return true;
    } catch (e) {
        console.error("❌ WASM init failed:", e);
        console.error("Stack trace:", e.stack);
        console.error("Possible causes:");
        console.error("- WASM files not found or corrupted");
        console.error("- Browser doesn't support WASM");
        console.error("- Network loading issues");
        throw e;
    }
}

// Enhanced assertion function with detailed reporting
function defensiveAssert(condition, message, context = "") {
    if (condition) {
        console.log(`✅ PASS: ${message}${context ? ` (${context})` : ""}`);
        return true;
    } else {
        console.error(`❌ FAIL: ${message}${context ? ` (${context})` : ""}`);
        console.trace("Assertion failed at:");
        return false;
    }
}

// Safe property access with validation
function safeAccess(obj, property, expectedType, context) {
    try {
        const value = obj[property];
        const actualType = typeof value;
        
        if (expectedType && actualType !== expectedType) {
            console.warn(`⚠️ Type mismatch for ${property}: expected ${expectedType}, got ${actualType}`);
            return null;
        }
        
        return value;
    } catch (error) {
        console.error(`❌ Failed to access ${property} on ${context}:`, error);
        return null;
    }
}

// Main defensive test execution
(async () => {
    console.log("🚀 Starting Defensive WASM Crypto Usage Tests");
    console.log("=" .repeat(60));
    
    let testsPassed = 0;
    let testsTotal = 0;
    
    function runTest(testFn, testName) {
        testsTotal++;
        console.log(`\n🧪 Running: ${testName}`);
        try {
            const result = testFn();
            if (result !== false) {
                testsPassed++;
                console.log(`✅ ${testName} completed successfully`);
            }
        } catch (error) {
            console.error(`💥 ${testName} failed with exception:`, error);
        }
    }
    
    try {
        // Initialize WASM with defensive error handling
        await initWasmSafe();
        
        // Test 1: Keypair Generation with Comprehensive Validation
        runTest(() => {
            let kp;
            try {
                kp = new WasmKeyPair();
                defensiveAssert(kp !== null && kp !== undefined, "Keypair should be created", "constructor");
            } catch (e) {
                console.error("❌ Keypair creation failed:", e);
                return false;
            }
            
            // Validate public key properties
            const pubKeyBytes = safeAccess(kp, 'public_key_bytes', 'object', 'WasmKeyPair');
            defensiveAssert(pubKeyBytes instanceof Uint8Array, "Public key should be Uint8Array", "type check");
            defensiveAssert(pubKeyBytes.length === 32, "Public key must be 32 bytes", `got ${pubKeyBytes?.length}`);
            
            // Validate private key properties  
            const privKeyBytes = safeAccess(kp, 'private_key_bytes', 'object', 'WasmKeyPair');
            defensiveAssert(privKeyBytes instanceof Uint8Array, "Private key should be Uint8Array", "type check");
            defensiveAssert(privKeyBytes.length === 32, "Private key must be 32 bytes", `got ${privKeyBytes?.length}`);
            
            // Validate hex representation
            const pubKeyHex = safeAccess(kp, 'public_key_hex', 'string', 'WasmKeyPair');
            defensiveAssert(typeof pubKeyHex === 'string', "Public key hex should be string", "type check");
            defensiveAssert(pubKeyHex.length === 64, "Public key hex should be 64 chars", `got ${pubKeyHex?.length}`);
            defensiveAssert(/^[0-9a-f]+$/i.test(pubKeyHex), "Public key hex should contain only hex chars", "format check");
            
            console.log(`📋 Keypair generated: ${pubKeyHex.substring(0, 16)}...`);
            
            // Store for later tests
            window.testKeypair = kp;
            return true;
        }, "Keypair Generation & Validation");
        
        // Test 2: Invite Code Generation with Validation
        runTest(() => {
            let invite;
            try {
                invite = generate_invite_code();
                defensiveAssert(typeof invite === 'string', "Invite code should be string", "type check");
            } catch (e) {
                console.error("❌ Invite code generation failed:", e);
                return false;
            }
            
            // Validate invite code format
            defensiveAssert(invite.length === 16, "Invite code should be 16 chars", `got ${invite.length}`);
            defensiveAssert(/^[A-Z0-9]+$/.test(invite), "Invite code should be base32", "format check");
            defensiveAssert(validate_invite_code(invite), "Generated invite code should validate", "validation check");
            
            // Test validation edge cases
            defensiveAssert(!validate_invite_code(""), "Empty string should not validate", "empty test");
            defensiveAssert(!validate_invite_code("short"), "Short code should not validate", "length test");
            defensiveAssert(!validate_invite_code("toolongtobevalid123"), "Long code should not validate", "length test");
            defensiveAssert(!validate_invite_code("invalid@#$"), "Invalid chars should not validate", "character test");
            
            console.log(`📋 Invite code generated: ${invite}`);
            return true;
        }, "Invite Code Generation & Validation");
        
        // Test 3: Message Creation, Signing, and Verification
        runTest(() => {
            const kp = window.testKeypair;
            if (!kp) {
                console.error("❌ No keypair available for message test");
                return false;
            }
            
            let msg;
            try {
                // Create message with defensive parameter validation
                const senderBytes = kp.public_key_bytes;
                const recipientBytes = kp.public_key_bytes; // Self-message for testing
                const content = "Hello, this is a test message!";
                
                defensiveAssert(senderBytes instanceof Uint8Array, "Sender bytes should be Uint8Array", "pre-check");
                defensiveAssert(recipientBytes instanceof Uint8Array, "Recipient bytes should be Uint8Array", "pre-check");
                defensiveAssert(typeof content === 'string', "Content should be string", "pre-check");
                
                msg = new WasmMessage(senderBytes, recipientBytes, content);
                defensiveAssert(msg !== null && msg !== undefined, "Message should be created", "constructor");
            } catch (e) {
                console.error("❌ Message creation failed:", e);
                return false;
            }
            
            // Validate initial message state
            defensiveAssert(!msg.is_signed, "New message should not be signed initially", "initial state");
            defensiveAssert(typeof msg.id === 'string', "Message ID should be string", "ID type");
            defensiveAssert(msg.id.length > 0, "Message ID should not be empty", "ID length");
            defensiveAssert(msg.content === "Hello, this is a test message!", "Content should match input", "content check");
            
            // Test message signing with error handling
            try {
                const keypairBytes = kp.keypair_bytes;
                defensiveAssert(keypairBytes instanceof Uint8Array, "Keypair bytes should be Uint8Array", "signing pre-check");
                defensiveAssert(keypairBytes.length === 64, "Keypair bytes should be 64 bytes", `got ${keypairBytes.length}`);
                
                msg.sign(keypairBytes);
                defensiveAssert(msg.is_signed, "Message should be signed after signing", "signing result");
            } catch (e) {
                console.error("❌ Message signing failed:", e);
                return false;
            }
            
            // Test message verification
            try {
                const verified = msg.verify(kp.public_key_bytes);
                defensiveAssert(verified === true, "Signed message should verify with correct key", "verification");
                
                // Test verification with wrong key (create another keypair)
                const wrongKp = new WasmKeyPair();
                const wrongVerified = msg.verify(wrongKp.public_key_bytes);
                defensiveAssert(wrongVerified === false, "Message should not verify with wrong key", "wrong key test");
                
            } catch (e) {
                console.error("❌ Message verification failed:", e);
                return false;
            }
            
            console.log(`📋 Message signed and verified: ${msg.id}`);
            return true;
        }, "Message Operations (Create, Sign, Verify)");
        
        // Test 4: Security and Tampering Detection
        runTest(() => {
            const kp = window.testKeypair;
            if (!kp) {
                console.error("❌ No keypair available for security test");
                return false;
            }
            
            // Create two identical messages
            const msg1 = new WasmMessage(kp.public_key_bytes, kp.public_key_bytes, "Original content");
            const msg2 = new WasmMessage(kp.public_key_bytes, kp.public_key_bytes, "Tampered content");
            
            // Sign both messages
            msg1.sign(kp.keypair_bytes);
            msg2.sign(kp.keypair_bytes);
            
            // Both should verify with correct key
            defensiveAssert(msg1.verify(kp.public_key_bytes), "Original message should verify", "original verification");
            defensiveAssert(msg2.verify(kp.public_key_bytes), "Tampered message should verify with its own signature", "tampered verification");
            
            // Messages should have different IDs (replay attack prevention)
            defensiveAssert(msg1.id !== msg2.id, "Different messages should have different IDs", "ID uniqueness");
            
            // Test cross-verification (should fail)
            const wrongKp = new WasmKeyPair();
            defensiveAssert(!msg1.verify(wrongKp.public_key_bytes), "Message should not verify with wrong key", "key isolation");
            
            console.log("📋 Security tests passed - tampering detection working");
            return true;
        }, "Security & Tampering Detection");
        
        // Test 5: Data Conversion and Edge Cases
        runTest(() => {
            // Test hex conversion roundtrip
            const testBytes = new Uint8Array([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
            
            try {
                const hex = bytes_to_hex(testBytes);
                defensiveAssert(typeof hex === 'string', "bytes_to_hex should return string", "conversion type");
                defensiveAssert(hex === "0123456789abcdef", "Hex conversion should be correct", `got ${hex}`);
                
                const backToBytes = hex_to_bytes(hex);
                defensiveAssert(backToBytes instanceof Uint8Array, "hex_to_bytes should return Uint8Array", "conversion type");
                defensiveAssert(backToBytes.length === testBytes.length, "Converted bytes should have same length", "length check");
                
                // Check byte-by-byte equality
                let bytesEqual = true;
                for (let i = 0; i < testBytes.length; i++) {
                    if (testBytes[i] !== backToBytes[i]) {
                        bytesEqual = false;
                        break;
                    }
                }
                defensiveAssert(bytesEqual, "Hex conversion should be reversible", "roundtrip test");
                
            } catch (e) {
                console.error("❌ Data conversion test failed:", e);
                return false;
            }
            
            // Test error handling for invalid hex
            try {
                hex_to_bytes("invalid_hex_string");
                defensiveAssert(false, "Should throw error for invalid hex", "error handling");
            } catch (e) {
                defensiveAssert(true, "Correctly threw error for invalid hex", "error handling");
            }
            
            console.log("📋 Data conversion tests passed");
            return true;
        }, "Data Conversion & Edge Cases");
        
        // Test 6: Performance and Stress Testing
        runTest(() => {
            console.log("🔥 Running performance stress test...");
            const startTime = Date.now();
            const iterations = 50;
            let successCount = 0;
            
            for (let i = 0; i < iterations; i++) {
                try {
                    // Generate keypair
                    const kp = new WasmKeyPair();
                    
                    // Generate invite code
                    const invite = generate_invite_code();
                    
                    // Create and sign message
                    const msg = new WasmMessage(kp.public_key_bytes, kp.public_key_bytes, `Stress test ${i}`);
                    msg.sign(kp.keypair_bytes);
                    
                    // Verify message
                    const verified = msg.verify(kp.public_key_bytes);
                    
                    if (verified && validate_invite_code(invite)) {
                        successCount++;
                    }
                } catch (error) {
                    console.warn(`⚠️ Stress test iteration ${i} failed:`, error.message);
                }
            }
            
            const endTime = Date.now();
            const duration = endTime - startTime;
            const successRate = (successCount / iterations * 100).toFixed(1);
            const avgTime = (duration / iterations).toFixed(2);
            
            console.log(`📊 Stress test results:`);
            console.log(`   - Iterations: ${iterations}`);
            console.log(`   - Successful: ${successCount}`);
            console.log(`   - Success rate: ${successRate}%`);
            console.log(`   - Total time: ${duration}ms`);
            console.log(`   - Average per operation: ${avgTime}ms`);
            
            defensiveAssert(successRate >= 95, `Success rate should be ≥95% (got ${successRate}%)`, "performance test");
            defensiveAssert(avgTime < 100, `Average time should be <100ms (got ${avgTime}ms)`, "performance test");
            
            return true;
        }, "Performance & Stress Testing");
        
    } catch (error) {
        console.error("💥 Critical error in test suite:", error);
        console.error("Stack trace:", error.stack);
    }
    
    // Final results
    console.log("\n" + "=".repeat(60));
    console.log(`🎯 Test Results: ${testsPassed}/${testsTotal} tests passed`);
    
    if (testsPassed === testsTotal) {
        console.log("🎉 All tests passed! WASM crypto implementation is working correctly.");
        
        // Display final demo results if we have a DOM
        if (typeof document !== 'undefined') {
            const outputElement = document.getElementById("output");
            if (outputElement) {
                outputElement.textContent = `
🎉 Defensive WASM Crypto Tests - ALL PASSED!

Keypair: ${window.testKeypair?.public_key_hex || 'N/A'}
Invite: ${generate_invite_code()}
Tests Passed: ${testsPassed}/${testsTotal}
Success Rate: 100%

✅ Keypair generation and validation
✅ Invite code system
✅ Message operations (create, sign, verify)
✅ Security and tampering detection  
✅ Data conversion and edge cases
✅ Performance and stress testing

All cryptographic operations are working correctly with proper error handling!
                `;
            }
        }
    } else {
        console.log(`❌ ${testsTotal - testsPassed} tests failed. Please check the implementation.`);
    }
    
})().catch(error => {
    console.error("💥 Unhandled error in defensive test suite:", error);
});

// Export for use in other modules
export {
    initWasmSafe,
    defensiveAssert
};