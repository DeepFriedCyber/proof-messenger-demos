#!/usr/bin/env python3
"""
Performance Budget Testing Demo
Demonstrates the TDD workflow for performance testing
"""

import subprocess
import sys
import time
import requests
import threading
import os
from datetime import datetime

def print_header(title):
    """Print a formatted header"""
    print("\n" + "="*60)
    print(f" {title}")
    print("="*60)

def print_step(step_num, description):
    """Print a formatted step"""
    print(f"\n🔹 Step {step_num}: {description}")

def start_mock_server():
    """Start the mock server in a separate process"""
    print_step(1, "Starting Mock Proof Messenger Server")
    
    # Start the mock server
    server_process = subprocess.Popen([
        sys.executable, "mock_server.py"
    ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    
    # Wait for server to start
    print("Waiting for server to start...")
    for i in range(10):
        try:
            response = requests.get("http://localhost:8000/api/health", timeout=2)
            if response.status_code == 200:
                print("✅ Mock server is running!")
                return server_process
        except requests.exceptions.RequestException:
            pass
        time.sleep(1)
    
    print("❌ Failed to start mock server")
    return None

def demonstrate_tdd_workflow():
    """Demonstrate the TDD workflow for performance testing"""
    
    print_header("Performance Budget Testing Demo")
    print("This demo shows the TDD workflow for performance testing:")
    print("1. Define the Requirement (SLO) - Performance budget")
    print("2. Write the Test Script - Load testing simulation")
    print("3. Automate and Assert - Budget enforcement")
    
    # Step 1: Start mock server
    server_process = start_mock_server()
    if not server_process:
        return False
    
    try:
        # Step 2: Show the performance budget (SLO)
        print_step(2, "Define Performance Budget (SLO)")
        print("📊 Performance Budget for this demo:")
        print("  • p99 Latency: < 150ms")
        print("  • p95 Latency: < 100ms")
        print("  • Avg Latency: < 50ms")
        print("  • Failure Rate: < 0.1%")
        print("  • Min Throughput: > 100 RPS")
        print("  • Test Load: 100 concurrent users for 30 seconds")
        
        # Step 3: Show server health
        print_step(3, "Verify Server Health")
        try:
            health_response = requests.get("http://localhost:8000/api/health")
            health_data = health_response.json()
            print(f"✅ Server Status: {health_data['status']}")
            print(f"   Uptime: {health_data['uptime_seconds']:.1f}s")
            print(f"   Requests Processed: {health_data['requests_processed']}")
        except Exception as e:
            print(f"❌ Health check failed: {e}")
            return False
        
        # Step 4: Run performance test
        print_step(4, "Execute Performance Budget Test")
        print("Running Locust load test with performance budget enforcement...")
        
        # Set environment variables for the test
        env = os.environ.copy()
        env.update({
            'TARGET_HOST': 'http://localhost:8000',
            'MAX_P99_LATENCY_MS': '150',
            'MAX_P95_LATENCY_MS': '100',
            'MAX_AVG_LATENCY_MS': '50',
            'MAX_FAIL_RATE': '0.1',
            'MIN_RPS': '100',
            'TEST_USERS': '100',
            'SPAWN_RATE': '20',
            'TEST_DURATION': '30s',
            'TEST_SCENARIO': 'normal'
        })
        
        # Run the performance test
        if os.name == 'nt':  # Windows
            test_command = ['powershell', '-ExecutionPolicy', 'Bypass', '-File', 'run_perf_test.ps1']
        else:  # Unix/Linux
            test_command = ['bash', 'run_perf_test.sh']
        
        print(f"Executing: {' '.join(test_command)}")
        
        test_process = subprocess.run(
            test_command,
            env=env,
            capture_output=True,
            text=True,
            timeout=120  # 2 minute timeout
        )
        
        # Step 5: Show results
        print_step(5, "Performance Test Results")
        
        if test_process.returncode == 0:
            print("🎉 PERFORMANCE BUDGET TEST PASSED!")
            print("✅ All performance metrics are within the defined budget")
        else:
            print("🔴 PERFORMANCE BUDGET TEST FAILED!")
            print("❌ Performance budget violations detected")
        
        print("\n📊 Test Output:")
        print(test_process.stdout)
        
        if test_process.stderr:
            print("\n⚠️  Errors/Warnings:")
            print(test_process.stderr)
        
        # Step 6: Show server stats
        print_step(6, "Final Server Statistics")
        try:
            stats_response = requests.get("http://localhost:8000/api/stats")
            stats_data = stats_response.json()
            print(f"📈 Server Performance Summary:")
            print(f"   Total Requests: {stats_data['requests_processed']}")
            print(f"   Errors: {stats_data['errors']}")
            print(f"   Error Rate: {stats_data['error_rate']:.3f}%")
            print(f"   Average RPS: {stats_data['requests_per_second']:.1f}")
            print(f"   Uptime: {stats_data['uptime_seconds']:.1f}s")
        except Exception as e:
            print(f"❌ Failed to get server stats: {e}")
        
        return test_process.returncode == 0
        
    finally:
        # Clean up: Stop the mock server
        print_step(7, "Cleanup")
        print("Stopping mock server...")
        server_process.terminate()
        server_process.wait()
        print("✅ Mock server stopped")

def show_performance_files():
    """Show the performance testing files created"""
    print_header("Performance Testing Files")
    
    files = [
        ("locustfile.py", "Load testing script with realistic user simulation"),
        ("run_perf_test.sh", "Performance budget runner (Linux/Mac)"),
        ("run_perf_test.ps1", "Performance budget runner (Windows)"),
        ("config.py", "Performance configuration and SLOs"),
        ("mock_server.py", "Mock server for testing"),
        ("requirements.txt", "Python dependencies"),
        (".github/workflows/performance-budget.yml", "CI/CD integration")
    ]
    
    for filename, description in files:
        if os.path.exists(filename):
            print(f"✅ {filename:<35} - {description}")
        else:
            print(f"❌ {filename:<35} - {description} (missing)")

def main():
    """Main demo function"""
    print("🚀 Performance Budget Testing Demo")
    print("This demo shows how to implement 'Performance as a Feature'")
    print("using the TDD workflow for performance testing.")
    
    # Check dependencies
    print_header("Dependency Check")
    
    try:
        import locust
        print(f"✅ Locust version: {locust.__version__}")
    except ImportError:
        print("❌ Locust not installed. Run: pip install -r requirements.txt")
        return False
    
    try:
        import flask
        print(f"✅ Flask version: {flask.__version__}")
    except ImportError:
        print("❌ Flask not installed. Run: pip install -r requirements.txt")
        return False
    
    # Show files
    show_performance_files()
    
    # Ask user if they want to run the demo
    print_header("Demo Execution")
    response = input("Do you want to run the performance budget test demo? (y/N): ")
    
    if response.lower() in ['y', 'yes']:
        success = demonstrate_tdd_workflow()
        
        print_header("Demo Summary")
        if success:
            print("🎉 Demo completed successfully!")
            print("The performance budget test passed, demonstrating that:")
            print("  • Performance requirements are defined as testable SLOs")
            print("  • Load tests simulate realistic user behavior")
            print("  • Budget enforcement prevents performance regressions")
            print("  • CI/CD integration ensures continuous performance monitoring")
        else:
            print("⚠️  Demo completed with performance budget violations")
            print("This demonstrates how the system catches performance regressions:")
            print("  • Build would fail in CI/CD pipeline")
            print("  • Developers would be notified of performance issues")
            print("  • Performance problems are caught before production")
        
        print("\n📚 Next Steps:")
        print("  1. Integrate performance tests into your CI/CD pipeline")
        print("  2. Set up performance monitoring and alerting")
        print("  3. Create performance dashboards for trend analysis")
        print("  4. Train team on performance budgeting practices")
        
    else:
        print("Demo skipped. You can run it later with: python demo.py")
    
    return True

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n🛑 Demo interrupted by user")
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        sys.exit(1)