# Proof Messenger

A secure messaging system with cryptographic proofs, split into multiple focused repositories for better maintainability and deployment.

## Architecture

This workspace contains four interconnected projects:

### 🔧 [proof-messenger-protocol](./proof-messenger-protocol/)
Pure Rust library containing all cryptographic, proof, and protocol logic. This is the core that all other applications depend on.

### 💻 [proof-messenger-cli](./proof-messenger-cli/)
Command-line interface for technical users, demos, and automation. Perfect for security reviews and scripting.

### 🌐 [proof-messenger-web](./proof-messenger-web/)
Web application with WASM-compiled protocol library and modern frontend. Provides an intuitive UI for end users.

### 🚀 [proof-messenger-relay](./proof-messenger-relay/)
Minimal relay server for message routing. Stateless design with optional logging for demonstrations.

## Quick Start

### Prerequisites
- Rust 1.70+ 
- Node.js 18+ (for web frontend)
- wasm-pack (for WASM compilation)

### Development Setup

1. **Open the workspace in VS Code:**
   ```bash
   code proof-messenger.code-workspace
   ```

2. **Build all projects:**
   ```bash
   # Protocol library (core)
   cd proof-messenger-protocol && cargo build
   
   # CLI application
   cd proof-messenger-cli && cargo build
   
   # Web application (WASM)
   cd proof-messenger-web && wasm-pack build --target web
   
   # Relay server
   cd proof-messenger-relay && cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test --workspace
   ```

### Usage Examples

**CLI:**
```bash
# Initialize identity
./target/debug/proof-messenger-cli init --name "Alice"

# Generate invite
./target/debug/proof-messenger-cli invite --message "Join our secure chat"

# Send message
./target/debug/proof-messenger-cli send <recipient-key> "Hello, world!"
```

**Relay Server:**
```bash
./target/debug/proof-messenger-relay --port 8080
```

**Web Application:**
Open `proof-messenger-web/www/index.html` in a browser after building the WASM module.

## Development Workflow

This multi-root workspace is configured for optimal development experience:

- **Rust Analyzer** works across all projects with proper dependency resolution
- **Integrated tasks** for building, testing, and running each component
- **Debug configurations** for CLI and server applications
- **Unified settings** for consistent code formatting and linting

## Contributing

Each project has its own README with specific setup instructions and contribution guidelines. Please refer to the individual project documentation for detailed information.

## License

MIT OR Apache-2.0