# Simple data integrity verification script

Write-Host "Starting simple data integrity verification..." -ForegroundColor Green

# Function to generate a test payload
function Generate-TestPayload {
    param (
        [int]$TestNumber
    )
    
    $timestamp = [int](Get-Date -UFormat %s)
    $userId = "test-user-$TestNumber"
    
    $context = @{
        action = "login"
        user_id = $userId
        timestamp = $timestamp
        ip_address = "192.168.1.100"
        user_agent = "IntegrityTest/1.0"
    }
    
    $contextJson = $context | ConvertTo-Json -Compress
    
    # Generate a simple signature (in a real scenario, this would be cryptographically valid)
    $signature = [System.BitConverter]::ToString(
        [System.Security.Cryptography.SHA256]::Create().ComputeHash(
            [System.Text.Encoding]::UTF8.GetBytes("$contextJson-secret-key")
        )
    ).Replace("-", "")
    
    $payload = @{
        proof_bundle = @{
            context = $contextJson
            signature = $signature
            public_key = "pk-$userId"
            algorithm = "ECDSA-SHA256"
        }
        metadata = @{
            client_version = "1.0.0"
            timestamp = $timestamp
            test_id = "integrity-$TestNumber"
        }
    } | ConvertTo-Json -Depth 10
    
    return $payload
}

# Run a series of verification tests
for ($i = 1; $i -le 10; $i++) {
    Write-Host "Running verification test #$i..." -ForegroundColor Yellow
    
    $payload = Generate-TestPayload -TestNumber $i
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:8081/api/verify-proof" -Method Post -Body $payload -ContentType "application/json"
        
        if ($response.verified -eq $true) {
            Write-Host "✅ Verification test #$i passed" -ForegroundColor Green
        } else {
            Write-Host "❌ Verification test #$i failed: $($response.error)" -ForegroundColor Red
        }
    } catch {
        Write-Host "❌ Verification test #$i request failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    # Add a short delay between tests
    Start-Sleep -Seconds 1
}

Write-Host "Data integrity verification complete!" -ForegroundColor Green