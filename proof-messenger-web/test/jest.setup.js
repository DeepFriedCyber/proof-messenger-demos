/**
 * Jest test setup for biometric proof generator tests
 * Sets up browser API mocks for WebAuthn testing
 */

// Mock TextEncoder/TextDecoder for Node.js environment
global.TextEncoder = class {
    encode(str) {
        return new Uint8Array(Buffer.from(str, 'utf8'));
    }
};

global.TextDecoder = class {
    decode(buffer) {
        return Buffer.from(buffer).toString('utf8');
    }
};

// Mock crypto.getRandomValues
global.crypto = {
    getRandomValues: jest.fn((array) => {
        for (let i = 0; i < array.length; i++) {
            array[i] = Math.floor(Math.random() * 256);
        }
        return array;
    }),
};

// Mock btoa for base64 encoding
global.btoa = (str) => Buffer.from(str, 'binary').toString('base64');

// Mock atob for base64 decoding
global.atob = (str) => Buffer.from(str, 'base64').toString('binary');