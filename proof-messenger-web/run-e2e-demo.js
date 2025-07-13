#!/usr/bin/env node

/**
 * E2E Test Demo Runner
 * 
 * This script demonstrates how to run E2E tests for the Proof Messenger Protocol.
 * It runs a subset of tests to showcase the complete user journey validation.
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('🚀 Starting Proof Messenger E2E Test Demo...\n');

// Configuration
const config = {
  // Run only the core user journey test for demo
  testPattern: 'messaging.spec.js -g "complete user journey"',
  // Use only Chromium for demo (faster)
  project: 'chromium',
  // Show output in real-time
  headed: false, // Set to true to see browser
  timeout: 60000
};

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    console.log(`📋 Running: ${command} ${args.join(' ')}`);
    
    const child = spawn(command, args, {
      stdio: 'inherit',
      shell: true,
      ...options
    });
    
    child.on('close', (code) => {
      if (code === 0) {
        resolve(code);
      } else {
        reject(new Error(`Command failed with exit code ${code}`));
      }
    });
    
    child.on('error', (error) => {
      reject(error);
    });
  });
}

async function main() {
  try {
    console.log('🔧 Step 1: Building WASM module...');
    await runCommand('wasm-pack', [
      'build',
      '--target', 'web',
      '--out-dir', 'pkg',
      '--dev'
    ]);
    console.log('✅ WASM module built successfully\n');
    
    console.log('🧪 Step 2: Running E2E test demo...');
    const playwrightArgs = [
      'test',
      config.testPattern,
      '--project', config.project,
      '--timeout', config.timeout.toString()
    ];
    
    if (config.headed) {
      playwrightArgs.push('--headed');
    }
    
    await runCommand('npx', ['playwright', ...playwrightArgs]);
    
    console.log('\n🎉 E2E Test Demo completed successfully!');
    console.log('\n📊 What was tested:');
    console.log('  ✅ WASM module loading and initialization');
    console.log('  ✅ Ed25519 keypair generation in WASM');
    console.log('  ✅ Message signing with private key encapsulation');
    console.log('  ✅ Relay server communication and verification');
    console.log('  ✅ Complete end-to-end cryptographic workflow');
    
    console.log('\n🚀 Next steps:');
    console.log('  • Run full test suite: npm run test:e2e');
    console.log('  • Run with UI: npm run test:e2e:ui');
    console.log('  • Run performance tests: npx playwright test performance.spec.js');
    console.log('  • View test report: npx playwright show-report');
    
  } catch (error) {
    console.error('\n❌ E2E Test Demo failed:', error.message);
    console.log('\n🔧 Troubleshooting:');
    console.log('  • Ensure relay server is running: cd ../proof-messenger-relay && cargo run');
    console.log('  • Check WASM build: wasm-pack build --target web --out-dir pkg --dev');
    console.log('  • Verify demo.html exists and is accessible');
    process.exit(1);
  }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n⏹️  E2E Test Demo interrupted by user');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\n⏹️  E2E Test Demo terminated');
  process.exit(0);
});

main();