# Test Script for Containerized Proof Messenger Application
# This script validates that both the relay server and web app are working correctly

Write-Host "🧪 Testing Containerized Proof Messenger Application" -ForegroundColor Cyan
Write-Host "=" * 60

# Test 1: Check if containers are running
Write-Host "`n1. Checking container status..." -ForegroundColor Yellow
$containers = docker-compose -f docker-compose.test.yml ps --format json | ConvertFrom-Json
foreach ($container in $containers) {
    $status = if ($container.State -eq "running") { "✅" } else { "❌" }
    Write-Host "   $status $($container.Name): $($container.State)" -ForegroundColor $(if ($container.State -eq "running") { "Green" } else { "Red" })
}

# Test 2: Test Relay Server Health
Write-Host "`n2. Testing Relay Server Health..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-WebRequest -Uri "http://localhost:8080/health" -UseBasicParsing
    if ($healthResponse.StatusCode -eq 200) {
        $healthData = $healthResponse.Content | ConvertFrom-Json
        Write-Host "   ✅ Relay Server Health: $($healthData.status)" -ForegroundColor Green
        Write-Host "   📊 Service: $($healthData.service)" -ForegroundColor Gray
        Write-Host "   🔢 Version: $($healthData.version)" -ForegroundColor Gray
    }
} catch {
    Write-Host "   ❌ Relay Server Health Check Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Test Web Application
Write-Host "`n3. Testing Web Application..." -ForegroundColor Yellow
try {
    $webResponse = Invoke-WebRequest -Uri "http://localhost/" -UseBasicParsing
    if ($webResponse.StatusCode -eq 200) {
        Write-Host "   ✅ Web Application: Accessible (Status: $($webResponse.StatusCode))" -ForegroundColor Green
        
        # Check if WASM files are accessible
        try {
            $wasmResponse = Invoke-WebRequest -Uri "http://localhost/pkg/proof_messenger_web.wasm" -UseBasicParsing
            if ($wasmResponse.StatusCode -eq 200) {
                Write-Host "   ✅ WASM Module: Accessible" -ForegroundColor Green
            }
        } catch {
            Write-Host "   ⚠️  WASM Module: Not accessible (this is expected if not built)" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "   ❌ Web Application Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Test CORS Headers
Write-Host "`n4. Testing CORS Configuration..." -ForegroundColor Yellow
try {
    $corsResponse = Invoke-WebRequest -Uri "http://localhost:8080/health" -UseBasicParsing
    $corsHeader = $corsResponse.Headers["Access-Control-Allow-Origin"]
    if ($corsHeader) {
        Write-Host "   ✅ CORS Headers: Present ($corsHeader)" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  CORS Headers: Not found" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ CORS Test Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Test Network Connectivity Between Containers
Write-Host "`n5. Testing Container Network..." -ForegroundColor Yellow
try {
    # Test if web container can reach relay container
    $networkTest = docker exec proof-messenger-web-test curl -s -f http://relay-server:8080/health
    if ($networkTest) {
        Write-Host "   ✅ Container Network: Web → Relay communication working" -ForegroundColor Green
    }
} catch {
    Write-Host "   ❌ Container Network Test Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Summary
Write-Host "`n" + "=" * 60
Write-Host "🎉 Containerized Application Test Complete!" -ForegroundColor Cyan
Write-Host "`n📋 Access Information:" -ForegroundColor White
Write-Host "   🌐 Web Application: http://localhost" -ForegroundColor Green
Write-Host "   🔗 Relay Server: http://localhost:8080" -ForegroundColor Green
Write-Host "   📊 Health Check: http://localhost:8080/health" -ForegroundColor Green

Write-Host "`n🛠️  Management Commands:" -ForegroundColor White
Write-Host "   Stop:    docker-compose -f docker-compose.test.yml down" -ForegroundColor Gray
Write-Host "   Logs:    docker-compose -f docker-compose.test.yml logs -f" -ForegroundColor Gray
Write-Host "   Restart: docker-compose -f docker-compose.test.yml restart" -ForegroundColor Gray