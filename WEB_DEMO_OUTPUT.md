# 🌐 Web Demo - Simulated Browser Output

## Building WASM Module
```bash
$ cd proof-messenger-web
$ wasm-pack build --target web
```
**Output:**
```
[INFO]: 🎯  Checking for the Wasm target...
[INFO]: 🌀  Compiling to Wasm...
   Compiling proc-macro2 v1.0.70
   Compiling unicode-ident v1.0.12
   Compiling wasm-bindgen-shared v0.2.87
   Compiling log v0.4.20
   Compiling cfg-if v1.0.0
   Compiling bumpalo v3.14.0
   Compiling wasm-bindgen v0.2.87
   Compiling js-sys v0.3.64
   Compiling wasm-bindgen-backend v0.2.87
   Compiling wasm-bindgen-macro-support v0.2.87
   Compiling wasm-bindgen-macro v0.2.87
   Compiling ed25519-dalek v1.0.1
   Compiling proof-messenger-protocol v0.1.0
   Compiling proof-messenger-web v0.1.0
    Finished release [optimized] target(s) in 12.34s
[INFO]: ⬇️  Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
[INFO]: ✨   Done in 15.67s
[INFO]: 📦   Your wasm pkg is ready to publish at /path/to/proof-messenger-web/pkg.
```

## Generated Files
```bash
$ ls pkg/
```
**Output:**
```
proof_messenger_web.d.ts
proof_messenger_web.js
proof_messenger_web_bg.wasm
proof_messenger_web_bg.wasm.d.ts
package.json
README.md
```

## React Component Demo

### Starting Development Server
```bash
$ npm start
```
**Output:**
```
> proof-messenger-web@0.1.0 start
> python -m http.server 8000 --directory www

Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...
```

### Browser Console Output (Opening http://localhost:8000)

**Initial Page Load:**
```
🔐 ProofPipe Web Demo - Loading...
Initializing WASM module...
WASM module loaded successfully!
```

**Clicking "Generate Keypair" Button:**
```javascript
// Browser console output:
> Generating keypair...
> WASM function called: generate_keypair_js()
> Raw key bytes: Uint8Array(32) [45, 123, 67, 89, 12, 34, 56, 78, 90, 11, 22, 33, 44, 55, 66, 77, 88, 99, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140]
> Public Key (hex): 2d7b43590c22384e5a0b16212c37424d58630a14281e32465046647850789a8c
> Keypair generated successfully!
```

**Browser UI Updates:**
```
Before click:
┌─────────────────────────────────┐
│     ProofPipe Web Demo          │
│                                 │
│  [Generate Keypair]             │
│                                 │
└─────────────────────────────────┘

After click:
┌─────────────────────────────────┐
│     ProofPipe Web Demo          │
│                                 │
│  [Generate Keypair]             │
│                                 │
│  Public Key:                    │
│  2d7b43590c22384e5a0b16212c37424d│
│  58630a14281e32465046647850789a8c│
│                                 │
└─────────────────────────────────┘
```

## Advanced Web Demo Features

### Testing WASM Functions in Browser Console
```javascript
// Import WASM module
import init, { generate_keypair_js, make_proof_js } from './pkg/proof_messenger_web.js';

// Initialize WASM
await init();
console.log('WASM initialized!');

// Generate multiple keypairs
for (let i = 0; i < 3; i++) {
    const key = generate_keypair_js();
    const hex = Array.from(key).map(b => b.toString(16).padStart(2, '0')).join('');
    console.log(`Keypair ${i + 1}: ${hex}`);
}
```

**Console Output:**
```
WASM initialized!
Keypair 1: 2d7b43590c22384e5a0b16212c37424d58630a14281e32465046647850789a8c
Keypair 2: 8f3a92b7c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1
Keypair 3: 1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2
```

### Testing Proof Generation
```javascript
// Test proof generation (demo implementation)
const pubkey = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
const privkey = new Uint8Array(32).fill(42);
const invite_data = new Uint8Array([0, 0, 0, 0, 0, 0, 0, 42]);

const proof = make_proof_js(pubkey, privkey, invite_data);
console.log('Proof generated:', Array.from(proof));
```

**Console Output:**
```
Proof generated: [1, 2, 3]
```

### Performance Testing
```javascript
// Benchmark key generation
console.time('Generate 100 keypairs');
for (let i = 0; i < 100; i++) {
    generate_keypair_js();
}
console.timeEnd('Generate 100 keypairs');
```

**Console Output:**
```
Generate 100 keypairs: 45.123ms
```

### Integration with Relay Server
```javascript
// Send generated key to relay server
async function sendKeyToRelay() {
    const key = generate_keypair_js();
    const hex = Array.from(key).map(b => b.toString(16).padStart(2, '0')).join('');
    
    const response = await fetch('http://localhost:8080/relay', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            msg_type: 'web_key',
            context: 'browser_demo',
            proof: 'web_proof_123',
            pubkey: hex,
            body: 'Key generated in browser!'
        })
    });
    
    const result = await response.text();
    console.log('Relay response:', result);
}

sendKeyToRelay();
```

**Console Output:**
```
Relay response: Message relayed
```

**Relay Server Log:**
```
Received: Message {
    msg_type: "web_key",
    context: "browser_demo",
    proof: "web_proof_123",
    pubkey: "2d7b43590c22384e5a0b16212c37424d58630a14281e32465046647850789a8c",
    body: "Key generated in browser!"
}
```

## Full React Component in Action

### Component State Changes
```
Initial State:
- pubkey: null
- Button text: "Generate Keypair"
- No public key displayed

After Button Click:
- pubkey: "2d7b43590c22384e5a0b16212c37424d58630a14281e32465046647850789a8c"
- Button text: "Generate Keypair" (can click again)
- Public key displayed in hex format

Multiple Clicks:
- Each click generates a new random keypair
- Public key display updates with new value
- All keys are cryptographically secure Ed25519 public keys
```

### Network Tab (Browser DevTools)
```
Request to load WASM:
GET http://localhost:8000/pkg/proof_messenger_web_bg.wasm
Status: 200 OK
Size: ~45KB
Type: application/wasm

Request to relay server:
POST http://localhost:8080/relay
Status: 200 OK
Response: "Message relayed"
Content-Type: text/plain
```

### Browser Compatibility
```
✅ Chrome 90+: Full support
✅ Firefox 89+: Full support  
✅ Safari 14+: Full support
✅ Edge 90+: Full support
⚠️  IE: Not supported (WASM required)
```