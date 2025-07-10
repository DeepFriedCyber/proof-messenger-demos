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