/**
 * End-to-End Test Suite: Complete User Journey
 * 
 * This test suite validates the entire Proof Messenger Protocol workflow:
 * 1. Web app loads and initializes WASM
 * 2. User generates a keypair
 * 3. User signs a message
 * 4. Message is sent to relay server
 * 5. Relay server verifies the cryptographic proof
 * 6. Success is confirmed in the UI
 * 
 * Following TDD principles: Write the test first, then ensure implementation works.
 */

import { test, expect } from '@playwright/test';

test.describe('Proof Messenger E2E Journey', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to the demo page
    await page.goto('/demo.html');
    
    // Wait for WASM to load - this is critical for crypto operations
    await page.waitForSelector('#wasm-status:has-text("✅ Loaded")', { timeout: 30000 });
    
    // Verify initial state
    await expect(page.locator('#status-text')).toHaveText('Uninitialized');
    await expect(page.locator('#has-keypair')).toHaveText('❌ No');
    await expect(page.locator('#is-ready')).toHaveText('❌ No');
  });

  test('complete user journey: generate key, sign message, and verify end-to-end', async ({ page }) => {
    // Step 1: Generate Keypair
    console.log('🔑 Step 1: Generating keypair...');
    
    // Click generate button
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    
    // Wait for key generation to complete
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    await expect(page.locator('#has-keypair')).toHaveText('✅ Yes');
    await expect(page.locator('#is-ready')).toHaveText('✅ Yes');
    
    // Verify public key is displayed
    const publicKeyDisplay = page.locator('#public-key-display');
    await expect(publicKeyDisplay).toBeVisible();
    
    const publicKeyText = await page.locator('#public-key').textContent();
    expect(publicKeyText).toBeTruthy();
    expect(publicKeyText.length).toBeGreaterThan(50); // Ed25519 public key in hex
    
    console.log('✅ Step 1 Complete: Keypair generated successfully');

    // Step 2: Sign a Message
    console.log('✍️ Step 2: Signing message...');
    
    const testMessage = 'Hello from Playwright E2E test!';
    
    // Fill in the message
    await page.locator('#message-input').fill(testMessage);
    
    // Click sign button
    await page.getByRole('button', { name: /Sign Message/i }).click();
    
    // Wait for signature to appear
    const signatureDisplay = page.locator('#signature-display');
    await expect(signatureDisplay).toBeVisible({ timeout: 5000 });
    
    const signatureText = await page.locator('#signature').textContent();
    expect(signatureText).toBeTruthy();
    expect(signatureText.length).toBe(128); // Ed25519 signature is 64 bytes = 128 hex chars
    
    console.log('✅ Step 2 Complete: Message signed successfully');

    // Step 3: Prepare Message for Relay Server
    console.log('📡 Step 3: Preparing message for relay server...');
    
    // Extract the cryptographic components
    const publicKey = publicKeyText;
    const signature = signatureText;
    const context = Array.from(new TextEncoder().encode(testMessage))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    
    // Verify we have all required components
    expect(publicKey).toBeTruthy();
    expect(signature).toBeTruthy();
    expect(context).toBeTruthy();
    
    console.log('✅ Step 3 Complete: Message components prepared');

    // Step 4: Send Message to Relay Server
    console.log('🚀 Step 4: Sending message to relay server...');
    
    const relayMessage = {
      sender: publicKey,
      context: context,
      body: testMessage,
      proof: signature
    };
    
    // Make API call to relay server
    const response = await page.request.post('http://localhost:8080/relay', {
      data: relayMessage,
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    // Verify successful response
    expect(response.status()).toBe(200);
    
    const responseBody = await response.json();
    expect(responseBody.status).toBe('success');
    expect(responseBody.message).toContain('verified and relayed successfully');
    
    console.log('✅ Step 4 Complete: Message verified by relay server');

    // Step 5: Verify Security Properties
    console.log('🛡️ Step 5: Verifying security properties...');
    
    // Show diagnostics to verify security
    await page.getByRole('button', { name: /Show Security Diagnostics/i }).click();
    
    const diagnosticsDisplay = page.locator('#diagnostics-display');
    await expect(diagnosticsDisplay).toBeVisible();
    
    const diagnosticsText = await page.locator('#diagnostics').textContent();
    expect(diagnosticsText).toContain('Private key is securely encapsulated');
    expect(diagnosticsText).toContain('WASM boundary protection active');
    
    console.log('✅ Step 5 Complete: Security properties verified');

    console.log('🎉 E2E Test Complete: Full user journey successful!');
  });

  test('error handling: invalid message rejection by relay server', async ({ page }) => {
    console.log('❌ Testing error handling with invalid message...');
    
    // Generate a keypair first
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    const publicKeyText = await page.locator('#public-key').textContent();
    
    // Create an invalid message (tampered signature)
    const invalidMessage = {
      sender: publicKeyText,
      context: '48656c6c6f', // "Hello" in hex
      body: 'Hello',
      proof: '0'.repeat(128) // Invalid signature (all zeros)
    };
    
    // Send invalid message to relay server
    const response = await page.request.post('http://localhost:8080/relay', {
      data: invalidMessage,
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    // Verify rejection
    expect(response.status()).toBe(401); // Unauthorized due to verification failure
    
    const responseBody = await response.json();
    expect(responseBody.error).toContain('Proof verification failed');
    
    console.log('✅ Error handling test complete: Invalid messages properly rejected');
  });

  test('cross-browser compatibility: key generation and signing', async ({ page, browserName }) => {
    console.log(`🌐 Testing cross-browser compatibility on ${browserName}...`);
    
    // Generate keypair
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 15000 });
    
    // Verify WASM works across browsers
    const publicKeyText = await page.locator('#public-key').textContent();
    expect(publicKeyText).toBeTruthy();
    expect(publicKeyText.length).toBeGreaterThan(50);
    
    // Test signing
    await page.locator('#message-input').fill(`Cross-browser test on ${browserName}`);
    await page.getByRole('button', { name: /Sign Message/i }).click();
    
    const signatureText = await page.locator('#signature').textContent();
    expect(signatureText).toBeTruthy();
    expect(signatureText.length).toBe(128);
    
    console.log(`✅ Cross-browser test complete: ${browserName} fully compatible`);
  });

  test('performance: key generation and signing speed', async ({ page }) => {
    console.log('⚡ Testing performance metrics...');
    
    const startTime = Date.now();
    
    // Generate keypair and measure time
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    const keyGenTime = Date.now() - startTime;
    console.log(`🔑 Key generation time: ${keyGenTime}ms`);
    
    // Signing performance
    await page.locator('#message-input').fill('Performance test message');
    
    const signStartTime = Date.now();
    await page.getByRole('button', { name: /Sign Message/i }).click();
    await expect(page.locator('#signature-display')).toBeVisible();
    const signTime = Date.now() - signStartTime;
    
    console.log(`✍️ Signing time: ${signTime}ms`);
    
    // Performance assertions (reasonable thresholds)
    expect(keyGenTime).toBeLessThan(5000); // Key generation should be under 5 seconds
    expect(signTime).toBeLessThan(1000);   // Signing should be under 1 second
    
    console.log('✅ Performance test complete: All operations within acceptable limits');
  });

  test('ui state management: reset functionality', async ({ page }) => {
    console.log('🔄 Testing UI state management and reset...');
    
    // Generate keypair and sign message
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    await page.locator('#message-input').fill('Test message for reset');
    await page.getByRole('button', { name: /Sign Message/i }).click();
    await expect(page.locator('#signature-display')).toBeVisible();
    
    // Verify state before reset
    await expect(page.locator('#has-keypair')).toHaveText('✅ Yes');
    await expect(page.locator('#public-key-display')).toBeVisible();
    await expect(page.locator('#signature-display')).toBeVisible();
    
    // Reset
    await page.getByRole('button', { name: /Reset/i }).click();
    
    // Verify state after reset
    await expect(page.locator('#status-text')).toHaveText('Uninitialized');
    await expect(page.locator('#has-keypair')).toHaveText('❌ No');
    await expect(page.locator('#is-ready')).toHaveText('❌ No');
    await expect(page.locator('#public-key-display')).toBeHidden();
    await expect(page.locator('#signature-display')).toBeHidden();
    
    console.log('✅ Reset functionality test complete: State properly cleared');
  });

  test('security validation: private key never exposed', async ({ page }) => {
    console.log('🔐 Testing security: private key encapsulation...');
    
    // Generate keypair
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    // Show diagnostics
    await page.getByRole('button', { name: /Show Security Diagnostics/i }).click();
    
    const diagnosticsText = await page.locator('#diagnostics').textContent();
    
    // Verify security properties
    expect(diagnosticsText).toContain('Private key is securely encapsulated');
    expect(diagnosticsText).toContain('WASM boundary protection active');
    expect(diagnosticsText).toContain('No private key material in JavaScript');
    
    // Verify that attempting to access private key through browser console fails
    const privateKeyAccess = await page.evaluate(() => {
      try {
        // Try to access any global variables that might contain private keys
        const globals = Object.keys(window);
        const suspiciousGlobals = globals.filter(key => 
          key.toLowerCase().includes('private') || 
          key.toLowerCase().includes('secret') ||
          key.toLowerCase().includes('key')
        );
        return { accessible: suspiciousGlobals.length > 0, globals: suspiciousGlobals };
      } catch (error) {
        return { accessible: false, error: error.message };
      }
    });
    
    // Private keys should not be accessible through global scope
    expect(privateKeyAccess.accessible).toBe(false);
    
    console.log('✅ Security validation complete: Private keys properly encapsulated');
  });

  test('relay server integration: multiple message verification', async ({ page }) => {
    console.log('📡 Testing relay server with multiple messages...');
    
    // Generate keypair
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    const publicKeyText = await page.locator('#public-key').textContent();
    
    // Send multiple messages with different contexts
    const testMessages = [
      'First test message',
      'Second test message',
      'Third test message with special chars: 🚀🔐✅'
    ];
    
    for (let i = 0; i < testMessages.length; i++) {
      const message = testMessages[i];
      console.log(`📤 Sending message ${i + 1}: "${message}"`);
      
      // Sign the message
      await page.locator('#message-input').fill(message);
      await page.getByRole('button', { name: /Sign Message/i }).click();
      await expect(page.locator('#signature-display')).toBeVisible();
      
      const signature = await page.locator('#signature').textContent();
      const context = Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
      
      // Send to relay server
      const response = await page.request.post('http://localhost:8080/relay', {
        data: {
          sender: publicKeyText,
          context: context,
          body: message,
          proof: signature
        },
        headers: {
          'Content-Type': 'application/json'
        }
      });
      
      expect(response.status()).toBe(200);
      const responseBody = await response.json();
      expect(responseBody.status).toBe('success');
      
      console.log(`✅ Message ${i + 1} verified successfully`);
    }
    
    console.log('✅ Multiple message test complete: All messages verified');
  });
});

test.describe('Error Scenarios and Edge Cases', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/demo.html');
    await page.waitForSelector('#wasm-status:has-text("✅ Loaded")', { timeout: 30000 });
  });

  test('malformed requests to relay server', async ({ page }) => {
    console.log('🚨 Testing malformed request handling...');
    
    const malformedRequests = [
      // Missing fields
      { sender: 'test' },
      // Invalid hex encoding
      { sender: 'invalid_hex', context: 'invalid_hex', body: 'test', proof: 'invalid_hex' },
      // Wrong field types
      { sender: 123, context: 456, body: 789, proof: 'abc' },
      // Empty request
      {}
    ];
    
    for (const request of malformedRequests) {
      const response = await page.request.post('http://localhost:8080/relay', {
        data: request,
        headers: {
          'Content-Type': 'application/json'
        }
      });
      
      // Should return 400 Bad Request for malformed data
      expect(response.status()).toBe(400);
      
      const responseBody = await response.json();
      expect(responseBody.error).toBeTruthy();
    }
    
    console.log('✅ Malformed request test complete: All properly rejected');
  });

  test('wasm loading failure simulation', async ({ page }) => {
    console.log('⚠️ Testing WASM loading failure handling...');
    
    // Block WASM file loading to simulate failure
    await page.route('**/pkg/proof_messenger_web_bg.wasm', route => {
      route.abort();
    });
    
    // Navigate to page
    await page.goto('/demo.html');
    
    // Wait a bit for loading attempt
    await page.waitForTimeout(5000);
    
    // Verify that WASM status shows error
    const wasmStatus = await page.locator('#wasm-status').textContent();
    expect(wasmStatus).toContain('❌');
    
    // Verify that generate button is disabled
    const generateBtn = page.getByRole('button', { name: /Generate New Keypair/i });
    await expect(generateBtn).toBeDisabled();
    
    console.log('✅ WASM failure test complete: Graceful error handling verified');
  });
});

test.describe('Accessibility and Usability', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/demo.html');
    await page.waitForSelector('#wasm-status:has-text("✅ Loaded")', { timeout: 30000 });
  });

  test('keyboard navigation and accessibility', async ({ page }) => {
    console.log('♿ Testing keyboard navigation and accessibility...');
    
    // Test tab navigation
    await page.keyboard.press('Tab'); // Should focus on generate button
    await expect(page.getByRole('button', { name: /Generate New Keypair/i })).toBeFocused();
    
    // Generate keypair using keyboard
    await page.keyboard.press('Enter');
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    // Navigate to message input
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab'); // Skip reset button
    await expect(page.locator('#message-input')).toBeFocused();
    
    // Type message using keyboard
    await page.keyboard.type('Accessibility test message');
    
    // Navigate to sign button and activate
    await page.keyboard.press('Tab');
    await expect(page.getByRole('button', { name: /Sign Message/i })).toBeFocused();
    await page.keyboard.press('Enter');
    
    await expect(page.locator('#signature-display')).toBeVisible();
    
    console.log('✅ Accessibility test complete: Full keyboard navigation working');
  });

  test('responsive design on mobile viewport', async ({ page }) => {
    console.log('📱 Testing responsive design...');
    
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Verify layout adapts
    const container = page.locator('.container');
    await expect(container).toBeVisible();
    
    // Test functionality on mobile
    await page.getByRole('button', { name: /Generate New Keypair/i }).click();
    await expect(page.locator('#status-text')).toHaveText('Ready', { timeout: 10000 });
    
    // Verify public key display is readable on mobile
    const publicKeyDisplay = page.locator('#public-key-display');
    await expect(publicKeyDisplay).toBeVisible();
    
    console.log('✅ Responsive design test complete: Mobile layout working');
  });
});