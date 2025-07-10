import { defineConfig } from 'vitest/config';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./test/setup.js'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'pkg/',
        'test/',
        '*.config.js'
      ]
    },
    testTimeout: 30000, // 30 seconds for WASM loading
    hookTimeout: 30000
  },
  resolve: {
    alias: {
      '@': new URL('./src', import.meta.url).pathname,
      '@pkg': new URL('./pkg', import.meta.url).pathname
    }
  },
  define: {
    global: 'globalThis',
  },
  optimizeDeps: {
    exclude: ['proof_messenger_web']
  }
});