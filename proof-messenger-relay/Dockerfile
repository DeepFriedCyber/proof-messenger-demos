# Multi-stage Dockerfile for Proof Messenger Relay Server
# Following TDD principles: Build, Test, Deploy

# =============================================================================
# Stage 1: Build Environment
# =============================================================================
FROM rust:1.75-bullseye as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/proof-messenger

# Copy all workspace members to satisfy workspace dependencies
COPY Cargo.toml ./
COPY proof-messenger-protocol ./proof-messenger-protocol
COPY proof-messenger-relay ./proof-messenger-relay
COPY proof-messenger-cli ./proof-messenger-cli
COPY proof-messenger-web ./proof-messenger-web

# Build only the relay server binary
RUN cargo build --release --bin proof-messenger-relay

# =============================================================================
# Stage 2: Test Environment (TDD Validation)
# =============================================================================
FROM builder as tester

# Run all tests to ensure quality
RUN cargo test --release --workspace

# Run integration tests specifically for relay
RUN cargo test --release --bin proof-messenger-relay

# Validate that the binary was built correctly
RUN ls -la target/release/proof-messenger-relay
RUN file target/release/proof-messenger-relay

# =============================================================================
# Stage 3: Runtime Environment (Minimal & Secure)
# =============================================================================
FROM debian:bullseye-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create a non-root user for security
RUN groupadd -r proofmessenger && useradd -r -g proofmessenger proofmessenger

# Create necessary directories
RUN mkdir -p /app/static && chown -R proofmessenger:proofmessenger /app

# Copy the compiled binary from builder stage
COPY --from=builder /usr/src/proof-messenger/target/release/proof-messenger-relay /app/proof-messenger-relay

# Copy static files if they exist
COPY --from=builder /usr/src/proof-messenger/proof-messenger-relay/static /app/static

# Set proper permissions
RUN chmod +x /app/proof-messenger-relay

# Switch to non-root user
USER proofmessenger

# Set working directory
WORKDIR /app

# Expose the port (default 8080)
EXPOSE 8080

# Health check to ensure service is running
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment variables with defaults
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:8080

# Set the command to run the server
CMD ["./proof-messenger-relay"]

# =============================================================================
# Metadata
# =============================================================================
LABEL maintainer="Proof Messenger Team"
LABEL description="Secure cryptographic message relay server"
LABEL version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/proof-messenger/relay"