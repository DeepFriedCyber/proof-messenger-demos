# End-to-End (E2E) Testing Suite

## 🎯 Overview

This E2E testing suite provides comprehensive validation of the Proof Messenger Protocol from a user's perspective. The tests simulate real user interactions in actual browsers, validating the complete workflow from web app to relay server.

## 🏗️ Architecture

### Test Structure
```
tests/e2e/
├── messaging.spec.js          # Core user journey tests
├── react-integration.spec.js  # React component integration tests
├── performance.spec.js        # Performance and load testing
├── helpers/
│   └── test-utils.js          # Common utilities and helpers
├── global-setup.js           # Global test setup
├── global-teardown.js        # Global test cleanup
└── README.md                 # This documentation
```

### Test Categories

#### 1. **Core User Journey Tests** (`messaging.spec.js`)
- Complete E2E workflow validation
- Key generation → Message signing → Relay server verification
- Error handling and edge cases
- Cross-browser compatibility
- Security property validation

#### 2. **React Integration Tests** (`react-integration.spec.js`)
- Zustand store integration
- Component state management
- React-specific error handling
- Component isolation testing

#### 3. **Performance Tests** (`performance.spec.js`)
- Key generation benchmarks
- Message signing performance
- Concurrent operations stress testing
- Memory usage monitoring
- Load testing with multiple messages

## 🚀 Getting Started

### Prerequisites
- Node.js 18+ installed
- Rust toolchain with `wasm-pack`
- Python 3.x (for local HTTP server)

### Installation
```bash
cd proof-messenger-web
npm install
```

### Running Tests

#### Run All E2E Tests
```bash
npm run test:e2e
```

#### Run with UI (Interactive Mode)
```bash
npm run test:e2e:ui
```

#### Run in Debug Mode
```bash
npm run test:e2e:debug
```

#### Run with Browser Visible
```bash
npm run test:e2e:headed
```

#### Run Specific Test File
```bash
npx playwright test messaging.spec.js
```

#### Run Specific Test
```bash
npx playwright test -g "complete user journey"
```

## 🧪 Test Scenarios

### Core Functionality Tests

#### ✅ Complete User Journey
1. **WASM Loading**: Verify WebAssembly module loads correctly
2. **Key Generation**: Generate Ed25519 keypair securely in WASM
3. **Message Signing**: Sign message with private key (never exposed)
4. **Relay Communication**: Send signed message to relay server
5. **Verification**: Relay server verifies cryptographic proof
6. **Success Confirmation**: UI shows successful verification

#### ✅ Error Handling
- Invalid signature rejection
- Malformed request handling
- WASM loading failure simulation
- Network error recovery

#### ✅ Security Validation
- Private key encapsulation verification
- WASM boundary protection testing
- Component tree safety validation
- Memory inspection protection

### Performance Tests

#### ⚡ Benchmarks
- **Key Generation**: < 3 seconds average, < 5 seconds max
- **Message Signing**: < 2 seconds regardless of message size
- **E2E Workflow**: < 5 seconds average per complete cycle

#### 📊 Load Testing
- Multiple concurrent operations
- Extended operation memory usage
- Relay server load handling
- Browser resource utilization

### Cross-Browser Testing

#### 🌐 Supported Browsers
- **Chromium**: Full WebAssembly and Crypto API support
- **Firefox**: Full WebAssembly and Crypto API support  
- **WebKit/Safari**: Full WebAssembly and Crypto API support
- **Mobile Chrome**: Touch interface and mobile performance
- **Mobile Safari**: iOS-specific behavior validation

## 🔧 Configuration

### Playwright Configuration (`playwright.config.js`)
- **Parallel Execution**: Tests run in parallel for speed
- **Retry Logic**: Automatic retry on CI environments
- **Multiple Reporters**: HTML, JSON, and JUnit output
- **Screenshot/Video**: Captured on test failures
- **Timeouts**: Configured for crypto operations

### Environment Setup
- **Web Server**: Python HTTP server on port 8000
- **Relay Server**: Rust server on port 8080
- **WASM Build**: Automatic build during global setup

## 📋 Test Data and Utilities

### Test Utilities (`helpers/test-utils.js`)
- **waitForWasmReady()**: Wait for WASM module initialization
- **generateKeypair()**: Generate keypair and return public key
- **signMessage()**: Sign message and return signature
- **completeE2EWorkflow()**: Full workflow automation
- **PerformanceTracker**: Performance measurement utilities

### Test Data Sets
- **Message Variations**: Unicode, long messages, special characters
- **Invalid Data**: Malformed hex, wrong lengths, type errors
- **Performance Data**: Various message sizes for benchmarking

## 🐛 Debugging

### Debug Mode
```bash
npm run test:e2e:debug
```
- Runs tests with Playwright Inspector
- Step through tests interactively
- Inspect page state at any point

### Console Logging
Tests automatically capture and log:
- Browser console messages
- Network requests/responses
- Performance metrics
- Error details

### Screenshots and Videos
- Automatic screenshot on test failure
- Video recording for failed tests
- Trace files for detailed debugging

## 📊 Reporting

### HTML Report
```bash
npx playwright show-report
```
- Interactive test results
- Screenshots and videos
- Performance metrics
- Error details

### CI/CD Integration
- **JSON Output**: Machine-readable results
- **JUnit XML**: Integration with CI systems
- **Exit Codes**: Proper failure indication

## 🔒 Security Testing

### Private Key Protection
- Verify private keys never appear in JavaScript
- Test WASM boundary encapsulation
- Validate component tree isolation
- Check serialization safety

### Cryptographic Validation
- Ed25519 signature verification
- Context-bound proof validation
- Cross-keypair verification failure
- Tampered data rejection

## 🚀 Performance Expectations

### Benchmarks
| Operation | Target | Maximum |
|-----------|--------|---------|
| Key Generation | < 3s avg | < 5s max |
| Message Signing | < 2s | < 2s |
| E2E Workflow | < 5s avg | < 10s max |
| Memory Usage | < 50MB increase | < 100MB |

### Load Testing
- **Concurrent Operations**: 10+ simultaneous signings
- **Extended Usage**: 50+ operations without memory leaks
- **Relay Load**: 20+ messages processed successfully
- **Multi-Instance**: 3+ browser instances simultaneously

## 🔄 Continuous Integration

### GitHub Actions Integration
```yaml
- name: Run E2E Tests
  run: |
    cd proof-messenger-web
    npm install
    npm run test:e2e
```

### Test Artifacts
- Test results uploaded as CI artifacts
- Screenshots/videos for failed tests
- Performance metrics tracking
- Coverage reports

## 📈 Metrics and Monitoring

### Success Criteria
- **Functionality**: 100% core user journey success
- **Performance**: All benchmarks within targets
- **Compatibility**: All browsers passing
- **Security**: All security properties validated

### Monitoring
- Test execution time trends
- Performance regression detection
- Browser compatibility tracking
- Error rate monitoring

## 🛠️ Maintenance

### Adding New Tests
1. Create test file in appropriate category
2. Use helper utilities for common operations
3. Follow TDD principles (test first)
4. Add performance assertions
5. Include error scenarios

### Updating Test Data
- Modify `TestData` object in `test-utils.js`
- Add new message variations
- Update performance thresholds
- Extend browser compatibility matrix

### Troubleshooting Common Issues
- **WASM Loading Timeout**: Increase timeout in config
- **Relay Server Connection**: Check server startup
- **Performance Variance**: Run multiple iterations
- **Browser Compatibility**: Update browser versions

## 📚 Best Practices

### Test Design
- **Isolation**: Each test is independent
- **Deterministic**: Tests produce consistent results
- **Fast**: Optimized for quick feedback
- **Comprehensive**: Cover all user scenarios

### Error Handling
- **Graceful Degradation**: Test failure scenarios
- **Clear Messages**: Descriptive error reporting
- **Recovery Testing**: Validate error recovery
- **Edge Cases**: Test boundary conditions

### Performance
- **Realistic Load**: Test with realistic data sizes
- **Resource Monitoring**: Track memory and CPU usage
- **Scalability**: Test multiple concurrent users
- **Regression Prevention**: Benchmark tracking

---

## 🎉 Success Metrics

This E2E testing suite ensures:
- ✅ **100% User Journey Coverage**: Complete workflow validation
- ✅ **Cross-Browser Compatibility**: All major browsers supported
- ✅ **Performance Validation**: All operations within targets
- ✅ **Security Assurance**: Private key protection verified
- ✅ **Error Resilience**: Graceful handling of all error conditions
- ✅ **Production Readiness**: Real-world scenario validation

The Proof Messenger Protocol is thoroughly tested and ready for production deployment! 🚀