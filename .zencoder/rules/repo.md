---
description: Repository Information Overview
alwaysApply: true
---

# Repository Information Overview

## Repository Summary
Proof Messenger is a secure messaging system with cryptographic proofs, designed with a "self-hosted first" governance model. The system is split into multiple focused components for better maintainability and deployment, allowing enterprises to maintain control over their data and verification processes.

## Strategic Priorities

### Cryptographic Priorities
- **Current State**: Ed25519 signatures are implemented and working well with excellent performance (~64 bytes with fast verification)
- **Roadmap**: Implementing hybrid post-quantum cryptography with both Ed25519 and Dilithium signatures
- **Implementation Phases**:
  - Phase 1 (2-4 weeks): Create hybrid cryptography module structure
  - Phase 2 (1-2 months): Implement feature flags and backward compatibility
  - Phase 3 (3-6 months): Full hybrid implementation with opt-in capability

### Performance Targets
- **Current State**: 67.9 RPS with bottlenecks in database queries and message processing
- **Optimization Goals**: Achieve 100+ RPS through connection pooling, message batching, and async processing
- **Implementation Timeline**: 1-4 weeks for immediate and medium-term optimizations

### IAM Prioritization
- **Current State**: Okta integration completed and tested
- **Roadmap**: Add Auth0 integration (2-3 weeks) and custom JWT issuers (3-4 weeks)
- **Implementation Strategy**: Leverage existing JWT validation architecture with pluggable IAM connectors

### Deployment Focus
- **Current State**:
  - Self-hosted Docker deployment is functional
  - Kubernetes-ready with proper containerization
  - No managed service offering yet
- **Hybrid Deployment Strategy**:
  - Immediate (1-2 weeks): Enhanced Helm chart for self-hosted deployments with configurable modes
  - Medium-term (2-3 months): Implement tenant isolation, automated scaling policies, and operational tooling for both models
- **Implementation Approach**: 
  - Maintain self-hosted as primary deployment model
  - Develop managed service option with strict tenant isolation

## Repository Structure
The repository is organized as a Rust workspace with four main projects:

- **proof-messenger-protocol**: Core Rust library containing all cryptographic, proof, and protocol logic
- **proof-messenger-cli**: Command-line interface for technical users, demos, and automation
- **proof-messenger-web**: Web application with WASM-compiled protocol library and modern frontend
- **proof-messenger-relay**: Minimal relay server for message routing with stateless design

Additional directories include:
- **demos**: Example applications showcasing different use cases (fintech, healthcare, government)
- **docs**: Documentation files including proof revocation flows
- **performance-tests**: Load testing and performance benchmarking tools using Locust
- **scripts**: Utility scripts for development and deployment
- **.github**: CI/CD workflows
- **shared files**: Common configuration and test utilities

## Projects

### proof-messenger-protocol
**Configuration File**: proof-messenger-protocol/Cargo.toml

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
cargo test --package proof-messenger-protocol
```

### proof-messenger-relay
**Configuration File**: proof-messenger-relay/Cargo.toml

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
- sqlx: 0.7 (Database access with SQLite)
- jsonwebtoken: 9.2 (JWT validation)
- tower/tower-http: 0.4/0.5 (HTTP middleware)
- prometheus-client: 0.22 (Metrics)
- aes-gcm: 0.10 (Secure logging encryption)

#### Build & Installation
```bash
cd proof-messenger-relay && cargo build --release
```

#### Docker
**Dockerfile**: proof-messenger-relay/Dockerfile
**Image**: Multi-stage build (builder, tester, runtime)
**Configuration**: Runs as non-root user, uses SQLite for persistence, exposes port 8080

#### Testing
**Framework**: Rust's built-in testing + hyper, proptest
**Test Location**: tests/ directory with integration and Docker tests
**Run Command**:
```bash
cargo test --package proof-messenger-relay
```

### proof-messenger-web
**Configuration File**: proof-messenger-web/Cargo.toml, package.json

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
- js-sys: 0.3 (JavaScript interop)
- getrandom: 0.2 (Random number generation)

**Development Dependencies**:
- vitest: 1.0.0 (Testing)
- jest: 29.7.0 (Testing)
- playwright: 1.40.0 (E2E testing)

#### Build & Installation
```bash
cd proof-messenger-web && wasm-pack build --target web
```

#### Docker
**Dockerfile**: proof-messenger-web/Dockerfile
**Image**: Multi-stage build (wasm-builder, node-builder, tester, production)
**Configuration**: Nginx for production, Python for development server

#### Testing
**Framework**: Vitest, Jest, Playwright
**Test Location**: test/ directory, tests/e2e directory
**Run Command**:
```bash
cd proof-messenger-web && npm test
cd proof-messenger-web && npm run test:e2e
```

### proof-messenger-cli
**Configuration File**: proof-messenger-cli/Cargo.toml

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
cargo test --package proof-messenger-cli
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

## Containerization & Deployment

The project includes comprehensive Docker support for all components:

**Docker Compose**: docker-compose.yml configures:
- relay-server: Backend API service
- web-app: Frontend web application
- web-dev: Development server (optional)
- e2e-tests: End-to-end testing service
- prometheus/grafana: Monitoring stack
- traefik: Load balancer/reverse proxy (optional)

**Networks**: Dedicated bridge network (proof-messenger-network)
**Volumes**: Persistent storage for database, test results, and monitoring data
**Deployment Profiles**: Production, development, and testing configurations

**Docker Run Command**:
```bash
docker-compose up -d
```

### Kubernetes Deployment

The project supports Kubernetes deployment through Helm charts:

**Helm Chart Configuration**: charts/proof-messenger/values.yaml
```yaml
deployment:
  mode: "self-hosted"  # or "managed-service"
  
# Self-hosted specific
persistence:
  storageClass: "fast-ssd"
  size: 100Gi
  
# Managed service specific
multiTenancy:
  enabled: false
  tenantIsolation: "strict"
```

**Deployment Models**:
- **Self-Hosted**: Primary deployment model with full customer control
- **Managed Service**: Future option with multi-tenancy support (in development)

**Kubernetes Resources**:
- StatefulSets for relay servers with persistent storage
- Deployments for web applications
- Services and Ingress for network routing
- ConfigMaps and Secrets for configuration management

## Monitoring & Observability
The repository includes a complete monitoring stack:
- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **Custom Metrics**: Instrumentation in the relay server
- **Health Checks**: Implemented across all containerized services

## Prerequisites
- Rust 1.70+ 
- Node.js 18+ (for web frontend)
- wasm-pack (for WASM compilation)
- Docker & Docker Compose (for containerized deployment)
- Python 3.11+ (for performance testing)