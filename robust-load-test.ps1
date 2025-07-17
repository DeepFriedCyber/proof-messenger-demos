# Robust Load Testing Script for Proof-Messenger
# This script will generate load against the health endpoint

# Configuration
$targetUrl = "http://localhost:8081/health"
$concurrentUsers = 75  # Number of concurrent "users"
$requestsPerUser = 150  # Number of requests each "user" will make
$delayBetweenRequestsMs = 20  # Delay between requests in milliseconds
$totalRequests = $concurrentUsers * $requestsPerUser

# Results tracking
$results = @{
    TotalRequests = 0
    SuccessfulRequests = 0
    FailedRequests = 0
    TotalDurationMs = 0
    MinResponseTimeMs = [double]::MaxValue
    MaxResponseTimeMs = 0
    ResponseTimes = @()
}

# Function to calculate percentiles
function Get-Percentile {
    param (
        [double[]]$Values,
        [double]$Percentile
    )
    
    if ($Values.Count -eq 0) {
        return 0
    }
    
    $sortedValues = $Values | Sort-Object
    $index = [Math]::Ceiling($Percentile * $sortedValues.Count) - 1
    if ($index -lt 0) { $index = 0 }
    return $sortedValues[$index]
}

# Function to make a request and measure response time
function Make-Request {
    param (
        [string]$Url,
        [int]$RequestNumber,
        [int]$UserId
    )
    
    try {
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        $response = Invoke-RestMethod -Uri $Url -Method Get
        $stopwatch.Stop()
        $responseTimeMs = $stopwatch.ElapsedMilliseconds
        
        # Update results
        $script:results.TotalRequests++
        $script:results.SuccessfulRequests++
        $script:results.TotalDurationMs += $responseTimeMs
        $script:results.ResponseTimes += $responseTimeMs
        
        if ($responseTimeMs -lt $script:results.MinResponseTimeMs) {
            $script:results.MinResponseTimeMs = $responseTimeMs
        }
        
        if ($responseTimeMs -gt $script:results.MaxResponseTimeMs) {
            $script:results.MaxResponseTimeMs = $responseTimeMs
        }
        
        # Log every 10th request to avoid too much output
        if ($RequestNumber % 10 -eq 0) {
            Write-Host "User $UserId - Request $RequestNumber - Response time: $responseTimeMs ms - Status: $($response.status)" -ForegroundColor Green
        }
        
        return $true
    }
    catch {
        $script:results.TotalRequests++
        $script:results.FailedRequests++
        
        Write-Host "User $UserId - Request $RequestNumber - Failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to simulate a user making multiple requests
function Simulate-User {
    param (
        [int]$UserId,
        [int]$RequestCount,
        [int]$DelayMs
    )
    
    Write-Host "Starting User $UserId - Will make $RequestCount requests" -ForegroundColor Cyan
    
    for ($i = 1; $i -le $RequestCount; $i++) {
        $success = Make-Request -Url $targetUrl -RequestNumber $i -UserId $UserId
        
        # Add a small delay between requests
        if ($DelayMs -gt 0) {
            Start-Sleep -Milliseconds $DelayMs
        }
    }
    
    Write-Host "User $UserId completed all requests" -ForegroundColor Cyan
}

# Main load test function
function Start-LoadTest {
    param (
        [int]$ConcurrentUsers,
        [int]$RequestsPerUser,
        [int]$DelayBetweenRequestsMs
    )
    
    Write-Host "Starting load test with $ConcurrentUsers concurrent users, each making $RequestsPerUser requests" -ForegroundColor Green
    Write-Host "Total requests to be made: $($ConcurrentUsers * $RequestsPerUser)" -ForegroundColor Green
    Write-Host "Target URL: $targetUrl" -ForegroundColor Green
    Write-Host ""
    
    $overallStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    # Create and start jobs for each user
    $jobs = @()
    for ($userId = 1; $userId -le $ConcurrentUsers; $userId++) {
        $jobs += Start-Job -ScriptBlock {
            param($userId, $requestCount, $delayMs, $url)
            
            # Function to make a request and measure response time
            function Make-SingleRequest {
                param (
                    [string]$RequestUrl,
                    [int]$RequestNumber,
                    [int]$UserIdentifier
                )
                
                try {
                    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
                    $response = Invoke-RestMethod -Uri $RequestUrl -Method Get
                    $stopwatch.Stop()
                    $responseTimeMs = $stopwatch.ElapsedMilliseconds
                    
                    return @{
                        Success = $true
                        ResponseTimeMs = $responseTimeMs
                        Status = $response.status
                        RequestNumber = $RequestNumber
                        UserId = $UserIdentifier
                    }
                }
                catch {
                    return @{
                        Success = $false
                        Error = $_.Exception.Message
                        RequestNumber = $RequestNumber
                        UserId = $UserIdentifier
                    }
                }
            }
            
            $results = @()
            
            for ($i = 1; $i -le $requestCount; $i++) {
                $requestResult = Make-SingleRequest -RequestUrl $url -RequestNumber $i -UserIdentifier $userId
                $results += $requestResult
                
                # Add a small delay between requests
                if ($delayMs -gt 0) {
                    Start-Sleep -Milliseconds $delayMs
                }
            }
            
            return $results
        } -ArgumentList $userId, $RequestsPerUser, $DelayBetweenRequestsMs, $targetUrl
    }
    
    # Show progress while jobs are running
    $completed = $false
    while (-not $completed) {
        $runningJobs = $jobs | Where-Object { $_.State -eq 'Running' }
        if ($runningJobs.Count -eq 0) {
            $completed = $true
        }
        else {
            Write-Host "Waiting for $($runningJobs.Count) users to complete their requests..." -ForegroundColor Yellow
            Start-Sleep -Seconds 2
        }
    }
    
    # Collect results from all jobs
    $allResults = @()
    foreach ($job in $jobs) {
        $jobResults = Receive-Job -Job $job
        $allResults += $jobResults
    }
    
    # Clean up jobs
    $jobs | Remove-Job
    
    $overallStopwatch.Stop()
    $totalDurationSeconds = $overallStopwatch.Elapsed.TotalSeconds
    
    # Process and display results
    $successfulRequests = ($allResults | Where-Object { $_.Success -eq $true }).Count
    $failedRequests = ($allResults | Where-Object { $_.Success -eq $false }).Count
    $responseTimes = $allResults | Where-Object { $_.Success -eq $true } | ForEach-Object { $_.ResponseTimeMs }
    
    $avgResponseTime = if ($responseTimes.Count -gt 0) { ($responseTimes | Measure-Object -Average).Average } else { 0 }
    $minResponseTime = if ($responseTimes.Count -gt 0) { ($responseTimes | Measure-Object -Minimum).Minimum } else { 0 }
    $maxResponseTime = if ($responseTimes.Count -gt 0) { ($responseTimes | Measure-Object -Maximum).Maximum } else { 0 }
    
    # Calculate percentiles
    $p50 = if ($responseTimes.Count -gt 0) { Get-Percentile -Values $responseTimes -Percentile 0.5 } else { 0 }
    $p90 = if ($responseTimes.Count -gt 0) { Get-Percentile -Values $responseTimes -Percentile 0.9 } else { 0 }
    $p95 = if ($responseTimes.Count -gt 0) { Get-Percentile -Values $responseTimes -Percentile 0.95 } else { 0 }
    $p99 = if ($responseTimes.Count -gt 0) { Get-Percentile -Values $responseTimes -Percentile 0.99 } else { 0 }
    
    # Calculate requests per second
    $requestsPerSecond = if ($totalDurationSeconds -gt 0) { $successfulRequests / $totalDurationSeconds } else { 0 }
    
    # Display results
    Write-Host ""
    Write-Host "=== Load Test Results ===" -ForegroundColor Green
    Write-Host "Total Requests: $($successfulRequests + $failedRequests)" -ForegroundColor Cyan
    Write-Host "Successful Requests: $successfulRequests" -ForegroundColor Green
    Write-Host "Failed Requests: $failedRequests" -ForegroundColor $(if ($failedRequests -eq 0) { "Green" } else { "Red" })
    Write-Host "Success Rate: $([math]::Round(($successfulRequests / ($successfulRequests + $failedRequests)) * 100, 2))%" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Total Duration: $([math]::Round($totalDurationSeconds, 2)) seconds" -ForegroundColor Cyan
    Write-Host "Requests Per Second: $([math]::Round($requestsPerSecond, 2))" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Response Time Statistics:" -ForegroundColor Cyan
    Write-Host "  Min: $([math]::Round($minResponseTime, 2)) ms" -ForegroundColor Cyan
    Write-Host "  Avg: $([math]::Round($avgResponseTime, 2)) ms" -ForegroundColor Cyan
    Write-Host "  Max: $([math]::Round($maxResponseTime, 2)) ms" -ForegroundColor Cyan
    Write-Host "  p50: $([math]::Round($p50, 2)) ms" -ForegroundColor Cyan
    Write-Host "  p90: $([math]::Round($p90, 2)) ms" -ForegroundColor Cyan
    Write-Host "  p95: $([math]::Round($p95, 2)) ms" -ForegroundColor Cyan
    Write-Host "  p99: $([math]::Round($p99, 2)) ms" -ForegroundColor Cyan
    
    # Generate a report file
    $reportPath = "load-test-report-$(Get-Date -Format 'yyyyMMdd-HHmmss').md"
    
    @"
# Proof-Messenger Load Test Report

## Test Configuration
- **Target URL**: $targetUrl
- **Concurrent Users**: $ConcurrentUsers
- **Requests Per User**: $RequestsPerUser
- **Total Requests**: $($ConcurrentUsers * $RequestsPerUser)
- **Delay Between Requests**: $DelayBetweenRequestsMs ms

## Test Results
- **Total Requests**: $($successfulRequests + $failedRequests)
- **Successful Requests**: $successfulRequests
- **Failed Requests**: $failedRequests
- **Success Rate**: $([math]::Round(($successfulRequests / ($successfulRequests + $failedRequests)) * 100, 2))%

## Performance Metrics
- **Total Duration**: $([math]::Round($totalDurationSeconds, 2)) seconds
- **Requests Per Second**: $([math]::Round($requestsPerSecond, 2))

## Response Time Statistics
- **Minimum**: $([math]::Round($minResponseTime, 2)) ms
- **Average**: $([math]::Round($avgResponseTime, 2)) ms
- **Maximum**: $([math]::Round($maxResponseTime, 2)) ms
- **p50 (Median)**: $([math]::Round($p50, 2)) ms
- **p90**: $([math]::Round($p90, 2)) ms
- **p95**: $([math]::Round($p95, 2)) ms
- **p99**: $([math]::Round($p99, 2)) ms

## Service Level Objectives (SLOs)
- **p99 Latency Target**: < 150 ms - $(if ($p99 -lt 150) { "✅ PASSED" } else { "❌ FAILED" })
- **p95 Latency Target**: < 100 ms - $(if ($p95 -lt 100) { "✅ PASSED" } else { "❌ FAILED" })
- **Average Latency Target**: < 50 ms - $(if ($avgResponseTime -lt 50) { "✅ PASSED" } else { "❌ FAILED" })
- **Error Rate Target**: < 0.1% - $(if (($failedRequests / ($successfulRequests + $failedRequests) * 100) -lt 0.1) { "✅ PASSED" } else { "❌ FAILED" })
- **Throughput Target**: > 100 RPS - $(if ($requestsPerSecond -gt 100) { "✅ PASSED" } else { "❌ FAILED" })

## Conclusion
The Proof-Messenger relay server was tested under load with $ConcurrentUsers concurrent users making a total of $($ConcurrentUsers * $RequestsPerUser) requests.
The system demonstrated $(if ($requestsPerSecond -gt 100 -and $p99 -lt 150 -and $failedRequests -eq 0) { "excellent" } else { "acceptable" }) performance characteristics with a throughput of $([math]::Round($requestsPerSecond, 2)) requests per second and a p99 latency of $([math]::Round($p99, 2)) ms.

## Recommendations
- Continue monitoring the system in production using Grafana Cloud
- Set up alerts for any metrics that exceed the defined thresholds
- Consider periodic load testing as part of the CI/CD pipeline
"@ | Out-File -FilePath $reportPath
    
    Write-Host ""
    Write-Host "Load test report generated: $reportPath" -ForegroundColor Green
}

# Run the load test
Start-LoadTest -ConcurrentUsers $concurrentUsers -RequestsPerUser $requestsPerUser -DelayBetweenRequestsMs $delayBetweenRequestsMs