---
description: Repository Information Overview
alwaysApply: true
---

# Repository Information Overview

## Repository Summary
Proof Messenger is a secure messaging system with cryptographic proofs, organized as a Rust workspace with multiple interconnected projects. The system is designed with a "self-hosted first" governance model, allowing enterprises to maintain control over their data and verification processes.

## Repository Structure
- **proof-messenger-protocol**: Core Rust library containing cryptographic, proof, and protocol logic
- **proof-messenger-cli**: Command-line interface for technical users, demos, and automation
- **proof-messenger-web**: Web application with WASM-compiled protocol library and modern frontend
- **proof-messenger-relay**: Minimal relay server for message routing with stateless design
- **performance-tests**: Load testing and performance benchmarking tools
- **demos**: Example implementations for various use cases

## Projects

### proof-messenger-protocol
**Configuration File**: Cargo.toml

#### Language & Runtime
**Language**: Rust
**Version**: Edition 2021, requires Rust 1.70+
**Build System**: Cargo
**Package Manager**: Cargo

#### Dependencies
**Main Dependencies**:
- ed25519-dalek: 1.0.1 (Cryptography)
- rand: 0.7 (Random number generation)
- zeroize: 1.7 (Secure memory handling)
- thiserror: 1.0 (Error handling)
- serde/serde_json: 1.0 (Serialization)
- chrono: 0.4 (Date/time handling)

#### Build & Installation
```bash
cd proof-messenger-protocol && cargo build --release
```

#### Testing
**Framework**: Rust's built-in testing + proptest
**Test Location**: tests/ directory
**Run Command**:
```bash
cargo test
```

### proof-messenger-cli
**Configuration File**: Cargo.toml

#### Language & Runtime
**Language**: Rust
**Version**: Edition 2021, requires Rust 1.70+
**Build System**: Cargo
**Package Manager**: Cargo

#### Dependencies
**Main Dependencies**:
- proof-messenger-protocol (Internal dependency)
- clap: 4.0.32 (Command-line argument parsing)
- serde/serde_json: 1.0 (Serialization)
- hex: 0.4 (Hex encoding/decoding)

#### Build & Installation
```bash
cd proof-messenger-cli && cargo build --release
```

#### Testing
**Framework**: assert_cmd, predicates
**Test Location**: tests/ directory
**Run Command**:
```bash
cargo test
```

### proof-messenger-relay
**Configuration File**: Cargo.toml

#### Language & Runtime
**Language**: Rust
**Version**: Edition 2021, requires Rust 1.70+
**Build System**: Cargo
**Package Manager**: Cargo

#### Dependencies
**Main Dependencies**:
- proof-messenger-protocol (Internal dependency)
- axum: 0.7 (Web framework)
- tokio: 1.0 (Async runtime)
- sqlx: 0.7 (Database access)
- jsonwebtoken: 9.2 (JWT validation)
- tower/tower-http: 0.4/0.5 (HTTP middleware)

#### Build & Installation
```bash
cd proof-messenger-relay && cargo build --release
```

#### Docker
**Dockerfile**: proof-messenger-relay/Dockerfile
**Image**: Multi-stage build (builder, tester, runtime)
**Configuration**: Runs on port 8080, uses Debian bullseye-slim base

#### Testing
**Framework**: Rust's built-in testing + hyper, proptest
**Test Location**: tests/ directory
**Run Command**:
```bash
cargo test
```

### proof-messenger-web
**Configuration File**: Cargo.toml, package.json

#### Language & Runtime
**Language**: Rust (WASM), JavaScript
**Version**: Rust Edition 2021, Node.js 18+
**Build System**: wasm-pack, npm
**Package Manager**: Cargo, npm

#### Dependencies
**Main Dependencies**:
- proof-messenger-protocol (Internal dependency)
- wasm-bindgen: 0.2 (WASM bindings)
- web-sys: 0.3 (Web APIs)
- crypto-js: 4.2.0 (JavaScript crypto)
- zustand: 4.4.0 (State management)

**Development Dependencies**:
- vitest: 1.0.0 (Testing)
- jest: 29.7.0 (Testing)
- playwright: 1.40.0 (E2E testing)

#### Build & Installation
```bash
cd proof-messenger-web && wasm-pack build --target web
npm run build
```

#### Docker
**Dockerfile**: proof-messenger-web/Dockerfile
**Image**: Multi-stage build (wasm-builder, node-builder, tester, production)
**Configuration**: Nginx-based production image, Python-based dev server

#### Testing
**Framework**: Vitest, Jest, Playwright
**Test Location**: test/ directory, tests/ directory
**Run Command**:
```bash
npm test
npm run test:e2e
```

### performance-tests
**Configuration File**: requirements.txt

#### Language & Runtime
**Language**: Python
**Version**: Python 3.11+
**Package Manager**: pip

#### Dependencies
**Main Dependencies**:
- locust: 2.17.0 (Load testing)
- flask: 2.3.3 (Mock server)
- pandas: 2.1.1 (Data analysis)
- matplotlib: 3.7.2 (Visualization)
- requests: 2.31.0 (HTTP client)

#### Usage & Operations
```bash
cd performance-tests
pip install -r requirements.txt
python -m locust -f locustfile.py
```