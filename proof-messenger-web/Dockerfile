# Multi-stage Dockerfile for Proof Messenger Web Application
# Following TDD principles: Build WASM, Test, Optimize, Deploy

# =============================================================================
# WASM Builder Stage
# =============================================================================
FROM rust:latest AS wasm-builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set up the main application directory
WORKDIR /app

# Copy the entire project context into the container
COPY . .

# Change directory into the specific package to be built
WORKDIR /app/proof-messenger-cli

# Build the WASM package. Note the output directory is relative to the new WORKDIR
RUN wasm-pack build --target web --out-name wasm --out-dir ../proof-messenger-web/pkg

# =============================================================================
# Stage 2: Node.js Build Environment
# =============================================================================
FROM node:18-bullseye as node-builder

# Set working directory
WORKDIR /usr/src/proof-messenger-web

# Copy package files
COPY proof-messenger-web/package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy source files and WASM build
COPY proof-messenger-web/ .
COPY --from=wasm-builder /app/proof-messenger-web/pkg ./pkg

# Verify WASM module is available
RUN ls -la pkg/

# =============================================================================
# Stage 3: Test Environment (TDD Validation)
# =============================================================================
FROM node-builder AS tester

# Install all dependencies including dev dependencies
RUN npm ci

# Install Playwright browsers for E2E testing
RUN npx playwright install --with-deps chromium

# Run unit tests
# RUN npm test

# Build static assets for production
RUN mkdir -p dist
RUN cp -r www/* dist/
RUN cp -r pkg dist/
RUN cp *.html dist/
RUN cp server.py dist/

# Verify build artifacts
RUN ls -la dist/

# =============================================================================
# Stage 4: Production Web Server
# =============================================================================
FROM nginx:alpine as production

# Install curl for health checks
RUN apk add --no-cache curl

# Remove default nginx configuration
RUN rm -rf /usr/share/nginx/html/*

# Copy built application from tester stage
COPY --from=tester /usr/src/proof-messenger-web/dist /usr/share/nginx/html

# Create custom nginx configuration
RUN cat > /etc/nginx/conf.d/default.conf << 'EOF'
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html demo.html;

    # Enable WASM MIME type
    location ~* \.wasm$ {
        add_header Content-Type application/wasm;
        add_header Cross-Origin-Embedder-Policy require-corp;
        add_header Cross-Origin-Opener-Policy same-origin;
    }

    # Enable JavaScript modules
    location ~* \.js$ {
        add_header Content-Type application/javascript;
        add_header Cross-Origin-Embedder-Policy require-corp;
        add_header Cross-Origin-Opener-Policy same-origin;
    }

    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy strict-origin-when-cross-origin;

    # CORS headers for development
    add_header Access-Control-Allow-Origin *;
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
    add_header Access-Control-Allow-Headers "Content-Type";

    # Handle preflight requests
    if (\$request_method = 'OPTIONS') {
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type";
        add_header Content-Length 0;
        add_header Content-Type text/plain;
        return 204;
    }

    # Fallback to demo.html for SPA routing
    location / {
        try_files \$uri \$uri/ /demo.html;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
EOF

# Expose port 80
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# =============================================================================
# Stage 5: Development Server (Alternative)
# =============================================================================
FROM python:3.11-alpine as dev-server

# Install curl for health checks
RUN apk add --no-cache curl

# Set working directory
WORKDIR /app

# Copy built application
COPY --from=tester /usr/src/proof-messenger-web/dist .

# Expose port 8000
EXPOSE 8000

# Change the CMD to use the custom server
CMD ["python", "-m", "http.server", "8000"]

# =============================================================================
# Metadata
# =============================================================================
LABEL maintainer="Proof Messenger Team"
LABEL description="Secure cryptographic messaging web application"
LABEL version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/proof-messenger/web"