/**
 * Comprehensive Test Suite for All WASM Exports
 * 
 * This test suite validates every WASM export with:
 * - Contract validation
 * - Property-based testing
 * - Error handling verification
 * - Performance benchmarking
 */

import { describe, test, expect, beforeAll, afterEach } from 'vitest';
import { createValidatedWasmExports, ContractViolationError } from '../src/contract_validation.js';

let wasmExports;
let validator;

beforeAll(async () => {
  console.log('Loading WASM exports for testing...');
  wasmExports = await createValidatedWasmExports();
  validator = wasmExports.validator;
  console.log('WASM exports loaded successfully');
});

afterEach(() => {
  // Reset validator state between tests
  validator.reset();
});

describe('WASM Export Contract Validation', () => {
  
  describe('generate_invite_code', () => {
    test('should generate valid invite codes', () => {
      const invite = wasmExports.generate_invite_code();
      
      expect(invite).toBeTypeOf('string');
      expect(invite).toHaveLength(16);
      expect(invite).toMatch(/^[A-Z0-9]+$/);
      expect(wasmExports.validate_invite_code(invite)).toBe(true);
    });

    test('should generate unique invite codes', () => {
      const invites = Array.from({ length: 100 }, () => wasmExports.generate_invite_code());
      const uniqueInvites = new Set(invites);
      
      expect(uniqueInvites.size).toBe(invites.length);
    });

    test('should handle generation errors gracefully', () => {
      // This test verifies error handling exists, even if errors are rare
      expect(() => wasmExports.generate_invite_code()).not.toThrow();
    });
  });

  describe('validate_invite_code', () => {
    test('should validate correct invite codes', () => {
      const validCodes = [
        'ABCDEFGHIJKLMNOP',
        '1234567890123456',
        'A1B2C3D4E5F6G7H8'
      ];

      validCodes.forEach(code => {
        expect(wasmExports.validate_invite_code(code)).toBe(true);
      });
    });

    test('should reject invalid invite codes', () => {
      const invalidCodes = [
        '',                    // Empty
        'short',              // Too short
        'toolongtobevalid123', // Too long
        'invalid@#$%^&*()',   // Invalid characters
        'abcdefghijklmnop',   // Lowercase (should be uppercase)
        null,                 // Null
        undefined,            // Undefined
        123                   // Wrong type
      ];

      invalidCodes.forEach(code => {
        if (typeof code === 'string') {
          expect(wasmExports.validate_invite_code(code)).toBe(false);
        } else {
          expect(() => wasmExports.validate_invite_code(code)).toThrow(ContractViolationError);
        }
      });
    });

    test('should enforce parameter contracts', () => {
      expect(() => wasmExports.validate_invite_code()).toThrow(ContractViolationError);
      expect(() => wasmExports.validate_invite_code(123)).toThrow(ContractViolationError);
      expect(() => wasmExports.validate_invite_code(null)).toThrow(ContractViolationError);
    });
  });

  describe('bytes_to_hex', () => {
    test('should convert bytes to hex correctly', () => {
      const testCases = [
        { bytes: new Uint8Array([]), expected: '' },
        { bytes: new Uint8Array([0x00]), expected: '00' },
        { bytes: new Uint8Array([0xff]), expected: 'ff' },
        { bytes: new Uint8Array([0x01, 0x23, 0x45, 0x67]), expected: '01234567' },
        { bytes: new Uint8Array([0x89, 0xab, 0xcd, 0xef]), expected: '89abcdef' }
      ];

      testCases.forEach(({ bytes, expected }) => {
        const result = wasmExports.bytes_to_hex(bytes);
        expect(result).toBe(expected);
        expect(result).toMatch(/^[0-9a-f]*$/);
      });
    });

    test('should handle large byte arrays', () => {
      const largeBytes = new Uint8Array(10000).fill(0xaa);
      const result = wasmExports.bytes_to_hex(largeBytes);
      
      expect(result).toHaveLength(20000);
      expect(result).toBe('aa'.repeat(10000));
    });

    test('should enforce parameter contracts', () => {
      expect(() => wasmExports.bytes_to_hex()).toThrow(ContractViolationError);
      expect(() => wasmExports.bytes_to_hex('not bytes')).toThrow(ContractViolationError);
      expect(() => wasmExports.bytes_to_hex(null)).toThrow(ContractViolationError);
    });
  });

  describe('hex_to_bytes', () => {
    test('should convert hex to bytes correctly', () => {
      const testCases = [
        { hex: '', expected: new Uint8Array([]) },
        { hex: '00', expected: new Uint8Array([0x00]) },
        { hex: 'ff', expected: new Uint8Array([0xff]) },
        { hex: '01234567', expected: new Uint8Array([0x01, 0x23, 0x45, 0x67]) },
        { hex: '89abcdef', expected: new Uint8Array([0x89, 0xab, 0xcd, 0xef]) },
        { hex: '89ABCDEF', expected: new Uint8Array([0x89, 0xab, 0xcd, 0xef]) } // Case insensitive
      ];

      testCases.forEach(({ hex, expected }) => {
        const result = wasmExports.hex_to_bytes(hex);
        expect(result).toEqual(expected);
      });
    });

    test('should handle roundtrip conversion', () => {
      const originalBytes = new Uint8Array([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
      const hex = wasmExports.bytes_to_hex(originalBytes);
      const convertedBack = wasmExports.hex_to_bytes(hex);
      
      expect(convertedBack).toEqual(originalBytes);
    });

    test('should reject invalid hex strings', () => {
      const invalidHex = [
        'invalid',     // Invalid characters
        'xyz',         // Invalid characters
        '12g',         // Invalid character
        '1',           // Odd length
        '123'          // Odd length
      ];

      invalidHex.forEach(hex => {
        expect(() => wasmExports.hex_to_bytes(hex)).toThrow();
      });
    });

    test('should enforce parameter contracts', () => {
      expect(() => wasmExports.hex_to_bytes()).toThrow(ContractViolationError);
      expect(() => wasmExports.hex_to_bytes(123)).toThrow(ContractViolationError);
      expect(() => wasmExports.hex_to_bytes(null)).toThrow(ContractViolationError);
    });
  });

  describe('validate_public_key', () => {
    test('should validate correct public keys', () => {
      const kp = new wasmExports.WasmKeyPair();
      const publicKey = kp.public_key_bytes;
      
      expect(wasmExports.validate_public_key(publicKey)).toBe(true);
    });

    test('should reject invalid public keys', () => {
      const invalidKeys = [
        new Uint8Array(31), // Too short
        new Uint8Array(33), // Too long
        new Uint8Array(0),  // Empty
        new Uint8Array(64)  // Wrong length
      ];

      invalidKeys.forEach(key => {
        expect(wasmExports.validate_public_key(key)).toBe(false);
      });
    });

    test('should enforce parameter contracts', () => {
      expect(() => wasmExports.validate_public_key()).toThrow(ContractViolationError);
      expect(() => wasmExports.validate_public_key('not bytes')).toThrow(ContractViolationError);
    });
  });

  describe('validate_signature', () => {
    test('should validate correct signatures', () => {
      const kp = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode('test data');
      const signature = kp.sign(data);
      
      expect(wasmExports.validate_signature(signature)).toBe(true);
    });

    test('should reject invalid signatures', () => {
      const invalidSignatures = [
        new Uint8Array(63), // Too short
        new Uint8Array(65), // Too long
        new Uint8Array(0),  // Empty
        new Uint8Array(32)  // Wrong length
      ];

      invalidSignatures.forEach(sig => {
        expect(wasmExports.validate_signature(sig)).toBe(false);
      });
    });
  });

  describe('verify_signature', () => {
    test('should verify correct signatures', () => {
      const kp = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode('test message');
      const signature = kp.sign(data);
      
      const isValid = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      expect(isValid).toBe(true);
    });

    test('should reject signatures with wrong key', () => {
      const kp1 = new wasmExports.WasmKeyPair();
      const kp2 = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode('test message');
      const signature = kp1.sign(data);
      
      const isValid = wasmExports.verify_signature(kp2.public_key_bytes, data, signature);
      expect(isValid).toBe(false);
    });

    test('should reject signatures with tampered data', () => {
      const kp = new wasmExports.WasmKeyPair();
      const originalData = new TextEncoder().encode('original message');
      const tamperedData = new TextEncoder().encode('tampered message');
      const signature = kp.sign(originalData);
      
      const isValid = wasmExports.verify_signature(kp.public_key_bytes, tamperedData, signature);
      expect(isValid).toBe(false);
    });

    test('should enforce parameter contracts', () => {
      const kp = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode('test');
      const signature = kp.sign(data);
      
      // Missing parameters
      expect(() => wasmExports.verify_signature()).toThrow(ContractViolationError);
      expect(() => wasmExports.verify_signature(kp.public_key_bytes)).toThrow(ContractViolationError);
      
      // Wrong types
      expect(() => wasmExports.verify_signature('not bytes', data, signature)).toThrow(ContractViolationError);
      expect(() => wasmExports.verify_signature(kp.public_key_bytes, 'not bytes', signature)).toThrow(ContractViolationError);
      expect(() => wasmExports.verify_signature(kp.public_key_bytes, data, 'not bytes')).toThrow(ContractViolationError);
      
      // Wrong sizes
      expect(() => wasmExports.verify_signature(new Uint8Array(31), data, signature)).toThrow(ContractViolationError);
      expect(() => wasmExports.verify_signature(kp.public_key_bytes, data, new Uint8Array(63))).toThrow(ContractViolationError);
    });
  });
});

describe('WasmKeyPair Class', () => {
  test('should create keypairs with correct properties', () => {
    const kp = new wasmExports.WasmKeyPair();
    
    expect(kp).toBeInstanceOf(wasmExports.WasmKeyPair);
    expect(kp.public_key_bytes).toBeInstanceOf(Uint8Array);
    expect(kp.public_key_bytes).toHaveLength(32);
    expect(kp.private_key_bytes).toBeInstanceOf(Uint8Array);
    expect(kp.private_key_bytes).toHaveLength(32);
    expect(kp.keypair_bytes).toBeInstanceOf(Uint8Array);
    expect(kp.keypair_bytes).toHaveLength(64);
    expect(kp.public_key_hex).toBeTypeOf('string');
    expect(kp.public_key_hex).toHaveLength(64);
    expect(kp.public_key_hex).toMatch(/^[0-9a-f]+$/);
  });

  test('should create unique keypairs', () => {
    const kp1 = new wasmExports.WasmKeyPair();
    const kp2 = new wasmExports.WasmKeyPair();
    
    expect(kp1.public_key_hex).not.toBe(kp2.public_key_hex);
    expect(kp1.public_key_bytes).not.toEqual(kp2.public_key_bytes);
  });

  test('should sign data correctly', () => {
    const kp = new wasmExports.WasmKeyPair();
    const data = new TextEncoder().encode('test message');
    
    const signature = kp.sign(data);
    expect(signature).toBeInstanceOf(Uint8Array);
    expect(signature).toHaveLength(64);
    
    const isValid = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
    expect(isValid).toBe(true);
  });

  test('should create keypair from bytes', () => {
    const kp1 = new wasmExports.WasmKeyPair();
    const keypairBytes = kp1.keypair_bytes;
    
    const kp2 = wasmExports.WasmKeyPair.from_bytes(keypairBytes);
    
    expect(kp2.public_key_bytes).toEqual(kp1.public_key_bytes);
    expect(kp2.private_key_bytes).toEqual(kp1.private_key_bytes);
    expect(kp2.public_key_hex).toBe(kp1.public_key_hex);
  });

  test('should enforce constructor contracts', () => {
    // Constructor should not throw with no parameters
    expect(() => new wasmExports.WasmKeyPair()).not.toThrow();
  });

  test('should enforce method contracts', () => {
    const kp = new wasmExports.WasmKeyPair();
    
    // sign method contracts
    expect(() => kp.sign()).toThrow(ContractViolationError);
    expect(() => kp.sign('not bytes')).toThrow(ContractViolationError);
    expect(() => kp.sign(null)).toThrow(ContractViolationError);
    
    // from_bytes static method contracts
    expect(() => wasmExports.WasmKeyPair.from_bytes()).toThrow(ContractViolationError);
    expect(() => wasmExports.WasmKeyPair.from_bytes('not bytes')).toThrow(ContractViolationError);
    expect(() => wasmExports.WasmKeyPair.from_bytes(new Uint8Array(63))).toThrow(ContractViolationError);
  });
});

describe('WasmMessage Class', () => {
  let alice, bob;
  
  beforeAll(() => {
    alice = new wasmExports.WasmKeyPair();
    bob = new wasmExports.WasmKeyPair();
  });

  test('should create messages with correct properties', () => {
    const message = new wasmExports.WasmMessage(
      alice.public_key_bytes,
      bob.public_key_bytes,
      'Hello Bob!'
    );
    
    expect(message).toBeInstanceOf(wasmExports.WasmMessage);
    expect(message.id).toBeTypeOf('string');
    expect(message.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    expect(message.sender_hex).toHaveLength(64);
    expect(message.recipient_hex).toHaveLength(64);
    expect(message.sender_bytes).toEqual(alice.public_key_bytes);
    expect(message.recipient_bytes).toEqual(bob.public_key_bytes);
    expect(message.content).toBe('Hello Bob!');
    expect(message.timestamp).toBeTypeOf('string');
    expect(message.is_signed).toBe(false);
  });

  test('should sign and verify messages', () => {
    const message = new wasmExports.WasmMessage(
      alice.public_key_bytes,
      bob.public_key_bytes,
      'Test message'
    );
    
    expect(message.is_signed).toBe(false);
    
    message.sign(alice.keypair_bytes);
    expect(message.is_signed).toBe(true);
    
    const verified = message.verify(alice.public_key_bytes);
    expect(verified).toBe(true);
    
    const wrongVerified = message.verify(bob.public_key_bytes);
    expect(wrongVerified).toBe(false);
  });

  test('should serialize and deserialize messages', () => {
    const originalMessage = new wasmExports.WasmMessage(
      alice.public_key_bytes,
      bob.public_key_bytes,
      'Serialization test'
    );
    
    originalMessage.sign(alice.keypair_bytes);
    
    const json = originalMessage.to_json();
    expect(json).toBeTypeOf('string');
    
    const deserializedMessage = wasmExports.WasmMessage.from_json(json);
    expect(deserializedMessage).toBeInstanceOf(wasmExports.WasmMessage);
    expect(deserializedMessage.id).toBe(originalMessage.id);
    expect(deserializedMessage.content).toBe(originalMessage.content);
    expect(deserializedMessage.is_signed).toBe(true);
  });

  test('should handle serialization errors gracefully', () => {
    expect(wasmExports.WasmMessage.from_json('invalid json')).toBeUndefined();
    expect(wasmExports.WasmMessage.from_json('')).toBeUndefined();
  });

  test('should enforce constructor contracts', () => {
    // Missing parameters
    expect(() => new wasmExports.WasmMessage()).toThrow(ContractViolationError);
    expect(() => new wasmExports.WasmMessage(alice.public_key_bytes)).toThrow(ContractViolationError);
    expect(() => new wasmExports.WasmMessage(alice.public_key_bytes, bob.public_key_bytes)).toThrow(ContractViolationError);
    
    // Wrong types
    expect(() => new wasmExports.WasmMessage('not bytes', bob.public_key_bytes, 'test')).toThrow(ContractViolationError);
    expect(() => new wasmExports.WasmMessage(alice.public_key_bytes, 'not bytes', 'test')).toThrow(ContractViolationError);
    expect(() => new wasmExports.WasmMessage(alice.public_key_bytes, bob.public_key_bytes, 123)).toThrow(ContractViolationError);
    
    // Wrong sizes
    expect(() => new wasmExports.WasmMessage(new Uint8Array(31), bob.public_key_bytes, 'test')).toThrow(ContractViolationError);
    expect(() => new wasmExports.WasmMessage(alice.public_key_bytes, new Uint8Array(33), 'test')).toThrow(ContractViolationError);
  });

  test('should enforce method contracts', () => {
    const message = new wasmExports.WasmMessage(
      alice.public_key_bytes,
      bob.public_key_bytes,
      'Contract test'
    );
    
    // sign method contracts
    expect(() => message.sign()).toThrow(ContractViolationError);
    expect(() => message.sign('not bytes')).toThrow(ContractViolationError);
    expect(() => message.sign(new Uint8Array(63))).toThrow(ContractViolationError);
    
    // verify method contracts
    expect(() => message.verify()).toThrow(ContractViolationError);
    expect(() => message.verify('not bytes')).toThrow(ContractViolationError);
    expect(() => message.verify(new Uint8Array(31))).toThrow(ContractViolationError);
    
    // from_json static method contracts
    expect(() => wasmExports.WasmMessage.from_json()).toThrow(ContractViolationError);
    expect(() => wasmExports.WasmMessage.from_json(123)).toThrow(ContractViolationError);
  });
});

describe('Performance and Stress Testing', () => {
  test('should handle high-frequency operations', () => {
    const iterations = 1000;
    const startTime = Date.now();
    
    for (let i = 0; i < iterations; i++) {
      const kp = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode(`test ${i}`);
      const signature = kp.sign(data);
      const verified = wasmExports.verify_signature(kp.public_key_bytes, data, signature);
      expect(verified).toBe(true);
    }
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    const avgTime = duration / iterations;
    
    console.log(`Performance test: ${iterations} operations in ${duration}ms (${avgTime.toFixed(2)}ms avg)`);
    expect(avgTime).toBeLessThan(100); // Should be fast
  });

  test('should handle large data signing', () => {
    const kp = new wasmExports.WasmKeyPair();
    const largeData = new Uint8Array(1_000_000).fill(0xaa); // 1MB
    
    const startTime = Date.now();
    const signature = kp.sign(largeData);
    const verified = wasmExports.verify_signature(kp.public_key_bytes, largeData, signature);
    const endTime = Date.now();
    
    expect(verified).toBe(true);
    expect(endTime - startTime).toBeLessThan(1000); // Should complete within 1 second
  });

  test('should handle concurrent operations', async () => {
    const promises = Array.from({ length: 100 }, async (_, i) => {
      const kp = new wasmExports.WasmKeyPair();
      const data = new TextEncoder().encode(`concurrent test ${i}`);
      const signature = kp.sign(data);
      return wasmExports.verify_signature(kp.public_key_bytes, data, signature);
    });
    
    const results = await Promise.all(promises);
    expect(results.every(result => result === true)).toBe(true);
  });
});

describe('Error Handling and Edge Cases', () => {
  test('should handle empty data gracefully', () => {
    const kp = new wasmExports.WasmKeyPair();
    const emptyData = new Uint8Array(0);
    
    const signature = kp.sign(emptyData);
    const verified = wasmExports.verify_signature(kp.public_key_bytes, emptyData, signature);
    
    expect(verified).toBe(true);
  });

  test('should handle unicode content', () => {
    const kp = new wasmExports.WasmKeyPair();
    const unicodeContent = 'Hello 世界 🌍 Здравствуй мир';
    
    const message = new wasmExports.WasmMessage(
      kp.public_key_bytes,
      kp.public_key_bytes,
      unicodeContent
    );
    
    message.sign(kp.keypair_bytes);
    const verified = message.verify(kp.public_key_bytes);
    
    expect(verified).toBe(true);
    expect(message.content).toBe(unicodeContent);
  });

  test('should maintain consistency across operations', () => {
    const kp = new wasmExports.WasmKeyPair();
    
    // Multiple operations should be consistent
    const data1 = new TextEncoder().encode('test data');
    const sig1a = kp.sign(data1);
    const sig1b = kp.sign(data1);
    
    expect(sig1a).toEqual(sig1b); // Same data should produce same signature
    
    const verified1a = wasmExports.verify_signature(kp.public_key_bytes, data1, sig1a);
    const verified1b = wasmExports.verify_signature(kp.public_key_bytes, data1, sig1b);
    
    expect(verified1a).toBe(true);
    expect(verified1b).toBe(true);
  });
});

describe('Contract Validation Report', () => {
  test('should generate comprehensive validation report', () => {
    // Perform some operations to generate test data
    const kp = new wasmExports.WasmKeyPair();
    const invite = wasmExports.generate_invite_code();
    wasmExports.validate_invite_code(invite);
    
    const report = validator.getReport();
    
    expect(report).toHaveProperty('totalTests');
    expect(report).toHaveProperty('violations');
    expect(report).toHaveProperty('passedTests');
    expect(report).toHaveProperty('failedTests');
    expect(report).toHaveProperty('testResults');
    
    expect(report.totalTests).toBeGreaterThan(0);
    expect(report.passedTests).toBeGreaterThan(0);
    
    console.log('Contract Validation Report:', report);
  });
});