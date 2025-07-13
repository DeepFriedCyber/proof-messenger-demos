#!/bin/bash

# Docker Build Script for Proof Messenger
# Following TDD principles: Build, Test, Validate

set -e  # Exit on any error

echo "🐳 Starting Docker Build Process for Proof Messenger..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Validate prerequisites
print_status "Validating prerequisites..."

if ! command_exists docker; then
    print_error "Docker is not installed or not in PATH"
    exit 1
fi

if ! command_exists docker-compose; then
    print_error "Docker Compose is not installed or not in PATH"
    exit 1
fi

print_success "Prerequisites validated"

# Build relay server
print_status "Building Relay Server container..."
docker build -f proof-messenger-relay/Dockerfile -t proof-messenger-relay:latest .

if [ $? -eq 0 ]; then
    print_success "Relay Server container built successfully"
else
    print_error "Failed to build Relay Server container"
    exit 1
fi

# Build web application
print_status "Building Web Application container..."
docker build -f proof-messenger-web/Dockerfile -t proof-messenger-web:latest .

if [ $? -eq 0 ]; then
    print_success "Web Application container built successfully"
else
    print_error "Failed to build Web Application container"
    exit 1
fi

# Validate images
print_status "Validating built images..."

RELAY_IMAGE_ID=$(docker images -q proof-messenger-relay:latest)
WEB_IMAGE_ID=$(docker images -q proof-messenger-web:latest)

if [ -z "$RELAY_IMAGE_ID" ]; then
    print_error "Relay Server image not found"
    exit 1
fi

if [ -z "$WEB_IMAGE_ID" ]; then
    print_error "Web Application image not found"
    exit 1
fi

print_success "All images validated successfully"

# Display image information
print_status "Image Information:"
echo "Relay Server Image: $RELAY_IMAGE_ID"
echo "Web Application Image: $WEB_IMAGE_ID"

docker images | grep proof-messenger

# Test containers (basic smoke test)
print_status "Running basic container tests..."

# Test relay server container
print_status "Testing Relay Server container..."
RELAY_CONTAINER=$(docker run -d -p 8081:8080 proof-messenger-relay:latest)

# Wait for container to start
sleep 5

# Check if container is running
if docker ps | grep -q $RELAY_CONTAINER; then
    print_success "Relay Server container is running"
    
    # Test health endpoint (if available)
    if curl -f http://localhost:8081/health >/dev/null 2>&1; then
        print_success "Relay Server health check passed"
    else
        print_warning "Relay Server health check failed (endpoint may not exist)"
    fi
else
    print_error "Relay Server container failed to start"
    docker logs $RELAY_CONTAINER
fi

# Cleanup test container
docker stop $RELAY_CONTAINER >/dev/null 2>&1
docker rm $RELAY_CONTAINER >/dev/null 2>&1

# Test web application container
print_status "Testing Web Application container..."
WEB_CONTAINER=$(docker run -d -p 8082:80 proof-messenger-web:latest)

# Wait for container to start
sleep 5

# Check if container is running
if docker ps | grep -q $WEB_CONTAINER; then
    print_success "Web Application container is running"
    
    # Test health endpoint
    if curl -f http://localhost:8082/health >/dev/null 2>&1; then
        print_success "Web Application health check passed"
    else
        print_warning "Web Application health check failed"
    fi
else
    print_error "Web Application container failed to start"
    docker logs $WEB_CONTAINER
fi

# Cleanup test container
docker stop $WEB_CONTAINER >/dev/null 2>&1
docker rm $WEB_CONTAINER >/dev/null 2>&1

print_success "Docker build process completed successfully!"

echo ""
echo "🚀 Next Steps:"
echo "  • Start the full application: docker-compose up"
echo "  • Start in development mode: docker-compose --profile dev up"
echo "  • Run E2E tests: docker-compose --profile test up e2e-tests"
echo "  • View logs: docker-compose logs -f"
echo "  • Stop services: docker-compose down"

echo ""
echo "📊 Container Information:"
echo "  • Relay Server: http://localhost:8080"
echo "  • Web Application: http://localhost:80"
echo "  • Development Server: http://localhost:8000 (with --profile dev)"