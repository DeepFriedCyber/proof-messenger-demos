#!/bin/bash
# Enhanced docker-build-relay.sh - Relay Server Only
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_RELAY_PORT=8080
HEALTH_CHECK_RETRIES=5
HEALTH_CHECK_INTERVAL=2

# Logging function
log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Cleanup function
cleanup() {
    log "Cleaning up test containers..."
    docker stop test-relay-server 2>/dev/null || true
    docker rm test-relay-server 2>/dev/null || true
}

# Set up cleanup trap
trap cleanup EXIT

# Setup environment for Windows/WSL
setup_environment() {
    # Try multiple possible cargo paths
    CARGO_PATHS=(
        "$HOME/.cargo/bin"
        "/c/Users/$USER/.cargo/bin"
        "/mnt/c/Users/$USER/.cargo/bin"
    )
    
    # Add cargo paths to PATH
    for cargo_path in "${CARGO_PATHS[@]}"; do
        if [ -d "$cargo_path" ]; then
            export PATH="$PATH:$cargo_path"
        fi
    done
    
    # Try to find cargo executable directly
    if ! command -v cargo &> /dev/null; then
        # Try PowerShell to get cargo path
        CARGO_FROM_PS=$(powershell.exe -Command "Get-Command cargo -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source" 2>/dev/null | tr -d '\r')
        if [ -n "$CARGO_FROM_PS" ] && [ -f "$CARGO_FROM_PS" ]; then
            CARGO_DIR=$(dirname "$CARGO_FROM_PS")
            export PATH="$PATH:$CARGO_DIR"
        fi
    fi
    
    # Final verification
    if ! command -v cargo &> /dev/null; then
        warning "Cargo not found in PATH. Attempting to use PowerShell for tests..."
        USE_POWERSHELL_FOR_TESTS=true
    else
        log "Cargo found: $(which cargo)"
        USE_POWERSHELL_FOR_TESTS=false
    fi
}

# Pre-build validation
pre_build_validation() {
    log "Running pre-build validation..."

    # Check if Docker is running
    if ! docker info > /dev/null 2>&1; then
        error "Docker is not running. Please start Docker and try again."
        exit 1
    fi

    # Check required files exist
    required_files=(
        "proof-messenger-relay/Dockerfile"
        "proof-messenger-relay/src/main.rs"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            error "Required file not found: $file"
            exit 1
        fi
    done

    # Validate database path configuration
    if ! grep -q "sqlite:/app/db/messages.db" proof-messenger-relay/src/main.rs; then
        error "Database path not configured correctly in main.rs"
        error "Expected: sqlite:/app/db/messages.db"
        exit 1
    fi

    # Validate Dockerfile permissions setup
    if ! grep -q "RUN chown proofmessenger:proofmessenger /app/db" proof-messenger-relay/Dockerfile; then
        error "Database permissions not configured correctly in Dockerfile"
        exit 1
    fi

    success "Pre-build validation passed"
}

# Run unit tests
run_unit_tests() {
    log "Running unit tests..."
    
    # Run tests for all workspace members
    if [ "$USE_POWERSHELL_FOR_TESTS" = "true" ]; then
        log "Using PowerShell to run cargo tests..."
        if powershell.exe -Command "cargo test --workspace --lib"; then
            success "Unit tests passed"
        else
            error "Unit tests failed"
            exit 1
        fi
    else
        if cargo test --workspace --lib; then
            success "Unit tests passed"
        else
            error "Unit tests failed"
            exit 1
        fi
    fi
}

# Build Docker image
build_docker_image() {
    log "Building Relay Server Docker image..."

    # Build relay server image
    if docker build -t proof-messenger-relay:latest -f proof-messenger-relay/Dockerfile .; then
        success "Relay Server image built successfully"
    else
        error "Failed to build Relay Server image"
        exit 1
    fi
}

# Validate built image
validate_image() {
    log "Validating built image..."

    # Check if image exists
    if ! docker image inspect proof-messenger-relay:latest > /dev/null 2>&1; then
        error "Relay Server image not found"
        exit 1
    fi

    success "Image validated successfully"
}

# Test container startup
test_container_startup() {
    log "Testing container startup..."

    # Test relay server container
    if docker run -d --name test-relay-server \
        -p $TEST_RELAY_PORT:8080 \
        -e DATABASE_URL=sqlite:/app/db/messages.db \
        proof-messenger-relay:latest; then
        success "Relay Server container started successfully"
    else
        error "Relay Server container failed to start"
        exit 1
    fi
}

# Wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=$HEALTH_CHECK_RETRIES
    local attempt=1

    log "Waiting for $service_name to be ready..."

    while [ $attempt -le $max_attempts ]; do
        if curl -s --fail --max-time 5 "$url" > /dev/null 2>&1; then
            success "$service_name is ready"
            return 0
        fi

        log "Attempt $attempt/$max_attempts: $service_name not ready yet, waiting..."
        sleep $HEALTH_CHECK_INTERVAL
        attempt=$((attempt + 1))
    done

    error "$service_name failed to become ready after $max_attempts attempts"
    return 1
}

# Test service health
test_service_health() {
    log "Testing service health..."

    # Test relay server health
    if wait_for_service "http://localhost:$TEST_RELAY_PORT/health" "Relay Server"; then
        # Test actual health endpoint response
        response=$(curl -s "http://localhost:$TEST_RELAY_PORT/health")
        if echo "$response" | grep -q '"status":"healthy"'; then
            success "Relay Server health check passed"
            log "Health response: $response"
        else
            error "Relay Server health check failed - unhealthy response"
            echo "Response: $response"
            exit 1
        fi
    else
        error "Relay Server health check failed - service not accessible"
        exit 1
    fi
}

# Main execution
main() {
    log "Starting Docker build process for Relay Server..."

    # Setup environment
    setup_environment

    # Pre-build phase
    pre_build_validation
    run_unit_tests

    # Build phase
    build_docker_image
    validate_image

    # Test phase
    test_container_startup
    test_service_health

    # Final validation
    log "Running final validation..."

    # Show image information
    log "Image Information:"
    docker images | head -1
    docker images | grep proof-messenger-relay || true

    success "All tests passed! Relay Server Docker build completed successfully."
    success "Ready to run: docker run -p 8080:8080 proof-messenger-relay:latest"

    log "Next steps:"
    log "1. Run 'docker run -d -p 8080:8080 -e DATABASE_URL=sqlite:/app/db/messages.db proof-messenger-relay:latest' to start the relay server"
    log "2. Access Relay Server API at: http://localhost:8080"
    log "3. Health check: http://localhost:8080/health"
}

# Run main function
main "$@"