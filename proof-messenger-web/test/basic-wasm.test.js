/**
 * Basic WASM Export Tests
 * 
 * Simple tests to verify WASM exports work without complex contract validation
 */

import { describe, test, expect, beforeAll } from 'vitest';
import init, {
    WasmKeyPair,
    WasmMessage,
    generate_invite_code,
    validate_invite_code,
    bytes_to_hex,
    hex_to_bytes
} from '../pkg/proof_messenger_web.js';

beforeAll(async () => {
  await init();
});

describe('Basic WASM Functionality', () => {
  test('should initialize WASM successfully', () => {
    expect(WasmKeyPair).toBeDefined();
    expect(WasmMessage).toBeDefined();
    expect(generate_invite_code).toBeDefined();
    expect(validate_invite_code).toBeDefined();
    expect(bytes_to_hex).toBeDefined();
    expect(hex_to_bytes).toBeDefined();
  });

  test('should generate keypairs', () => {
    const kp = new WasmKeyPair();
    
    expect(kp.public_key_bytes).toBeInstanceOf(Uint8Array);
    expect(kp.public_key_bytes).toHaveLength(32);
    expect(kp.private_key_bytes).toBeInstanceOf(Uint8Array);
    expect(kp.private_key_bytes).toHaveLength(32);
    expect(kp.public_key_hex).toMatch(/^[0-9a-f]{64}$/);
  });

  test('should generate and validate invite codes', () => {
    const invite = generate_invite_code();
    
    expect(invite).toMatch(/^[A-Z0-9]{16}$/);
    expect(validate_invite_code(invite)).toBe(true);
    expect(validate_invite_code('invalid')).toBe(false);
  });

  test('should convert between bytes and hex', () => {
    const bytes = new Uint8Array([0x01, 0x23, 0x45, 0x67]);
    const hex = bytes_to_hex(bytes);
    const backToBytes = hex_to_bytes(hex);
    
    expect(hex).toBe('01234567');
    expect(backToBytes).toEqual(bytes);
  });

  test('should create and sign messages', () => {
    const kp = new WasmKeyPair();
    const message = new WasmMessage(kp.public_key_bytes, kp.public_key_bytes, 'Hello!');
    
    expect(message.is_signed).toBe(false);
    
    message.sign(kp.keypair_bytes);
    expect(message.is_signed).toBe(true);
    
    const verified = message.verify(kp.public_key_bytes);
    expect(verified).toBe(true);
  });
});