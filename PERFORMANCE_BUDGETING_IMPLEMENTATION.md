# Performance Budgeting Implementation

## Overview

This implementation provides a comprehensive **Performance as a Feature** system that treats performance budgets as automated tests that can fail the build. The system follows the TDD workflow for performance testing and ensures performance regressions are caught before they reach production.

## Strategic Principle: "Performance as a Feature"

Performance isn't something you check occasionally; it's a core feature of your product that must be protected against regressions just like any other functionality. The best way to do this is to define a "performance budget" and write automated tests that fail the build if the budget is exceeded.

## TDD Workflow for Performance

### 1. Define the Requirement (The "Test")
Before writing any code, define specific, measurable Service Level Objectives (SLOs). This is your performance budget.

**Example SLO:**
```
The /verify-proof endpoint must maintain:
- 99th percentile (p99) latency of less than 150ms
- 95th percentile (p95) latency of less than 100ms
- Average latency of less than 50ms
- Error rate of less than 0.1%
- Minimum throughput of 100 RPS
When under a load of 500 concurrent users for 2 minutes
```

### 2. Write the Test Script (The "Implementation")
Create a load testing script that simulates realistic user behavior. The script itself doesn't contain the pass/fail logic.

### 3. Automate and Assert (The "Runner")
Create a wrapper script that executes the load test and then compares the results against your SLO. This script provides the pass/fail signal to your CI/CD system.

## Implementation Architecture

### Core Components

1. **Load Testing Script** (`locustfile.py`)
   - Simulates realistic user behavior patterns
   - Multiple user classes for different scenarios
   - Realistic payload generation
   - Comprehensive error handling

2. **Performance Budget Runner** (`run_perf_test.sh` / `run_perf_test.ps1`)
   - Defines Service Level Objectives (SLOs)
   - Executes load tests
   - Parses results and enforces budget
   - Generates performance reports
   - Provides pass/fail signals for CI/CD

3. **CI/CD Integration** (`.github/workflows/performance-budget.yml`)
   - Automated performance testing in CI/CD pipeline
   - Performance regression detection
   - PR comments with performance results
   - Performance trend analysis
   - Build failure on budget violations

4. **Configuration Management** (`config.py`)
   - Environment-specific performance budgets
   - Test scenario definitions
   - Endpoint-specific SLOs
   - Alert thresholds and monitoring config

## Performance Budgets (SLOs)

### Environment-Specific Budgets

```python
PERFORMANCE_BUDGETS = {
    "development": PerformanceBudget(
        max_p99_latency_ms=300,
        max_p95_latency_ms=200,
        max_avg_latency_ms=100,
        max_failure_rate_percent=1.0,
        min_rps=50
    ),
    
    "staging": PerformanceBudget(
        max_p99_latency_ms=200,
        max_p95_latency_ms=150,
        max_avg_latency_ms=75,
        max_failure_rate_percent=0.5,
        min_rps=100
    ),
    
    "production": PerformanceBudget(
        max_p99_latency_ms=150,
        max_p95_latency_ms=100,
        max_avg_latency_ms=50,
        max_failure_rate_percent=0.1,
        min_rps=200
    )
}
```

### Endpoint-Specific SLOs

```python
ENDPOINT_SLOS = {
    "/api/verify-proof": {
        "max_p99_latency_ms": 150,
        "max_avg_latency_ms": 50,
        "max_failure_rate_percent": 0.1
    },
    
    "/api/verify-biometric-proof": {
        "max_p99_latency_ms": 200,
        "max_avg_latency_ms": 75,
        "max_failure_rate_percent": 0.05
    },
    
    "/api/batch-verify-proofs": {
        "max_p99_latency_ms": 500,
        "max_avg_latency_ms": 200,
        "max_failure_rate_percent": 0.1
    }
}
```

## Load Testing Implementation

### Realistic User Simulation

```python
class ProofMessengerUser(HttpUser):
    """Simulates realistic user behavior patterns"""
    wait_time = between(0.5, 2.5)  # Realistic user think time
    
    @task(50)  # 50% of requests - most common operation
    def verify_login_proof(self):
        """Verify login proof - highest frequency operation"""
        payload = self.generate_proof_payload("login")
        # ... realistic request handling
    
    @task(30)  # 30% of requests - transaction verifications
    def verify_transaction_proof(self):
        """Verify transaction proof - medium frequency operation"""
        # ... transaction-specific testing
    
    @task(15)  # 15% of requests - biometric approvals
    def verify_biometric_proof(self):
        """Verify biometric proof - lower frequency, higher value"""
        # ... biometric-specific testing
```

### Multiple User Classes

```python
class HighVolumeUser(HttpUser):
    """Simulates high-volume batch processing clients"""
    wait_time = between(0.1, 0.5)  # Faster requests for batch processing
    
    @task
    def batch_verify_proofs(self):
        """Batch proof verification - simulates high-volume clients"""
        # ... batch processing simulation

class PeakHourUser(ProofMessengerUser):
    """Simulates peak hour traffic patterns"""
    wait_time = between(0.2, 1.0)  # Faster during peak hours
    weight = 3  # More of these users during peak testing
```

## Test Scenarios

### Comprehensive Test Coverage

```python
TEST_SCENARIOS = {
    "smoke": TestScenario(
        users=10, duration="30s",
        description="Quick smoke test to verify basic functionality"
    ),
    
    "normal": TestScenario(
        users=500, duration="2m",
        description="Normal production load simulation"
    ),
    
    "peak": TestScenario(
        users=1000, duration="5m",
        description="Peak hour traffic simulation"
    ),
    
    "stress": TestScenario(
        users=2000, duration="10m",
        description="Stress test to find breaking point"
    ),
    
    "capacity": TestScenario(
        users=5000, duration="15m",
        description="Maximum capacity test"
    ),
    
    "endurance": TestScenario(
        users=800, duration="30m",
        description="Long-running endurance test"
    )
}
```

## Performance Budget Enforcement

### Automated Budget Validation

```bash
# Performance Budget Assertion Logic
BUDGET_PASSED=true
VIOLATIONS=()

# Check p99 latency
if (( $(echo "$P99_LATENCY > $MAX_P99_LATENCY_MS" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("p99 latency: ${P99_LATENCY}ms > ${MAX_P99_LATENCY_MS}ms")
fi

# Check failure rate
if (( $(echo "$FAIL_RATE > $MAX_FAIL_RATE" | bc -l) )); then
    BUDGET_PASSED=false
    VIOLATIONS+=("failure rate: ${FAIL_RATE}% > ${MAX_FAIL_RATE}%")
fi

# Fail build if budget exceeded
if [ "$BUDGET_PASSED" = false ]; then
    echo "🔴 PERFORMANCE BUDGET TEST FAILED"
    exit 1  # Fail the CI/CD pipeline
fi
```

### CI/CD Integration

```yaml
# GitHub Actions workflow
- name: Run Performance Budget Test
  run: |
    ./run_perf_test.sh
  env:
    TARGET_HOST: http://localhost:8000
    MAX_P99_LATENCY_MS: 150
    MAX_FAIL_RATE: 0.1
    TEST_USERS: 500

- name: Fail build if performance budget exceeded
  if: failure()
  run: |
    echo "🔴 Performance budget test failed!"
    echo "The build is being failed to prevent performance regression."
    exit 1
```

## Performance Monitoring and Alerting

### Real-time Performance Tracking

```python
@events.request.add_listener
def on_request(request_type, name, response_time, response_length, exception, **kwargs):
    """Custom performance monitoring"""
    if exception:
        print(f"Request failed: {name} - {exception}")
    
    # Log slow requests for debugging
    if response_time > 1000:
        print(f"Slow request detected: {name} took {response_time}ms")
```

### Performance Alerts

```python
PERFORMANCE_ALERTS = {
    "critical": {
        "p99_latency_threshold_ms": 500,
        "failure_rate_threshold_percent": 1.0,
        "description": "Critical performance degradation - immediate attention required"
    },
    
    "warning": {
        "p99_latency_threshold_ms": 300,
        "failure_rate_threshold_percent": 0.5,
        "description": "Performance warning - investigation recommended"
    }
}
```

## Usage Examples

### Running Performance Tests

```bash
# Basic performance test
./run_perf_test.sh

# Peak hour scenario
TEST_SCENARIO=peak TEST_USERS=1000 ./run_perf_test.sh

# Stress test with custom budget
MAX_P99_LATENCY_MS=200 TEST_SCENARIO=stress ./run_perf_test.sh

# Capacity planning test
TEST_SCENARIO=capacity TEST_USERS=5000 TEST_DURATION=15m ./run_perf_test.sh
```

### PowerShell (Windows)

```powershell
# Basic performance test
.\run_perf_test.ps1

# Custom scenario
.\run_perf_test.ps1 -TestScenario "peak" -TestUsers 1000 -MaxP99LatencyMs 200

# Stress test
.\run_perf_test.ps1 -TestScenario "stress" -TestUsers 2000 -TestDuration "10m"
```

### CI/CD Integration

```yaml
# Manual trigger with custom parameters
workflow_dispatch:
  inputs:
    test_scenario:
      description: 'Test scenario to run'
      type: choice
      options: [normal, peak, stress, capacity]
    test_users:
      description: 'Number of concurrent users'
      default: '500'
```

## Performance Regression Detection

### Baseline Comparison

```python
def detect_performance_regression(current_metrics, baseline_metrics):
    """Detect performance regressions by comparing with baseline"""
    regressions = []
    
    # Check latency regression (>10% increase)
    if current_metrics['p99_latency'] > baseline_metrics['p99_latency'] * 1.1:
        regressions.append(f"p99 latency regression: {current_metrics['p99_latency']}ms vs {baseline_metrics['p99_latency']}ms baseline")
    
    # Check throughput regression (>10% decrease)
    if current_metrics['rps'] < baseline_metrics['rps'] * 0.9:
        regressions.append(f"throughput regression: {current_metrics['rps']} RPS vs {baseline_metrics['rps']} RPS baseline")
    
    return regressions
```

### PR Performance Comments

The CI/CD system automatically comments on pull requests with performance results:

```markdown
## 📊 Performance Budget Test Results

**✅ PASSED** - All performance metrics within budget

**Test Configuration:**
- Scenario: normal
- Users: 500
- Duration: 2m
- Commit: abc1234

**Performance Metrics:**
- Total Requests: 15,432
- Failed Requests: 12
- Requests/sec: 128.6
- Average Latency: 45.2ms
- 95th Percentile: 89.1ms
- 99th Percentile: 142.7ms
- Failure Rate: 0.08%
```

## Performance Dashboard Integration

### Metrics Collection

```python
def collect_performance_metrics(stats):
    """Collect metrics for dashboard integration"""
    return {
        'timestamp': datetime.now().isoformat(),
        'commit': os.environ.get('GITHUB_SHA'),
        'avg_latency': stats.total.avg_response_time,
        'p95_latency': stats.total.get_response_time_percentile(0.95),
        'p99_latency': stats.total.get_response_time_percentile(0.99),
        'rps': stats.total.current_rps,
        'failure_rate': (stats.total.num_failures / stats.total.num_requests) * 100
    }
```

### Integration Points

- **Time-series Database**: Store metrics in InfluxDB or Prometheus
- **Dashboards**: Visualize trends in Grafana
- **Alerting**: Set up alerts for performance degradation
- **Reporting**: Generate performance reports for stakeholders

## Best Practices

### 1. Define Realistic SLOs
- Base SLOs on actual user requirements
- Consider different user types and usage patterns
- Account for infrastructure limitations
- Review and adjust SLOs regularly

### 2. Test Early and Often
- Run smoke tests on every commit
- Full performance tests on PR merges
- Scheduled performance tests (daily/weekly)
- Performance tests before releases

### 3. Realistic Load Simulation
- Model actual user behavior patterns
- Use realistic data and payloads
- Simulate network conditions
- Include error scenarios

### 4. Comprehensive Monitoring
- Monitor application metrics (latency, throughput, errors)
- Monitor system metrics (CPU, memory, disk, network)
- Monitor business metrics (user experience, conversion rates)
- Set up alerting for all critical metrics

### 5. Performance Culture
- Make performance everyone's responsibility
- Include performance requirements in user stories
- Review performance impact of code changes
- Celebrate performance improvements

## Troubleshooting Performance Issues

### Common Performance Problems

1. **High Latency**
   - Check database query performance
   - Review algorithm complexity
   - Analyze network latency
   - Check resource contention

2. **Low Throughput**
   - Identify bottlenecks in request processing
   - Check connection pool settings
   - Review thread/async handling
   - Analyze resource utilization

3. **High Error Rates**
   - Check timeout configurations
   - Review error handling logic
   - Analyze resource exhaustion
   - Check dependency health

### Performance Optimization Workflow

1. **Identify**: Use performance tests to identify issues
2. **Measure**: Collect detailed metrics and profiling data
3. **Analyze**: Determine root cause of performance problems
4. **Optimize**: Implement performance improvements
5. **Validate**: Run performance tests to verify improvements
6. **Monitor**: Continuously monitor for regressions

## Conclusion

This performance budgeting implementation provides:

1. ✅ **TDD Workflow for Performance** (Define SLO → Write Test → Automate & Assert)
2. ✅ **Comprehensive Load Testing** (Realistic user simulation, multiple scenarios)
3. ✅ **Automated Budget Enforcement** (Build fails on budget violations)
4. ✅ **CI/CD Integration** (Automated testing, PR comments, trend analysis)
5. ✅ **Performance Regression Detection** (Baseline comparison, alerts)
6. ✅ **Comprehensive Monitoring** (Real-time metrics, dashboards, alerting)
7. ✅ **Multi-Environment Support** (Dev, staging, production budgets)
8. ✅ **Scalable Architecture** (Multiple test scenarios, user classes)

The system treats **performance as a feature** that must be protected against regressions, ensuring that performance issues are caught before they impact users in production. This proactive approach to performance management helps maintain high-quality user experiences and prevents costly performance incidents.