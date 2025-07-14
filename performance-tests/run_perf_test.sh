#!/bin/bash

# Performance Budget Test Runner
# This script implements the TDD workflow for performance:
# 1. Define the Requirement (SLO) - Performance budget defined below
# 2. Write the Test Script - locustfile.py simulates realistic load  
# 3. Automate and Assert - This script enforces the budget

set -e  # Exit on any error

# --- 1. Define the Service Level Objective (SLO) ---
# This is our "test definition" - the performance budget
TARGET_HOST="${TARGET_HOST:-http://localhost:8000}"
MAX_P99_LATENCY_MS="${MAX_P99_LATENCY_MS:-150}"     # p99 must be under 150ms
MAX_P95_LATENCY_MS="${MAX_P95_LATENCY_MS:-100}"     # p95 must be under 100ms
MAX_AVG_LATENCY_MS="${MAX_AVG_LATENCY_MS:-50}"      # Average must be under 50ms
MAX_FAIL_RATE="${MAX_FAIL_RATE:-0.1}"               # Failure rate must be under 0.1%
MIN_RPS="${MIN_RPS:-100}"                           # Minimum requests per second
TEST_USERS="${TEST_USERS:-500}"                     # Number of concurrent users
SPAWN_RATE="${SPAWN_RATE:-50}"                      # Users spawned per second
TEST_DURATION="${TEST_DURATION:-2m}"                # Test duration
TEST_SCENARIO="${TEST_SCENARIO:-normal}"            # Test scenario (normal, peak, stress)

# Output directory for results
RESULTS_DIR="results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "🚀 === Starting Performance Budget Test ==="
echo "Target: $TARGET_HOST"
echo "Test Scenario: $TEST_SCENARIO"
echo "Users: $TEST_USERS (spawn rate: $SPAWN_RATE/s)"
echo "Duration: $TEST_DURATION"
echo ""
echo "📊 Performance Budget (SLO):"
echo "  • p99 Latency: < ${MAX_P99_LATENCY_MS}ms"
echo "  • p95 Latency: < ${MAX_P95_LATENCY_MS}ms"
echo "  • Avg Latency: < ${MAX_AVG_LATENCY_MS}ms"
echo "  • Failure Rate: < ${MAX_FAIL_RATE}%"
echo "  • Min RPS: > ${MIN_RPS}"
echo ""

# --- 2. Pre-test Health Check ---
echo "🔍 Pre-test health check..."
if ! curl -f -s "$TARGET_HOST/api/health" > /dev/null; then
    echo "❌ Health check failed. Is the server running at $TARGET_HOST?"
    exit 1
fi
echo "✅ Server is healthy"
echo ""

# --- 3. Run the Load Test ---
echo "⚡ Running load test..."

# Build locust command based on scenario
LOCUST_CMD="locust -f locustfile.py --host $TARGET_HOST \
    -u $TEST_USERS -r $SPAWN_RATE --run-time $TEST_DURATION \
    --headless --csv=$RESULTS_DIR/perf_results"

# Add scenario-specific parameters
case $TEST_SCENARIO in
    "peak")
        echo "🔥 Running PEAK HOUR scenario"
        LOCUST_CMD="$LOCUST_CMD --tags peak"
        ;;
    "stress")
        echo "💪 Running STRESS TEST scenario"
        LOCUST_CMD="$LOCUST_CMD --tags stress"
        ;;
    "capacity")
        echo "📈 Running CAPACITY TEST scenario"
        LOCUST_CMD="$LOCUST_CMD --tags capacity"
        ;;
    *)
        echo "📊 Running NORMAL scenario"
        ;;
esac

# Execute the load test
if ! $LOCUST_CMD; then
    echo "❌ Load test execution failed"
    exit 1
fi

echo "✅ Load test completed"
echo ""

# --- 4. Parse and Assert Results ---
echo "📊 Analyzing results..."

# Check if results files exist
STATS_FILE="$RESULTS_DIR/perf_results_stats.csv"
FAILURES_FILE="$RESULTS_DIR/perf_results_failures.csv"

if [ ! -f "$STATS_FILE" ]; then
    echo "❌ Results file not found: $STATS_FILE"
    exit 1
fi

# Parse the aggregate stats (last line with "Aggregated" data)
STATS_LINE=$(grep "Aggregated" "$STATS_FILE" | tail -n 1)

if [ -z "$STATS_LINE" ]; then
    echo "❌ Could not find aggregated stats in results"
    exit 1
fi

# Extract metrics from CSV (adjust column numbers based on Locust CSV format)
# Typical Locust CSV format: Type,Name,Request Count,Failure Count,Median,Average,Min,Max,Content Size,Requests/s,Failures/s,50%,66%,75%,80%,90%,95%,98%,99%,99.9%,99.99%,100%
REQUEST_COUNT=$(echo "$STATS_LINE" | cut -d, -f3)
FAILURE_COUNT=$(echo "$STATS_LINE" | cut -d, -f4)
AVG_LATENCY=$(echo "$STATS_LINE" | cut -d, -f6)
P95_LATENCY=$(echo "$STATS_LINE" | cut -d, -f16)
P99_LATENCY=$(echo "$STATS_LINE" | cut -d, -f19)
RPS=$(echo "$STATS_LINE" | cut -d, -f10)

# Calculate failure rate
if [ "$REQUEST_COUNT" -gt 0 ]; then
    FAIL_RATE=$(echo "scale=3; ($FAILURE_COUNT / $REQUEST_COUNT) * 100" | bc -l)
else
    FAIL_RATE=100
fi

# Display actual results
echo "📈 === Test Results ==="
echo "Total Requests: $REQUEST_COUNT"
echo "Failed Requests: $FAILURE_COUNT"
echo "Requests/sec: $RPS"
echo "Average Latency: ${AVG_LATENCY}ms"
echo "95th Percentile: ${P95_LATENCY}ms"
echo "99th Percentile: ${P99_LATENCY}ms"
echo "Failure Rate: ${FAIL_RATE}%"
echo ""

# --- 5. Performance Budget Assertion Logic ---
echo "🎯 === Performance Budget Validation ==="

BUDGET_PASSED=true
VIOLATIONS=()

# Check p99 latency
if (( $(echo "$P99_LATENCY > $MAX_P99_LATENCY_MS" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("p99 latency: ${P99_LATENCY}ms > ${MAX_P99_LATENCY_MS}ms")
else
    echo "✅ p99 latency: ${P99_LATENCY}ms ≤ ${MAX_P99_LATENCY_MS}ms"
fi

# Check p95 latency
if (( $(echo "$P95_LATENCY > $MAX_P95_LATENCY_MS" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("p95 latency: ${P95_LATENCY}ms > ${MAX_P95_LATENCY_MS}ms")
else
    echo "✅ p95 latency: ${P95_LATENCY}ms ≤ ${MAX_P95_LATENCY_MS}ms"
fi

# Check average latency
if (( $(echo "$AVG_LATENCY > $MAX_AVG_LATENCY_MS" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("avg latency: ${AVG_LATENCY}ms > ${MAX_AVG_LATENCY_MS}ms")
else
    echo "✅ avg latency: ${AVG_LATENCY}ms ≤ ${MAX_AVG_LATENCY_MS}ms"
fi

# Check failure rate
if (( $(echo "$FAIL_RATE > $MAX_FAIL_RATE" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("failure rate: ${FAIL_RATE}% > ${MAX_FAIL_RATE}%")
else
    echo "✅ failure rate: ${FAIL_RATE}% ≤ ${MAX_FAIL_RATE}%"
fi

# Check minimum RPS
if (( $(echo "$RPS < $MIN_RPS" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("throughput: ${RPS} RPS < ${MIN_RPS} RPS")
else
    echo "✅ throughput: ${RPS} RPS ≥ ${MIN_RPS} RPS"
fi

echo ""

# --- 6. Generate Performance Report ---
REPORT_FILE="$RESULTS_DIR/performance_report.md"
cat > "$REPORT_FILE" << EOF
# Performance Test Report

**Test Date:** $(date)
**Test Scenario:** $TEST_SCENARIO
**Target:** $TARGET_HOST
**Duration:** $TEST_DURATION
**Concurrent Users:** $TEST_USERS

## Performance Budget (SLO)
- p99 Latency: < ${MAX_P99_LATENCY_MS}ms
- p95 Latency: < ${MAX_P95_LATENCY_MS}ms
- Avg Latency: < ${MAX_AVG_LATENCY_MS}ms
- Failure Rate: < ${MAX_FAIL_RATE}%
- Min Throughput: > ${MIN_RPS} RPS

## Test Results
- **Total Requests:** $REQUEST_COUNT
- **Failed Requests:** $FAILURE_COUNT
- **Requests/sec:** $RPS
- **Average Latency:** ${AVG_LATENCY}ms
- **95th Percentile:** ${P95_LATENCY}ms
- **99th Percentile:** ${P99_LATENCY}ms
- **Failure Rate:** ${FAIL_RATE}%

## Budget Status
EOF

if [ "$BUDGET_PASSED" = true ]; then
    echo "**✅ PASSED** - All performance metrics within budget" >> "$REPORT_FILE"
else
    echo "**❌ FAILED** - Performance budget violations:" >> "$REPORT_FILE"
    for violation in "${VIOLATIONS[@]}"; do
        echo "- $violation" >> "$REPORT_FILE"
    done
fi

echo "" >> "$REPORT_FILE"
echo "## Files Generated" >> "$REPORT_FILE"
echo "- Stats: \`$STATS_FILE\`" >> "$REPORT_FILE"
echo "- Failures: \`$FAILURES_FILE\`" >> "$REPORT_FILE"
echo "- Report: \`$REPORT_FILE\`" >> "$REPORT_FILE"

# --- 7. Final Result ---
echo "📄 Performance report generated: $REPORT_FILE"
echo ""

if [ "$BUDGET_PASSED" = true ]; then
    echo "🎉 === PERFORMANCE BUDGET TEST PASSED ==="
    echo "All performance metrics are within the defined budget."
    echo "The application meets the Service Level Objectives (SLO)."
    exit 0
else
    echo "🔴 === PERFORMANCE BUDGET TEST FAILED ==="
    echo "The following performance budget violations were detected:"
    for violation in "${VIOLATIONS[@]}"; do
        echo "  • $violation"
    done
    echo ""
    echo "Performance regression detected. Build should be failed."
    echo "Review the performance report for detailed analysis."
    exit 1
fi