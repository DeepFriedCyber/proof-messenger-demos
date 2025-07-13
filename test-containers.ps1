# Test Script for Containerized Proof Messenger Application

Write-Host "Testing Containerized Proof Messenger Application" -ForegroundColor Cyan
Write-Host "=========================================================="

# Test 1: Check container status
Write-Host "`n1. Checking container status..." -ForegroundColor Yellow
docker-compose -f docker-compose.test.yml ps

# Test 2: Test Relay Server Health
Write-Host "`n2. Testing Relay Server Health..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-WebRequest -Uri "http://localhost:8080/health" -UseBasicParsing
    if ($healthResponse.StatusCode -eq 200) {
        Write-Host "   Relay Server Health: OK" -ForegroundColor Green
        Write-Host "   Response: $($healthResponse.Content)" -ForegroundColor Gray
    }
} catch {
    Write-Host "   Relay Server Health Check Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Test Web Application
Write-Host "`n3. Testing Web Application..." -ForegroundColor Yellow
try {
    $webResponse = Invoke-WebRequest -Uri "http://localhost/" -UseBasicParsing
    if ($webResponse.StatusCode -eq 200) {
        Write-Host "   Web Application: OK (Status: $($webResponse.StatusCode))" -ForegroundColor Green
    }
} catch {
    Write-Host "   Web Application Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Test Container Network
Write-Host "`n4. Testing Container Network..." -ForegroundColor Yellow
try {
    $networkTest = docker exec proof-messenger-web-test curl -s -f http://relay-server:8080/health
    if ($networkTest) {
        Write-Host "   Container Network: OK" -ForegroundColor Green
    }
} catch {
    Write-Host "   Container Network Test Failed" -ForegroundColor Red
}

Write-Host "`n=========================================================="
Write-Host "Test Complete!" -ForegroundColor Cyan
Write-Host "`nAccess Information:"
Write-Host "   Web Application: http://localhost"
Write-Host "   Relay Server: http://localhost:8080"
Write-Host "   Health Check: http://localhost:8080/health"