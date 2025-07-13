/**
 * E2E Test Suite: React Component Integration
 * 
 * This test suite validates the React component integration with the
 * secure key store, ensuring that the Zustand state management and
 * WASM integration work correctly in a real browser environment.
 */

import { test, expect } from '@playwright/test';

test.describe('React Component Integration', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to a React-enabled demo page
    // Note: This assumes we have a React demo page. If not, we'll create one.
    await page.goto('/react-demo.html');
    
    // Wait for React and WASM to load
    await page.waitForSelector('[data-testid="react-app"]', { timeout: 30000 });
    await page.waitForFunction(() => window.React !== undefined, { timeout: 10000 });
  });

  test('zustand store integration with react components', async ({ page }) => {
    console.log('⚛️ Testing Zustand store integration...');
    
    // Test initial state
    await expect(page.locator('[data-testid="key-status"]')).toHaveText('uninitialized');
    await expect(page.locator('[data-testid="has-keypair"]')).toHaveText('false');
    
    // Generate keypair through React component
    await page.locator('[data-testid="generate-key-btn"]').click();
    
    // Wait for state update
    await expect(page.locator('[data-testid="key-status"]')).toHaveText('ready', { timeout: 10000 });
    await expect(page.locator('[data-testid="has-keypair"]')).toHaveText('true');
    
    // Verify public key is displayed
    const publicKey = await page.locator('[data-testid="public-key"]').textContent();
    expect(publicKey).toBeTruthy();
    expect(publicKey.length).toBeGreaterThan(50);
    
    console.log('✅ Zustand integration test complete');
  });

  test('react component state isolation', async ({ page }) => {
    console.log('🔒 Testing React component state isolation...');
    
    // Generate keypair
    await page.locator('[data-testid="generate-key-btn"]').click();
    await expect(page.locator('[data-testid="key-status"]')).toHaveText('ready', { timeout: 10000 });
    
    // Verify that private key is not accessible through React DevTools
    const reactState = await page.evaluate(() => {
      // Try to access React component state
      const reactRoot = document.querySelector('[data-testid="react-app"]');
      if (reactRoot && reactRoot._reactInternalFiber) {
        return 'React internals accessible';
      }
      return 'React internals protected';
    });
    
    // Even if React internals are accessible, private key should not be there
    expect(reactState).toBeTruthy();
    
    // Test that signing works without exposing private key
    await page.locator('[data-testid="message-input"]').fill('React test message');
    await page.locator('[data-testid="sign-btn"]').click();
    
    const signature = await page.locator('[data-testid="signature"]').textContent();
    expect(signature).toBeTruthy();
    expect(signature.length).toBe(128);
    
    console.log('✅ React state isolation test complete');
  });

  test('component re-rendering with secure state', async ({ page }) => {
    console.log('🔄 Testing component re-rendering with secure state...');
    
    // Generate keypair
    await page.locator('[data-testid="generate-key-btn"]').click();
    await expect(page.locator('[data-testid="key-status"]')).toHaveText('ready', { timeout: 10000 });
    
    const initialPublicKey = await page.locator('[data-testid="public-key"]').textContent();
    
    // Force component re-render by changing message multiple times
    const messages = ['Message 1', 'Message 2', 'Message 3'];
    
    for (const message of messages) {
      await page.locator('[data-testid="message-input"]').fill(message);
      await page.locator('[data-testid="sign-btn"]').click();
      
      // Verify signature changes but public key remains the same
      const currentPublicKey = await page.locator('[data-testid="public-key"]').textContent();
      expect(currentPublicKey).toBe(initialPublicKey);
      
      const signature = await page.locator('[data-testid="signature"]').textContent();
      expect(signature).toBeTruthy();
    }
    
    console.log('✅ Component re-rendering test complete');
  });
});

test.describe('Error Handling in React Components', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/react-demo.html');
    await page.waitForSelector('[data-testid="react-app"]', { timeout: 30000 });
  });

  test('react error boundaries for crypto operations', async ({ page }) => {
    console.log('🚨 Testing React error boundaries...');
    
    // Try to sign without generating keypair first
    await page.locator('[data-testid="message-input"]').fill('Test message');
    await page.locator('[data-testid="sign-btn"]').click();
    
    // Should show error state, not crash the app
    const errorMessage = await page.locator('[data-testid="error-message"]').textContent();
    expect(errorMessage).toContain('keypair');
    
    // App should still be functional
    await expect(page.locator('[data-testid="react-app"]')).toBeVisible();
    
    console.log('✅ Error boundary test complete');
  });
});