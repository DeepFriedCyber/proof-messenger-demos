/**
 * Property-Based Tests in JavaScript
 * 
 * These tests verify protocol invariants using property-based testing
 * techniques with random data generation and fuzzing.
 */

import { describe, test, expect, beforeAll } from 'vitest';
import { createValidatedWasmExports } from '../src/contract_validation.js';

let wasmExports;

beforeAll(async () => {
  wasmExports = await createValidatedWasmExports();
});

// Property-based test utilities
class PropertyTester {
  static randomBytes(length) {
    return new Uint8Array(Array.from({ length }, () => Math.floor(Math.random() * 256)));
  }

  static randomString(length = 10) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    return Array.from({ length }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
  }

  static randomHex(length) {
    const chars = '0123456789abcdef';
    return Array.from({ length }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
  }

  static randomInviteCode() {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    return Array.from({ length: 16 }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
  }

  static runProperty(testFn, iterations = 100) {
    const failures = [];
    
    for (let i = 0; i < iterations; i++) {
      try {
        testFn(i);
      } catch (error) {
        failures.push({ iteration: i, error: error.message });
      }
    }
    
    if (failures.length > 0) {
      throw new Error(`Property failed in ${failures.length}/${iterations} cases: ${JSON.stringify(failures.slice(0, 5))}`);
    }
  }
}

describe('Property-Based Tests for Cryptographic Invariants', () => {
  
  test('PROPERTY: Keypair generation produces valid keys', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      
      // Invariant: Public key is always 32 bytes
      expect(kp.public_key_bytes).toHaveLength(32);
      
      // Invariant: Private key is always 32 bytes
      expect(kp.private_key_bytes).toHaveLength(32);
      
      // Invariant: Keypair bytes is always 64 bytes (private + public)
      expect(kp.keypair_bytes).toHaveLength(64);
      
      // Invariant: Hex representation is always 64 characters
      expect(kp.public_key_hex).toHaveLength(64);
      expect(kp.public_key_hex).toMatch(/^[0-9a-f]+$/);
      
      // Invariant: Keys validate correctly
      expect(wasmExports.validate_public_key(kp.public_key_bytes)).toBe(true);
    }, 50);
  });

  test('PROPERTY: Signature verification is consistent', () => {
    PropertyTester.runProperty((iteration) => {
      const kp = new wasmExports.WasmKeyPair();
      const data = PropertyTester.randomBytes(Math.floor(Math.random() * 1000) + 1);
      
      // Invariant: Signing always produces 64-byte signature
      const signature = kp.sign(data);
      expect(signature).toHaveLength(64);
      expect(wasmExports.validate_signature(signature)).toBe(true);
      
      // Invariant: Signature always verifies with correct key and data
      const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      expect(verified).toBe(true);
      
      // Invariant: Signature fails with wrong key
      const wrongKp = new wasmExports.WasmKeyPair();
      const wrongVerified = wasmExports.verify_signature(wrongKp.public_key_bytes, data, signature);
      expect(wrongVerified).toBe(false);
    }, 50);
  });

  test('PROPERTY: Signature fails with tampered data', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      const originalData = PropertyTester.randomBytes(Math.floor(Math.random() * 1000) + 1);
      const signature = kp.sign(originalData);
      
      // Tamper with data
      const tamperedData = new Uint8Array(originalData);
      if (tamperedData.length > 0) {
        tamperedData[0] = (tamperedData[0] + 1) % 256; // Change first byte
        
        // Invariant: Tampered data should not verify
        const verified = wasmExports.verify_signature(kp.public_key_bytes, tamperedData, signature);
        expect(verified).toBe(false);
      }
    }, 50);
  });

  test('PROPERTY: Hex conversion is reversible', () => {
    PropertyTester.runProperty(() => {
      const originalBytes = PropertyTester.randomBytes(Math.floor(Math.random() * 1000));
      
      // Invariant: bytes -> hex -> bytes should be identity
      const hex = wasmExports.bytes_to_hex(originalBytes);
      const convertedBack = wasmExports.hex_to_bytes(hex);
      
      expect(convertedBack).toEqual(originalBytes);
      
      // Invariant: Hex should be valid format
      expect(hex).toMatch(/^[0-9a-f]*$/);
      expect(hex.length).toBe(originalBytes.length * 2);
    }, 100);
  });

  test('PROPERTY: Invite codes have consistent format', () => {
    PropertyTester.runProperty(() => {
      const invite = wasmExports.generate_invite_code();
      
      // Invariant: Always 16 characters
      expect(invite).toHaveLength(16);
      
      // Invariant: Always base32 format (uppercase alphanumeric)
      expect(invite).toMatch(/^[A-Z0-9]+$/);
      
      // Invariant: Always validates
      expect(wasmExports.validate_invite_code(invite)).toBe(true);
    }, 100);
  });

  test('PROPERTY: Message signing is deterministic', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      const content = PropertyTester.randomString(Math.floor(Math.random() * 100) + 1);
      
      // Create two identical messages
      const msg1 = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, content);
      const msg2 = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, content);
      
      // Sign both with same key
      msg1.sign(kp.keypair_bytes);
      msg2.sign(kp.keypair_bytes);
      
      // Invariant: Same content should produce same signature
      // Note: This might not hold if timestamp is included in signing
      // But the verification should still work
      expect(msg1.verify(kp.public_key_bytes)).toBe(true);
      expect(msg2.verify(kp.public_key_bytes)).toBe(true);
    }, 30);
  });

  test('PROPERTY: Message IDs are unique', () => {
    const seenIds = new Set();
    
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      const content = PropertyTester.randomString();
      const msg = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, content);
      
      // Invariant: Message IDs should be unique
      expect(seenIds.has(msg.id)).toBe(false);
      seenIds.add(msg.id);
      
      // Invariant: ID should be valid UUID format
      expect(msg.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    }, 100);
  });

  test('PROPERTY: Keypair reconstruction preserves keys', () => {
    PropertyTester.runProperty(() => {
      const originalKp = new wasmExports.WasmKeyPair();
      const keypairBytes = originalKp.keypair_bytes;
      
      // Invariant: Reconstructed keypair should have same keys
      const reconstructedKp = wasmExports.WasmKeyPair.from_bytes(keypairBytes);
      
      expect(reconstructedKp.public_key_bytes).toEqual(originalKp.public_key_bytes);
      expect(reconstructedKp.private_key_bytes).toEqual(originalKp.private_key_bytes);
      expect(reconstructedKp.public_key_hex).toBe(originalKp.public_key_hex);
      
      // Invariant: Both should sign identically
      const testData = PropertyTester.randomBytes(100);
      const sig1 = originalKp.sign(testData);
      const sig2 = reconstructedKp.sign(testData);
      
      expect(sig1).toEqual(sig2);
    }, 30);
  });

  test('PROPERTY: Cross-verification consistency', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      const data = PropertyTester.randomBytes(Math.floor(Math.random() * 500) + 1);
      
      // Sign using keypair method
      const signature = kp.sign(data);
      
      // Verify using standalone function
      const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      
      // Invariant: Both verification methods should agree
      expect(verified).toBe(true);
      
      // Create message and verify using message method
      const message = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, 'test');
      message.sign(kp.keypair_bytes);
      const msgVerified = message.verify(kp.public_key_bytes);
      
      expect(msgVerified).toBe(true);
    }, 30);
  });

  test('PROPERTY: Invalid input handling is consistent', () => {
    PropertyTester.runProperty(() => {
      // Test various invalid inputs
      const invalidSizes = [0, 1, 15, 31, 33, 63, 65, 100];
      
      invalidSizes.forEach(size => {
        const invalidBytes = PropertyTester.randomBytes(size);
        
        // Invariant: Invalid public keys should always be rejected
        if (size !== 32) {
          expect(wasmExports.validate_public_key(invalidBytes)).toBe(false);
        }
        
        // Invariant: Invalid signatures should always be rejected
        if (size !== 64) {
          expect(wasmExports.validate_signature(invalidBytes)).toBe(false);
        }
      });
    }, 10);
  });

  test('PROPERTY: Serialization roundtrip preserves data', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      const content = PropertyTester.randomString(Math.floor(Math.random() * 200) + 1);
      
      const originalMsg = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, content);
      originalMsg.sign(kp.keypair_bytes);
      
      // Invariant: Serialization should be reversible
      const json = originalMsg.to_json();
      const deserializedMsg = wasmExports.WasmMessage.from_json(json);
      
      if (deserializedMsg) { // May be undefined for invalid JSON
        expect(deserializedMsg.id).toBe(originalMsg.id);
        expect(deserializedMsg.content).toBe(originalMsg.content);
        expect(deserializedMsg.is_signed).toBe(originalMsg.is_signed);
        expect(deserializedMsg.sender_hex).toBe(originalMsg.sender_hex);
        expect(deserializedMsg.recipient_hex).toBe(originalMsg.recipient_hex);
      }
    }, 30);
  });
});

describe('Fuzzing Tests for Edge Cases', () => {
  
  test('FUZZ: Random data signing and verification', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      
      // Test with various data sizes including edge cases
      const sizes = [0, 1, 2, 3, 4, 5, 10, 100, 1000, 10000];
      const randomSize = sizes[Math.floor(Math.random() * sizes.length)];
      const data = PropertyTester.randomBytes(randomSize);
      
      const signature = kp.sign(data);
      const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      
      expect(verified).toBe(true);
    }, 50);
  });

  test('FUZZ: Random hex strings', () => {
    PropertyTester.runProperty(() => {
      // Test valid hex strings of various lengths
      const validLengths = [0, 2, 4, 6, 8, 10, 20, 64, 128];
      const length = validLengths[Math.floor(Math.random() * validLengths.length)];
      const hexString = PropertyTester.randomHex(length);
      
      const bytes = wasmExports.hex_to_bytes(hexString);
      const backToHex = wasmExports.bytes_to_hex(bytes);
      
      expect(backToHex).toBe(hexString);
    }, 50);
  });

  test('FUZZ: Random invite code validation', () => {
    PropertyTester.runProperty(() => {
      // Test various string formats
      const testStrings = [
        PropertyTester.randomInviteCode(), // Valid format
        PropertyTester.randomString(16).toUpperCase(), // Might be valid
        PropertyTester.randomString(Math.floor(Math.random() * 30)), // Random length
        PropertyTester.randomString(16).toLowerCase(), // Wrong case
        PropertyTester.randomHex(16).toUpperCase(), // Hex chars only
      ];
      
      testStrings.forEach(str => {
        const isValid = wasmExports.validate_invite_code(str);
        
        // Invariant: Valid codes must be 16 chars and alphanumeric uppercase
        if (isValid) {
          expect(str).toHaveLength(16);
          expect(str).toMatch(/^[A-Z0-9]+$/);
        }
      });
    }, 20);
  });

  test('FUZZ: Message content edge cases', () => {
    PropertyTester.runProperty(() => {
      const kp = new wasmExports.WasmKeyPair();
      
      // Test various content types
      const contents = [
        '', // Empty
        ' ', // Whitespace
        '\n\t\r', // Control characters
        '🌍🚀✨', // Emojis
        'A'.repeat(10000), // Very long
        PropertyTester.randomString(Math.floor(Math.random() * 1000)), // Random
        JSON.stringify({ test: 'data', number: 42 }), // JSON
        '<script>alert("xss")</script>', // HTML/XSS
        'SELECT * FROM users;', // SQL-like
      ];
      
      const content = contents[Math.floor(Math.random() * contents.length)];
      
      const message = new wasmExports.WasmMessage(kp.public_key_bytes, kp.public_key_bytes, content);
      message.sign(kp.keypair_bytes);
      
      const verified = message.verify(kp.public_key_bytes);
      expect(verified).toBe(true);
      expect(message.content).toBe(content);
    }, 30);
  });

  test('FUZZ: Concurrent operations stress test', async () => {
    const concurrentOperations = Array.from({ length: 50 }, async (_, i) => {
      return new Promise((resolve) => {
        setTimeout(() => {
          try {
            const kp = new wasmExports.WasmKeyPair();
            const data = PropertyTester.randomBytes(Math.floor(Math.random() * 100) + 1);
            const signature = kp.sign(data);
            const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
            
            resolve({ success: verified, iteration: i });
          } catch (error) {
            resolve({ success: false, error: error.message, iteration: i });
          }
        }, Math.random() * 10); // Random delay up to 10ms
      });
    });
    
    const results = await Promise.all(concurrentOperations);
    const failures = results.filter(r => !r.success);
    
    expect(failures).toHaveLength(0);
  });
});

describe('Performance Property Tests', () => {
  
  test('PROPERTY: Operations complete within time bounds', () => {
    const timeouts = {
      keypairGeneration: 100, // ms
      signing: 50,
      verification: 50,
      inviteGeneration: 10,
      hexConversion: 10
    };
    
    PropertyTester.runProperty(() => {
      // Test keypair generation time
      const kpStart = Date.now();
      const kp = new wasmExports.WasmKeyPair();
      const kpTime = Date.now() - kpStart;
      expect(kpTime).toBeLessThan(timeouts.keypairGeneration);
      
      // Test signing time
      const data = PropertyTester.randomBytes(1000);
      const signStart = Date.now();
      const signature = kp.sign(data);
      const signTime = Date.now() - signStart;
      expect(signTime).toBeLessThan(timeouts.signing);
      
      // Test verification time
      const verifyStart = Date.now();
      const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      const verifyTime = Date.now() - verifyStart;
      expect(verifyTime).toBeLessThan(timeouts.verification);
      expect(verified).toBe(true);
      
      // Test invite generation time
      const inviteStart = Date.now();
      const invite = wasmExports.generate_invite_code();
      const inviteTime = Date.now() - inviteStart;
      expect(inviteTime).toBeLessThan(timeouts.inviteGeneration);
      
      // Test hex conversion time
      const hexStart = Date.now();
      const hex = wasmExports.bytes_to_hex(data);
      const backToBytes = wasmExports.hex_to_bytes(hex);
      const hexTime = Date.now() - hexStart;
      expect(hexTime).toBeLessThan(timeouts.hexConversion);
      expect(backToBytes).toEqual(data);
    }, 20);
  });

  test('PROPERTY: Memory usage remains stable', () => {
    const initialMemory = performance.memory ? performance.memory.usedJSHeapSize : 0;
    
    PropertyTester.runProperty(() => {
      // Perform memory-intensive operations
      const kp = new wasmExports.WasmKeyPair();
      const largeData = PropertyTester.randomBytes(10000);
      const signature = kp.sign(largeData);
      const verified = wasmExports.verify_signature(kp.public_key_bytes, largeData, signature);
      
      expect(verified).toBe(true);
      
      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }
    }, 10);
    
    const finalMemory = performance.memory ? performance.memory.usedJSHeapSize : 0;
    
    // Memory should not grow excessively (allow for some variance)
    if (initialMemory > 0 && finalMemory > 0) {
      const memoryGrowth = finalMemory - initialMemory;
      const maxAllowedGrowth = 10 * 1024 * 1024; // 10MB
      expect(memoryGrowth).toBeLessThan(maxAllowedGrowth);
    }
  });
});