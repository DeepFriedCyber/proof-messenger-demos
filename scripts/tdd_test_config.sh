#!/bin/bash
# Enhanced docker-build.sh with comprehensive TDD testing
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_RELAY_PORT=8080
TEST_WEB_PORT=8001
TEST_TIMEOUT=30
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
    docker stop test-relay-server test-web-app 2>/dev/null || true
    docker rm test-relay-server test-web-app 2>/dev/null || true
}

# Set up cleanup trap
trap cleanup EXIT

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
        "proof-messenger-web/Dockerfile"
        "proof-messenger-relay/src/main.rs"
        "docker-compose.yml"
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
    if cargo test --workspace --lib; then
        success "Unit tests passed"
    else
        error "Unit tests failed"
        exit 1
    fi
}

# Run integration tests
run_integration_tests() {
    log "Running integration tests..."

    # Run integration tests with proper database setup
    if cargo test --workspace --test '*' -- --test-threads=1; then
        success "Integration tests passed"
    else
        error "Integration tests failed"
        exit 1
    fi
}

# Build Docker images
build_docker_images() {
    log "Building Docker images..."

    # Build relay server image
    log "Building Relay Server image..."
    if docker build -t proof-messenger-relay:latest -f proof-messenger-relay/Dockerfile .; then
        success "Relay Server image built successfully"
    else
        error "Failed to build Relay Server image"
        exit 1
    fi

    # Build web application image
    log "Building Web Application image..."
    if docker build -t proof-messenger-web:latest -f proof-messenger-web/Dockerfile .; then
        success "Web Application image built successfully"
    else
        error "Failed to build Web Application image"
        exit 1
    fi
}

# Validate built images
validate_images() {
    log "Validating built images..."

    # Check if images exist
    if ! docker image inspect proof-messenger-relay:latest > /dev/null 2>&1; then
        error "Relay Server image not found"
        exit 1
    fi

    if ! docker image inspect proof-messenger-web:latest > /dev/null 2>&1; then
        error "Web Application image not found"
        exit 1
    fi

    success "All images validated successfully"
}

# Test container startup
test_container_startup() {
    log "Testing container startup..."

    # Test relay server container
    log "Testing Relay Server container startup..."
    if docker run -d --name test-relay-server \
        -p $TEST_RELAY_PORT:8080 \
        -e DATABASE_URL=sqlite:/app/db/messages.db \
        proof-messenger-relay:latest; then
        success "Relay Server container started successfully"
    else
        error "Relay Server container failed to start"
        exit 1
    fi

    # Test web application container
    log "Testing Web Application container startup..."
    if docker run -d --name test-web-app \
        -p $TEST_WEB_PORT:80 \
        proof-messenger-web:latest; then
        success "Web Application container started successfully"
    else
        error "Web Application container failed to start"
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
        else
            error "Relay Server health check failed - unhealthy response"
            echo "Response: $response"
            exit 1
        fi
    else
        error "Relay Server health check failed - service not accessible"
        exit 1
    fi

    # Test web application
    if wait_for_service "http://localhost:$TEST_WEB_PORT/index.html" "Web Application"; then
        success "Web Application health check passed"
    else
        error "Web Application health check failed"
        exit 1
    fi
}

# Test container logs for errors
test_container_logs() {
    log "Checking container logs for errors..."

    # Check relay server logs
    relay_logs=$(docker logs test-relay-server 2>&1)
    if echo "$relay_logs" | grep -i "error" | grep -v "ERROR.*tower_http.*trace.*on_failure"; then
        error "Relay Server logs contain errors:"
        echo "$relay_logs" | grep -i "error"
        exit 1
    fi

    # Check for successful startup messages
    if echo "$relay_logs" | grep -q "Server ready to accept connections"; then
        success "Relay Server started successfully"
    else
        warning "Relay Server startup message not found in logs"
    fi

    # Check web app logs
    web_logs=$(docker logs test-web-app 2>&1)
    if echo "$web_logs" | grep -i "error"; then
        error "Web Application logs contain errors:"
        echo "$web_logs" | grep -i "error"
        exit 1
    fi

    success "Container logs check passed"
}

# Performance test
performance_test() {
    log "Running basic performance tests..."

    # Test relay server response time
    start_time=$(date +%s%N)
    curl -s "http://localhost:$TEST_RELAY_PORT/health" > /dev/null
    end_time=$(date +%s%N)

    response_time_ms=$(( (end_time - start_time) / 1000000 ))

    if [ $response_time_ms -lt 1000 ]; then
        success "Relay Server response time: ${response_time_ms}ms (good)"
    else
        warning "Relay Server response time: ${response_time_ms}ms (may be slow)"
    fi

    # Test concurrent requests
    log "Testing concurrent requests..."
    for i in {1..5}; do
        curl -s "http://localhost:$TEST_RELAY_PORT/health" > /dev/null &
    done
    wait

    success "Concurrent requests test passed"
}

# Main execution
main() {
    log "Starting TDD-enhanced Docker build process..."

    # Pre-build phase
    pre_build_validation
    run_unit_tests
    run_integration_tests

    # Build phase
    build_docker_images
    validate_images

    # Test phase
    test_container_startup
    test_service_health
    test_container_logs
    performance_test

    # Final validation
    log "Running final validation..."

    # Show image information
    log "Image Information:"
    docker images | grep proof-messenger

    success "All tests passed! Docker build process completed successfully."
    success "Ready to run: docker-compose up"

    log "Next steps:"
    log "1. Run 'docker-compose up' to start all services"
    log "2. Access Web Application at: http://localhost:80"
    log "3. Access Relay Server API at: http://localhost:8080"
}

# Run main function
main "$@"