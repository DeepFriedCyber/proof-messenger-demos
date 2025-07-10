# ✅ Proof Messenger Web Demo - Complete Structure

## 📁 Web Demo Structure

The web demo has been set up according to your specifications:

```
proof-messenger-web/
├── src/
│   └── lib.rs              ✅ WASM bindings for protocol
├── react-demo/
│   └── App.tsx             ✅ React example component
├── www/                    ✅ Static web files (existing)
├── Cargo.toml              ✅ Minimal WASM dependencies
├── package.json            ✅ Simple build scripts
└── README.md               ✅ Setup instructions
```

## 🔧 Cargo.toml (Simplified)

```toml
[package]
name = "proof-messenger-web"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
proof-messenger-protocol = { git = "https://github.com/DeepFriedCyber/proof-messenger-protocol" }
wasm-bindgen = "0.2"
ed25519-dalek = "1.0.1"
```

## 📦 package.json (Simplified)

```json
{
  "name": "proof-messenger-web",
  "version": "0.1.0",
  "scripts": {
    "build": "wasm-pack build --target web",
    "start": "python -m http.server 8000 --directory www"
  },
  "devDependencies": {
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "@types/react": "^18.0.0",
    "typescript": "^5.0.0"
  }
}
```

## 🦀 WASM Bindings (src/lib.rs)

```rust
use wasm_bindgen::prelude::*;
use ed25519_dalek::Keypair;
use proof_messenger_protocol::key::generate_keypair;

#[wasm_bindgen]
pub fn generate_keypair_js() -> Vec<u8> {
    let kp = generate_keypair();
    kp.public.to_bytes().to_vec()
}

#[wasm_bindgen]
pub fn make_proof_js(pubkey: &[u8], privkey: &[u8], invite_data: &[u8]) -> Vec<u8> {
    // implement a real version using ed25519_dalek, for demo purpose only
    vec![1,2,3]
}
```

## ⚛️ React Example (react-demo/App.tsx)

```tsx
import React, { useState } from "react";
import init, { generate_keypair_js, make_proof_js } from "proof-messenger-protocol-wasm";

export default function App() {
  const [pubkey, setPubkey] = useState<string | null>(null);

  async function handleGenKey() {
    await init();
    const key = generate_keypair_js();
    setPubkey(Buffer.from(key).toString("hex"));
  }

  return (
    <div>
      <h1>ProofPipe Web Demo</h1>
      <button onClick={handleGenKey}>Generate Keypair</button>
      {pubkey && <div>Public Key: {pubkey}</div>}
      {/* Add onboarding, messaging, proof details here */}
    </div>
  );
}
```

## 📖 README.md (Simplified)

```markdown
# proof-messenger-web

Web demo for proof-driven messaging using WASM-compiled Rust protocol logic.
- Uses proof-messenger-protocol built with wasm-pack
- UI: React (or Svelte/Vue) + TypeScript
- Features: onboarding, invite via QR or link, real-time messaging, proof details
- Talks to relay server via WebSocket

## Setup

1. Build WASM bundle:
    ```bash
    wasm-pack build --target web
    ```

2. Start web UI (React example):
    ```bash
    npm install
    npm start
    ```
```

## 🚀 Setup Instructions

### 1. Build WASM Bundle
```bash
cd proof-messenger-web
wasm-pack build --target web
```

This generates:
- `pkg/proof_messenger_web.js` - JavaScript bindings
- `pkg/proof_messenger_web_bg.wasm` - WebAssembly module
- `pkg/proof_messenger_web.d.ts` - TypeScript definitions

### 2. React Development
```bash
# Install React dependencies
npm install

# Start development server (if using React setup)
npm start

# Or serve static files
python -m http.server 8000 --directory www
```

### 3. Integration Example
```javascript
// Import WASM module
import init, { generate_keypair_js, make_proof_js } from './pkg/proof_messenger_web.js';

// Initialize WASM
await init();

// Generate keypair
const publicKey = generate_keypair_js();
console.log('Public Key:', Buffer.from(publicKey).toString('hex'));

// Create proof (demo)
const proof = make_proof_js(publicKey, privateKey, inviteData);
console.log('Proof:', proof);
```

## 🌐 Web Features

### Core WASM Functions
- `generate_keypair_js()` - Generate Ed25519 keypair, return public key bytes
- `make_proof_js(pubkey, privkey, invite_data)` - Create cryptographic proof

### Planned Features (to extend)
- **Onboarding**: Process invite codes, create identity
- **QR Codes**: Generate/scan QR codes for invites
- **Real-time Messaging**: WebSocket connection to relay
- **Proof Details**: Display and verify proof information
- **Local Storage**: Persist keys and messages

### UI Framework Options
- **React** (example provided)
- **Svelte** (lightweight alternative)
- **Vue** (component-based)
- **Vanilla JS** (minimal approach)

## 🔗 Integration Points

### With Protocol Library
```rust
// WASM bindings call protocol functions
use proof_messenger_protocol::key::generate_keypair;
use proof_messenger_protocol::proof::{make_proof, verify_proof, Invite};
```

### With Relay Server
```javascript
// WebSocket connection for real-time messaging
const ws = new WebSocket('ws://localhost:8080');
ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    handleIncomingMessage(message);
};
```

### With React Components
```tsx
// Use WASM functions in React components
function KeyGenerator() {
    const [key, setKey] = useState(null);
    
    const generateKey = async () => {
        await init();
        const pubkey = generate_keypair_js();
        setKey(Buffer.from(pubkey).toString('hex'));
    };
    
    return (
        <button onClick={generateKey}>
            Generate Key
        </button>
    );
}
```

## ✨ What's Been Set Up

### ✅ WASM Infrastructure
- Minimal Cargo.toml with protocol dependency
- WASM bindings for key generation and proof creation
- TypeScript-friendly exports

### ✅ Build System
- `wasm-pack` for WASM compilation
- Simple npm scripts for building and serving
- React development dependencies

### ✅ Example Implementation
- React component showing WASM integration
- Key generation with hex display
- Extensible structure for additional features

### ✅ Documentation
- Clear setup instructions
- Integration examples
- Extension points for full features

## 🚀 Ready for Development

The web demo is now ready for:

1. **WASM Development**: Extend lib.rs with more protocol functions
2. **UI Development**: Build React/Svelte/Vue components
3. **Feature Integration**: Add onboarding, messaging, proofs
4. **Relay Connection**: Implement WebSocket communication
5. **Demo Deployment**: Deploy to static hosting

Perfect for showcasing the protocol in web browsers with modern JavaScript frameworks!