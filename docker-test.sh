#!/bin/bash

# Docker Test Script for Proof Messenger
# TDD-aligned testing of containerized application

set -e  # Exit on any error

echo "🧪 Starting Docker Test Suite for Proof Messenger..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    print_status "Running: $test_name"
    
    if eval "$test_command" >/dev/null 2>&1; then
        print_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local url="$1"
    local timeout="${2:-30}"
    local interval="${3:-2}"
    
    print_status "Waiting for service at $url..."
    
    for i in $(seq 1 $((timeout / interval))); do
        if curl -f "$url" >/dev/null 2>&1; then
            return 0
        fi
        sleep $interval
    done
    
    return 1
}

# Start the application stack
print_status "Starting application stack..."
docker-compose up -d

# Wait for services to be ready
print_status "Waiting for services to start..."
sleep 10

# Test 1: Relay Server Health
run_test "Relay Server Health Check" "curl -f http://localhost:8080/health"

# Test 2: Web Application Health
run_test "Web Application Health Check" "curl -f http://localhost/health"

# Test 3: Web Application Main Page
run_test "Web Application Main Page" "curl -f http://localhost/"

# Test 4: WASM Module Availability
run_test "WASM Module Availability" "curl -f http://localhost/pkg/proof_messenger_web_bg.wasm"

# Test 5: JavaScript Module Availability
run_test "JavaScript Module Availability" "curl -f http://localhost/pkg/proof_messenger_web.js"

# Test 6: Demo Page Availability
run_test "Demo Page Availability" "curl -f http://localhost/demo.html"

# Test 7: React Demo Page Availability
run_test "React Demo Page Availability" "curl -f http://localhost/react-demo.html"

# Test 8: Relay Server API Endpoint
print_status "Testing Relay Server API..."
RELAY_RESPONSE=$(curl -s -X POST http://localhost:8080/relay \
    -H "Content-Type: application/json" \
    -d '{"test": "data"}' || echo "FAILED")

if [[ "$RELAY_RESPONSE" != "FAILED" ]]; then
    print_success "Relay Server API Endpoint"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Relay Server API Endpoint"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# Test 9: Container Resource Usage
print_status "Checking container resource usage..."
RELAY_MEMORY=$(docker stats proof-messenger-relay --no-stream --format "{{.MemUsage}}" | cut -d'/' -f1)
WEB_MEMORY=$(docker stats proof-messenger-web --no-stream --format "{{.MemUsage}}" | cut -d'/' -f1)

print_status "Resource Usage:"
echo "  Relay Server Memory: $RELAY_MEMORY"
echo "  Web Application Memory: $WEB_MEMORY"

# Test 10: Container Logs Check
print_status "Checking container logs for errors..."
RELAY_ERRORS=$(docker logs proof-messenger-relay 2>&1 | grep -i error | wc -l)
WEB_ERRORS=$(docker logs proof-messenger-web 2>&1 | grep -i error | wc -l)

if [ "$RELAY_ERRORS" -eq 0 ] && [ "$WEB_ERRORS" -eq 0 ]; then
    print_success "No errors in container logs"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Errors found in container logs (Relay: $RELAY_ERRORS, Web: $WEB_ERRORS)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# Test 11: Network Connectivity
print_status "Testing network connectivity between containers..."
NETWORK_TEST=$(docker exec proof-messenger-web curl -f http://proof-messenger-relay:8080/health 2>/dev/null && echo "SUCCESS" || echo "FAILED")

if [[ "$NETWORK_TEST" == "SUCCESS" ]]; then
    print_success "Inter-container network connectivity"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Inter-container network connectivity"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# Test 12: Security Headers
print_status "Testing security headers..."
SECURITY_HEADERS=$(curl -I http://localhost/ 2>/dev/null | grep -E "(X-Frame-Options|X-Content-Type-Options|X-XSS-Protection)" | wc -l)

if [ "$SECURITY_HEADERS" -ge 2 ]; then
    print_success "Security headers present"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Insufficient security headers"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# Optional: Run E2E tests if profile is available
if docker-compose --profile test config >/dev/null 2>&1; then
    print_status "Running E2E tests..."
    if docker-compose --profile test up --exit-code-from e2e-tests e2e-tests; then
        print_success "E2E tests passed"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_error "E2E tests failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    TESTS_RUN=$((TESTS_RUN + 1))
fi

# Cleanup
print_status "Cleaning up test environment..."
docker-compose down

# Test Results Summary
echo ""
echo "📊 Test Results Summary:"
echo "  Tests Run: $TESTS_RUN"
echo "  Tests Passed: $TESTS_PASSED"
echo "  Tests Failed: $TESTS_FAILED"

if [ $TESTS_FAILED -eq 0 ]; then
    print_success "All tests passed! 🎉"
    echo ""
    echo "✅ Your containerized Proof Messenger application is working correctly!"
    echo ""
    echo "🚀 Ready for deployment:"
    echo "  • Production: docker-compose up -d"
    echo "  • Development: docker-compose --profile dev up -d"
    echo "  • With proxy: docker-compose --profile proxy up -d"
    exit 0
else
    print_error "Some tests failed. Please check the logs above."
    echo ""
    echo "🔧 Troubleshooting:"
    echo "  • Check container logs: docker-compose logs"
    echo "  • Verify build: ./docker-build.sh"
    echo "  • Check network: docker network ls"
    echo "  • Inspect containers: docker-compose ps"
    exit 1
fi