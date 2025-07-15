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

## Governance and Trust Model: Self-Hosted First

**Core Message: "You run the verifier. You control your data. You own your trust model."**

Proof-Messenger is designed with a **"self-hosted first"** governance model. This is a fundamental architectural decision that addresses enterprise security and data privacy requirements:

### 🏢 **Enterprise Control**
The **Relay Server**, which is the core verification engine, is a **stateless binary that you deploy within your own cloud or on-premise infrastructure**. This critical design choice provides several key advantages:

- **🔒 Data Privacy**: The context of your proofs (which may contain sensitive business data) **never leaves your network boundary**. You do not send any data to a third-party SaaS service.

- **⚙️ Full Operational Control**: You have complete control over the verification environment, allowing you to apply your own security policies, logging standards, compliance requirements, and monitoring systems.

- **🚫 No Third-Party Dependency**: Your system's uptime, availability, and security posture are **not dependent on our services** or any external SaaS provider.

- **📊 Audit and Compliance**: All verification logs and audit trails remain within your infrastructure, supporting regulatory compliance and internal security policies.

### 🚀 **Deployment Flexibility**
- **Primary Model**: Self-hosted deployment within your security perimeter
- **Container-Ready**: Distributed as lightweight Docker containers
- **Horizontally Scalable**: Stateless design supports load balancing and auto-scaling
- **Cloud-Agnostic**: Deploy on AWS, Azure, GCP, or on-premise infrastructure

### 🔮 **Future Options**
While a managed cloud offering may be available in the future for teams that prioritize convenience over control, **our primary and recommended deployment model for enterprise use is self-hosting**.

This approach ensures that enterprises maintain sovereignty over their most sensitive authorization decisions while benefiting from the security and non-repudiation guarantees of the Proof-Messenger protocol.

**📖 For detailed technical information, see our [Technical Architecture Document](./TECHNICAL_ARCHITECTURE.md)**

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