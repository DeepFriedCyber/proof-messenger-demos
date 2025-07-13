/**
 * Global Setup for E2E Tests
 * 
 * This file handles the global setup required before running E2E tests.
 * It ensures that all necessary services are running and properly configured.
 */

import { chromium } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function globalSetup() {
  console.log('🚀 Starting E2E Test Global Setup...');
  
  try {
    // Build the WASM module if needed
    console.log('📦 Building WASM module...');
    await execAsync('wasm-pack build --target web --out-dir pkg --dev', {
      cwd: process.cwd()
    });
    console.log('✅ WASM module built successfully');
    
    // Build the relay server if needed
    console.log('🔧 Building relay server...');
    await execAsync('cargo build --release', {
      cwd: '../proof-messenger-relay'
    });
    console.log('✅ Relay server built successfully');
    
    // Verify that the demo.html file exists
    const fs = await import('fs');
    if (!fs.existsSync('./demo.html')) {
      throw new Error('demo.html not found - required for E2E tests');
    }
    console.log('✅ Demo HTML file verified');
    
    console.log('🎉 Global setup completed successfully');
    
  } catch (error) {
    console.error('❌ Global setup failed:', error);
    throw error;
  }
}

export default globalSetup;