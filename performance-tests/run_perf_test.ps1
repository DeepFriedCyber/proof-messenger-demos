# Performance Budget Test Runner (PowerShell)
# This script implements the TDD workflow for performance:
# 1. Define the Requirement (SLO) - Performance budget defined below
# 2. Write the Test Script - locustfile.py simulates realistic load  
# 3. Automate and Assert - This script enforces the budget

param(
    [string]$TargetHost = "http://localhost:8000",
    [int]$MaxP99LatencyMs = 150,
    [int]$MaxP95LatencyMs = 100,
    [int]$MaxAvgLatencyMs = 50,
    [double]$MaxFailRate = 0.1,
    [int]$MinRps = 100,
    [int]$TestUsers = 500,
    [int]$SpawnRate = 50,
    [string]$TestDuration = "2m",
    [string]$TestScenario = "normal"
)

# Set error action preference
$ErrorActionPreference = "Stop"

# --- 1. Define the Service Level Objective (SLO) ---
$ResultsDir = "results\$(Get-Date -Format 'yyyyMMdd_HHmmss')"
New-Item -ItemType Directory -Path $ResultsDir -Force | Out-Null

Write-Host "🚀 === Starting Performance Budget Test ===" -ForegroundColor Green
Write-Host "Target: $TargetHost"
Write-Host "Test Scenario: $TestScenario"
Write-Host "Users: $TestUsers (spawn rate: $SpawnRate/s)"
Write-Host "Duration: $TestDuration"
Write-Host ""
Write-Host "📊 Performance Budget (SLO):" -ForegroundColor Yellow
Write-Host "  • p99 Latency: < ${MaxP99LatencyMs}ms"
Write-Host "  • p95 Latency: < ${MaxP95LatencyMs}ms"
Write-Host "  • Avg Latency: < ${MaxAvgLatencyMs}ms"
Write-Host "  • Failure Rate: < ${MaxFailRate}%"
Write-Host "  • Min RPS: > ${MinRps}"
Write-Host ""

# --- 2. Pre-test Health Check ---
Write-Host "🔍 Pre-test health check..." -ForegroundColor Cyan
try {
    $healthResponse = Invoke-RestMethod -Uri "$TargetHost/api/health" -Method Get -TimeoutSec 10
    Write-Host "✅ Server is healthy" -ForegroundColor Green
} catch {
    Write-Host "❌ Health check failed. Is the server running at $TargetHost?" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
Write-Host ""

# --- 3. Run the Load Test ---
Write-Host "⚡ Running load test..." -ForegroundColor Cyan

# Check if locust is installed
try {
    $locustVersion = & locust --version 2>$null
    Write-Host "Using Locust: $locustVersion"
} catch {
    Write-Host "❌ Locust is not installed. Please install with: pip install locust" -ForegroundColor Red
    exit 1
}

# Build locust command
$locustArgs = @(
    "-f", "locustfile.py",
    "--host", $TargetHost,
    "-u", $TestUsers,
    "-r", $SpawnRate,
    "--run-time", $TestDuration,
    "--headless",
    "--csv", "$ResultsDir\perf_results"
)

# Add scenario-specific parameters
switch ($TestScenario) {
    "peak" {
        Write-Host "🔥 Running PEAK HOUR scenario" -ForegroundColor Yellow
        $locustArgs += "--tags", "peak"
    }
    "stress" {
        Write-Host "💪 Running STRESS TEST scenario" -ForegroundColor Yellow
        $locustArgs += "--tags", "stress"
    }
    "capacity" {
        Write-Host "📈 Running CAPACITY TEST scenario" -ForegroundColor Yellow
        $locustArgs += "--tags", "capacity"
    }
    default {
        Write-Host "📊 Running NORMAL scenario" -ForegroundColor Yellow
    }
}

# Execute the load test
try {
    & locust @locustArgs
    Write-Host "✅ Load test completed" -ForegroundColor Green
} catch {
    Write-Host "❌ Load test execution failed" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
Write-Host ""

# --- 4. Parse and Assert Results ---
Write-Host "📊 Analyzing results..." -ForegroundColor Cyan

$statsFile = "$ResultsDir\perf_results_stats.csv"
$failuresFile = "$ResultsDir\perf_results_failures.csv"

if (-not (Test-Path $statsFile)) {
    Write-Host "❌ Results file not found: $statsFile" -ForegroundColor Red
    exit 1
}

# Parse the aggregate stats
$statsContent = Get-Content $statsFile
$aggregatedLine = $statsContent | Where-Object { $_ -match "Aggregated" } | Select-Object -Last 1

if (-not $aggregatedLine) {
    Write-Host "❌ Could not find aggregated stats in results" -ForegroundColor Red
    exit 1
}

# Extract metrics from CSV
$fields = $aggregatedLine -split ','
$requestCount = [int]$fields[2]
$failureCount = [int]$fields[3]
$avgLatency = [double]$fields[5]
$p95Latency = [double]$fields[15]
$p99Latency = [double]$fields[18]
$rps = [double]$fields[9]

# Calculate failure rate
if ($requestCount -gt 0) {
    $failRate = ($failureCount / $requestCount) * 100
} else {
    $failRate = 100
}

# Display actual results
Write-Host "📈 === Test Results ===" -ForegroundColor Yellow
Write-Host "Total Requests: $requestCount"
Write-Host "Failed Requests: $failureCount"
Write-Host "Requests/sec: $([math]::Round($rps, 2))"
Write-Host "Average Latency: $([math]::Round($avgLatency, 2))ms"
Write-Host "95th Percentile: $([math]::Round($p95Latency, 2))ms"
Write-Host "99th Percentile: $([math]::Round($p99Latency, 2))ms"
Write-Host "Failure Rate: $([math]::Round($failRate, 3))%"
Write-Host ""

# --- 5. Performance Budget Assertion Logic ---
Write-Host "🎯 === Performance Budget Validation ===" -ForegroundColor Yellow

$budgetPassed = $true
$violations = @()

# Check p99 latency
if ($p99Latency -gt $MaxP99LatencyMs) {
    $budgetPassed = $false
    $violations += "p99 latency: $([math]::Round($p99Latency, 2))ms > ${MaxP99LatencyMs}ms"
} else {
    Write-Host "✅ p99 latency: $([math]::Round($p99Latency, 2))ms ≤ ${MaxP99LatencyMs}ms" -ForegroundColor Green
}

# Check p95 latency
if ($p95Latency -gt $MaxP95LatencyMs) {
    $budgetPassed = $false
    $violations += "p95 latency: $([math]::Round($p95Latency, 2))ms > ${MaxP95LatencyMs}ms"
} else {
    Write-Host "✅ p95 latency: $([math]::Round($p95Latency, 2))ms ≤ ${MaxP95LatencyMs}ms" -ForegroundColor Green
}

# Check average latency
if ($avgLatency -gt $MaxAvgLatencyMs) {
    $budgetPassed = $false
    $violations += "avg latency: $([math]::Round($avgLatency, 2))ms > ${MaxAvgLatencyMs}ms"
} else {
    Write-Host "✅ avg latency: $([math]::Round($avgLatency, 2))ms ≤ ${MaxAvgLatencyMs}ms" -ForegroundColor Green
}

# Check failure rate
if ($failRate -gt $MaxFailRate) {
    $budgetPassed = $false
    $violations += "failure rate: $([math]::Round($failRate, 3))% > ${MaxFailRate}%"
} else {
    Write-Host "✅ failure rate: $([math]::Round($failRate, 3))% ≤ ${MaxFailRate}%" -ForegroundColor Green
}

# Check minimum RPS
if ($rps -lt $MinRps) {
    $budgetPassed = $false
    $violations += "throughput: $([math]::Round($rps, 2)) RPS < ${MinRps} RPS"
} else {
    Write-Host "✅ throughput: $([math]::Round($rps, 2)) RPS ≥ ${MinRps} RPS" -ForegroundColor Green
}

Write-Host ""

# --- 6. Generate Performance Report ---
$reportFile = "$ResultsDir\performance_report.md"
$reportContent = @"
# Performance Test Report

**Test Date:** $(Get-Date)
**Test Scenario:** $TestScenario
**Target:** $TargetHost
**Duration:** $TestDuration
**Concurrent Users:** $TestUsers

## Performance Budget (SLO)
- p99 Latency: < ${MaxP99LatencyMs}ms
- p95 Latency: < ${MaxP95LatencyMs}ms
- Avg Latency: < ${MaxAvgLatencyMs}ms
- Failure Rate: < ${MaxFailRate}%
- Min Throughput: > ${MinRps} RPS

## Test Results
- **Total Requests:** $requestCount
- **Failed Requests:** $failureCount
- **Requests/sec:** $([math]::Round($rps, 2))
- **Average Latency:** $([math]::Round($avgLatency, 2))ms
- **95th Percentile:** $([math]::Round($p95Latency, 2))ms
- **99th Percentile:** $([math]::Round($p99Latency, 2))ms
- **Failure Rate:** $([math]::Round($failRate, 3))%

## Budget Status
"@

if ($budgetPassed) {
    $reportContent += "`n**✅ PASSED** - All performance metrics within budget"
} else {
    $reportContent += "`n**❌ FAILED** - Performance budget violations:"
    foreach ($violation in $violations) {
        $reportContent += "`n- $violation"
    }
}

$reportContent += @"

## Files Generated
- Stats: ``$statsFile``
- Failures: ``$failuresFile``
- Report: ``$reportFile``
"@

$reportContent | Out-File -FilePath $reportFile -Encoding UTF8

# --- 7. Final Result ---
Write-Host "📄 Performance report generated: $reportFile" -ForegroundColor Cyan
Write-Host ""

if ($budgetPassed) {
    Write-Host "🎉 === PERFORMANCE BUDGET TEST PASSED ===" -ForegroundColor Green
    Write-Host "All performance metrics are within the defined budget." -ForegroundColor Green
    Write-Host "The application meets the Service Level Objectives (SLO)." -ForegroundColor Green
    exit 0
} else {
    Write-Host "🔴 === PERFORMANCE BUDGET TEST FAILED ===" -ForegroundColor Red
    Write-Host "The following performance budget violations were detected:" -ForegroundColor Red
    foreach ($violation in $violations) {
        Write-Host "  • $violation" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "Performance regression detected. Build should be failed." -ForegroundColor Red
    Write-Host "Review the performance report for detailed analysis." -ForegroundColor Red
    exit 1
}