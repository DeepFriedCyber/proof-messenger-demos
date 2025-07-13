/**
 * E2E Performance and Load Testing
 * 
 * This test suite focuses on performance characteristics and load testing
 * of the Proof Messenger Protocol under various conditions.
 */

import { test, expect } from '@playwright/test';
import { 
  waitForWasmReady, 
  generateKeypair, 
  signMessage, 
  completeE2EWorkflow,
  PerformanceTracker,
  TestData 
} from './helpers/test-utils.js';

test.describe('Performance Testing', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/demo.html');
    await waitForWasmReady(page);
  });

  test('key generation performance benchmarks', async ({ page }) => {
    console.log('⚡ Testing key generation performance...');
    
    const tracker = new PerformanceTracker();
    const iterations = 5;
    const results = [];
    
    for (let i = 0; i < iterations; i++) {
      // Reset between iterations
      if (i > 0) {
        await page.getByRole('button', { name: /Reset/i }).click();
        await page.waitForSelector('#status-text:has-text("Uninitialized")');
      }
      
      tracker.start(`keygen_${i}`);
      await generateKeypair(page);
      tracker.end(`keygen_${i}`);
      
      const duration = tracker.getDuration(`keygen_${i}`);
      results.push(duration);
      console.log(`  Iteration ${i + 1}: ${duration}ms`);
    }
    
    // Calculate statistics
    const average = results.reduce((a, b) => a + b, 0) / results.length;
    const min = Math.min(...results);
    const max = Math.max(...results);
    
    console.log(`📊 Key Generation Performance:`);
    console.log(`  Average: ${average.toFixed(2)}ms`);
    console.log(`  Min: ${min}ms`);
    console.log(`  Max: ${max}ms`);
    
    // Performance assertions
    expect(average).toBeLessThan(3000); // Average should be under 3 seconds
    expect(max).toBeLessThan(5000);     // No single generation should take over 5 seconds
    
    console.log('✅ Key generation performance test complete');
  });

  test('message signing performance with various message sizes', async ({ page }) => {
    console.log('✍️ Testing signing performance with different message sizes...');
    
    // Generate keypair once
    await generateKeypair(page);
    
    const tracker = new PerformanceTracker();
    const messageSizes = [
      { name: 'tiny', size: 10 },
      { name: 'small', size: 100 },
      { name: 'medium', size: 1000 },
      { name: 'large', size: 10000 },
      { name: 'xlarge', size: 100000 }
    ];
    
    for (const { name, size } of messageSizes) {
      const message = 'A'.repeat(size);
      
      tracker.start(`sign_${name}`);
      await signMessage(page, message);
      tracker.end(`sign_${name}`);
      
      const duration = tracker.getDuration(`sign_${name}`);
      console.log(`  ${name} (${size} chars): ${duration}ms`);
      
      // Signing should be fast regardless of message size (Ed25519 property)
      expect(duration).toBeLessThan(2000);
    }
    
    console.log('✅ Message signing performance test complete');
  });

  test('concurrent operations stress test', async ({ page }) => {
    console.log('🔄 Testing concurrent operations...');
    
    await generateKeypair(page);
    
    const tracker = new PerformanceTracker();
    const concurrentOperations = 10;
    
    tracker.start('concurrent_signing');
    
    // Create multiple signing operations concurrently
    const signingPromises = [];
    for (let i = 0; i < concurrentOperations; i++) {
      const promise = signMessage(page, `Concurrent message ${i}`);
      signingPromises.push(promise);
    }
    
    // Wait for all to complete
    const signatures = await Promise.all(signingPromises);
    
    tracker.end('concurrent_signing');
    
    // Verify all signatures are unique and valid
    const uniqueSignatures = new Set(signatures);
    expect(uniqueSignatures.size).toBe(concurrentOperations);
    
    signatures.forEach(signature => {
      expect(signature).toBeTruthy();
      expect(signature.length).toBe(128);
    });
    
    const totalTime = tracker.getDuration('concurrent_signing');
    console.log(`  ${concurrentOperations} concurrent operations: ${totalTime}ms`);
    console.log(`  Average per operation: ${(totalTime / concurrentOperations).toFixed(2)}ms`);
    
    console.log('✅ Concurrent operations test complete');
  });

  test('memory usage during extended operations', async ({ page }) => {
    console.log('💾 Testing memory usage during extended operations...');
    
    await generateKeypair(page);
    
    // Perform many signing operations to test for memory leaks
    const iterations = 50;
    
    for (let i = 0; i < iterations; i++) {
      await signMessage(page, `Memory test message ${i}`);
      
      // Check every 10 iterations
      if (i % 10 === 0) {
        const memoryInfo = await page.evaluate(() => {
          if (performance.memory) {
            return {
              used: performance.memory.usedJSHeapSize,
              total: performance.memory.totalJSHeapSize,
              limit: performance.memory.jsHeapSizeLimit
            };
          }
          return null;
        });
        
        if (memoryInfo) {
          console.log(`  Iteration ${i}: Memory used: ${(memoryInfo.used / 1024 / 1024).toFixed(2)}MB`);
        }
      }
    }
    
    console.log('✅ Memory usage test complete');
  });
});

test.describe('Load Testing', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/demo.html');
    await waitForWasmReady(page);
  });

  test('relay server load test with multiple messages', async ({ page }) => {
    console.log('📡 Testing relay server load handling...');
    
    await generateKeypair(page);
    
    const messageCount = 20;
    const results = [];
    
    for (let i = 0; i < messageCount; i++) {
      const message = `Load test message ${i}`;
      
      const startTime = Date.now();
      const result = await completeE2EWorkflow(page, message);
      const endTime = Date.now();
      
      const duration = endTime - startTime;
      results.push({
        iteration: i,
        duration,
        success: result.relayResponse.status === 200
      });
      
      if (i % 5 === 0) {
        console.log(`  Processed ${i + 1}/${messageCount} messages`);
      }
    }
    
    // Analyze results
    const successful = results.filter(r => r.success).length;
    const failed = results.length - successful;
    const averageTime = results.reduce((sum, r) => sum + r.duration, 0) / results.length;
    
    console.log(`📊 Load Test Results:`);
    console.log(`  Total messages: ${messageCount}`);
    console.log(`  Successful: ${successful}`);
    console.log(`  Failed: ${failed}`);
    console.log(`  Success rate: ${((successful / messageCount) * 100).toFixed(2)}%`);
    console.log(`  Average time per message: ${averageTime.toFixed(2)}ms`);
    
    // Assertions
    expect(successful).toBe(messageCount); // All should succeed
    expect(averageTime).toBeLessThan(5000); // Average should be reasonable
    
    console.log('✅ Relay server load test complete');
  });

  test('browser resource utilization under load', async ({ page }) => {
    console.log('🌐 Testing browser resource utilization...');
    
    await generateKeypair(page);
    
    // Monitor resource usage
    const startMetrics = await page.evaluate(() => ({
      timestamp: Date.now(),
      memory: performance.memory ? {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize
      } : null
    }));
    
    // Perform intensive operations
    const operations = 30;
    for (let i = 0; i < operations; i++) {
      await signMessage(page, `Resource test ${i}: ${'X'.repeat(100)}`);
    }
    
    const endMetrics = await page.evaluate(() => ({
      timestamp: Date.now(),
      memory: performance.memory ? {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize
      } : null
    }));
    
    if (startMetrics.memory && endMetrics.memory) {
      const memoryIncrease = endMetrics.memory.used - startMetrics.memory.used;
      const timeElapsed = endMetrics.timestamp - startMetrics.timestamp;
      
      console.log(`📊 Resource Utilization:`);
      console.log(`  Time elapsed: ${timeElapsed}ms`);
      console.log(`  Memory increase: ${(memoryIncrease / 1024 / 1024).toFixed(2)}MB`);
      console.log(`  Operations per second: ${((operations * 1000) / timeElapsed).toFixed(2)}`);
      
      // Memory increase should be reasonable
      expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024); // Less than 50MB increase
    }
    
    console.log('✅ Resource utilization test complete');
  });

  test('error recovery under load conditions', async ({ page }) => {
    console.log('🚨 Testing error recovery under load...');
    
    await generateKeypair(page);
    
    const publicKey = await page.locator('#public-key').textContent();
    
    // Mix of valid and invalid requests
    const requests = [];
    
    // Add valid requests
    for (let i = 0; i < 10; i++) {
      const message = `Valid message ${i}`;
      const signature = await signMessage(page, message);
      const context = Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
      
      requests.push({
        type: 'valid',
        data: {
          sender: publicKey,
          context: context,
          body: message,
          proof: signature
        }
      });
    }
    
    // Add invalid requests
    for (let i = 0; i < 5; i++) {
      requests.push({
        type: 'invalid',
        data: {
          sender: publicKey,
          context: '48656c6c6f', // "Hello"
          body: `Invalid message ${i}`,
          proof: '0'.repeat(128) // Invalid signature
        }
      });
    }
    
    // Shuffle requests
    for (let i = requests.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [requests[i], requests[j]] = [requests[j], requests[i]];
    }
    
    // Send all requests
    const results = [];
    for (const request of requests) {
      const response = await page.request.post('http://localhost:8080/relay', {
        data: request.data,
        headers: {
          'Content-Type': 'application/json'
        }
      });
      
      results.push({
        type: request.type,
        status: response.status(),
        expected: request.type === 'valid' ? 200 : 401
      });
    }
    
    // Analyze results
    const correctResponses = results.filter(r => r.status === r.expected).length;
    const accuracy = (correctResponses / results.length) * 100;
    
    console.log(`📊 Error Recovery Results:`);
    console.log(`  Total requests: ${results.length}`);
    console.log(`  Correct responses: ${correctResponses}`);
    console.log(`  Accuracy: ${accuracy.toFixed(2)}%`);
    
    // Should handle all requests correctly
    expect(accuracy).toBe(100);
    
    console.log('✅ Error recovery test complete');
  });
});

test.describe('Scalability Testing', () => {
  
  test('multiple browser instances simulation', async ({ browser }) => {
    console.log('🌍 Testing multiple browser instances...');
    
    const instanceCount = 3;
    const contexts = [];
    const pages = [];
    
    try {
      // Create multiple browser contexts
      for (let i = 0; i < instanceCount; i++) {
        const context = await browser.newContext();
        const page = await context.newPage();
        
        contexts.push(context);
        pages.push(page);
        
        await page.goto('/demo.html');
        await waitForWasmReady(page);
      }
      
      // Perform operations in parallel across all instances
      const operations = pages.map(async (page, index) => {
        await generateKeypair(page);
        const result = await completeE2EWorkflow(page, `Multi-instance message ${index}`);
        return { instance: index, success: result.relayResponse.status === 200 };
      });
      
      const results = await Promise.all(operations);
      
      // Verify all instances succeeded
      const successful = results.filter(r => r.success).length;
      console.log(`📊 Multi-instance Results:`);
      console.log(`  Instances: ${instanceCount}`);
      console.log(`  Successful: ${successful}`);
      console.log(`  Success rate: ${((successful / instanceCount) * 100).toFixed(2)}%`);
      
      expect(successful).toBe(instanceCount);
      
    } finally {
      // Cleanup
      for (const context of contexts) {
        await context.close();
      }
    }
    
    console.log('✅ Multiple browser instances test complete');
  });
});