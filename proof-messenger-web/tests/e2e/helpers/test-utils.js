/**
 * E2E Test Utilities
 * 
 * Common helper functions and utilities for E2E tests.
 * These functions encapsulate common test patterns and make tests more maintainable.
 */

/**
 * Wait for WASM module to load and be ready
 * @param {import('@playwright/test').Page} page 
 * @param {number} timeout - Timeout in milliseconds
 */
export async function waitForWasmReady(page, timeout = 30000) {
  await page.waitForSelector('#wasm-status:has-text("✅ Loaded")', { timeout });
}

/**
 * Generate a keypair and wait for completion
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<string>} The generated public key
 */
export async function generateKeypair(page) {
  await page.getByRole('button', { name: /Generate New Keypair/i }).click();
  await page.waitForSelector('#status-text:has-text("Ready")', { timeout: 10000 });
  
  const publicKey = await page.locator('#public-key').textContent();
  return publicKey;
}

/**
 * Sign a message and return the signature
 * @param {import('@playwright/test').Page} page 
 * @param {string} message - Message to sign
 * @returns {Promise<string>} The signature in hex format
 */
export async function signMessage(page, message) {
  await page.locator('#message-input').fill(message);
  await page.getByRole('button', { name: /Sign Message/i }).click();
  await page.waitForSelector('#signature-display', { state: 'visible', timeout: 5000 });
  
  const signature = await page.locator('#signature').textContent();
  return signature;
}

/**
 * Send a message to the relay server
 * @param {import('@playwright/test').Page} page 
 * @param {Object} message - Message object with sender, context, body, proof
 * @returns {Promise<Object>} Response from the relay server
 */
export async function sendToRelayServer(page, message) {
  const response = await page.request.post('http://localhost:8080/relay', {
    data: message,
    headers: {
      'Content-Type': 'application/json'
    }
  });
  
  const responseBody = await response.json();
  return { status: response.status(), body: responseBody };
}

/**
 * Create a complete message object for relay server
 * @param {string} publicKey - Public key in hex
 * @param {string} messageText - Message text
 * @param {string} signature - Signature in hex
 * @returns {Object} Complete message object
 */
export function createRelayMessage(publicKey, messageText, signature) {
  const context = Array.from(new TextEncoder().encode(messageText))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  return {
    sender: publicKey,
    context: context,
    body: messageText,
    proof: signature
  };
}

/**
 * Perform complete E2E workflow: generate key, sign message, send to relay
 * @param {import('@playwright/test').Page} page 
 * @param {string} message - Message to sign and send
 * @returns {Promise<Object>} Result object with publicKey, signature, and relayResponse
 */
export async function completeE2EWorkflow(page, message) {
  // Generate keypair
  const publicKey = await generateKeypair(page);
  
  // Sign message
  const signature = await signMessage(page, message);
  
  // Create relay message
  const relayMessage = createRelayMessage(publicKey, message, signature);
  
  // Send to relay server
  const relayResponse = await sendToRelayServer(page, relayMessage);
  
  return {
    publicKey,
    signature,
    relayMessage,
    relayResponse
  };
}

/**
 * Verify security diagnostics
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<string>} Diagnostics text
 */
export async function getSecurityDiagnostics(page) {
  await page.getByRole('button', { name: /Show Security Diagnostics/i }).click();
  await page.waitForSelector('#diagnostics-display', { state: 'visible' });
  
  const diagnosticsText = await page.locator('#diagnostics').textContent();
  return diagnosticsText;
}

/**
 * Reset the application state
 * @param {import('@playwright/test').Page} page 
 */
export async function resetApplication(page) {
  await page.getByRole('button', { name: /Reset/i }).click();
  await page.waitForSelector('#status-text:has-text("Uninitialized")');
}

/**
 * Verify initial application state
 * @param {import('@playwright/test').Page} page 
 */
export async function verifyInitialState(page) {
  const { expect } = await import('@playwright/test');
  
  await expect(page.locator('#status-text')).toHaveText('Uninitialized');
  await expect(page.locator('#has-keypair')).toHaveText('❌ No');
  await expect(page.locator('#is-ready')).toHaveText('❌ No');
}

/**
 * Verify ready state after key generation
 * @param {import('@playwright/test').Page} page 
 */
export async function verifyReadyState(page) {
  const { expect } = await import('@playwright/test');
  
  await expect(page.locator('#status-text')).toHaveText('Ready');
  await expect(page.locator('#has-keypair')).toHaveText('✅ Yes');
  await expect(page.locator('#is-ready')).toHaveText('✅ Yes');
  await expect(page.locator('#public-key-display')).toBeVisible();
}

/**
 * Create test data for various scenarios
 */
export const TestData = {
  messages: {
    simple: 'Hello, World!',
    unicode: 'Unicode test: 🚀🔐✅ 测试',
    long: 'A'.repeat(1000),
    empty: '',
    special: 'Special chars: !@#$%^&*()_+-=[]{}|;:,.<>?',
    json: '{"test": "message", "number": 42}',
    html: '<script>alert("test")</script>',
  },
  
  invalidHex: {
    notHex: 'not_hex_string',
    wrongLength: '1234',
    empty: '',
  },
  
  malformedMessages: [
    { sender: 'test' }, // Missing fields
    { sender: 'invalid_hex', context: 'invalid_hex', body: 'test', proof: 'invalid_hex' },
    { sender: 123, context: 456, body: 789, proof: 'abc' }, // Wrong types
    {}, // Empty
  ]
};

/**
 * Performance measurement utilities
 */
export class PerformanceTracker {
  constructor() {
    this.measurements = {};
  }
  
  start(name) {
    this.measurements[name] = { start: Date.now() };
  }
  
  end(name) {
    if (this.measurements[name]) {
      this.measurements[name].end = Date.now();
      this.measurements[name].duration = this.measurements[name].end - this.measurements[name].start;
    }
  }
  
  getDuration(name) {
    return this.measurements[name]?.duration || 0;
  }
  
  getAllMeasurements() {
    return this.measurements;
  }
  
  logResults() {
    console.log('⚡ Performance Results:');
    Object.entries(this.measurements).forEach(([name, data]) => {
      if (data.duration !== undefined) {
        console.log(`  ${name}: ${data.duration}ms`);
      }
    });
  }
}

/**
 * Browser compatibility utilities
 */
export function getBrowserInfo(browserName) {
  const browserFeatures = {
    chromium: {
      name: 'Chromium',
      wasmSupport: true,
      cryptoSupport: true,
      expectedFeatures: ['WebAssembly', 'crypto.subtle']
    },
    firefox: {
      name: 'Firefox',
      wasmSupport: true,
      cryptoSupport: true,
      expectedFeatures: ['WebAssembly', 'crypto.subtle']
    },
    webkit: {
      name: 'WebKit/Safari',
      wasmSupport: true,
      cryptoSupport: true,
      expectedFeatures: ['WebAssembly', 'crypto.subtle']
    }
  };
  
  return browserFeatures[browserName] || { name: 'Unknown', wasmSupport: false };
}

/**
 * Wait for network idle (useful for WASM loading)
 * @param {import('@playwright/test').Page} page 
 * @param {number} timeout 
 */
export async function waitForNetworkIdle(page, timeout = 5000) {
  await page.waitForLoadState('networkidle', { timeout });
}

/**
 * Capture and log console messages during test execution
 * @param {import('@playwright/test').Page} page 
 */
export function setupConsoleLogging(page) {
  const logs = [];
  
  page.on('console', msg => {
    const logEntry = {
      type: msg.type(),
      text: msg.text(),
      timestamp: new Date().toISOString()
    };
    logs.push(logEntry);
    
    // Log errors and warnings to test output
    if (msg.type() === 'error' || msg.type() === 'warning') {
      console.log(`[${msg.type().toUpperCase()}] ${msg.text()}`);
    }
  });
  
  return {
    getLogs: () => logs,
    getErrors: () => logs.filter(log => log.type === 'error'),
    getWarnings: () => logs.filter(log => log.type === 'warning')
  };
}