"""
Performance Testing Configuration
Defines Service Level Objectives (SLOs) and test scenarios for different environments
"""

import os
from dataclasses import dataclass
from typing import Dict, List, Optional


@dataclass
class PerformanceBudget:
    """
    Performance Budget (SLO) Definition
    These are the "tests" that must pass for the build to succeed
    """
    max_p99_latency_ms: int
    max_p95_latency_ms: int
    max_avg_latency_ms: int
    max_failure_rate_percent: float
    min_rps: int
    max_cpu_percent: Optional[int] = None
    max_memory_mb: Optional[int] = None
    max_error_rate_percent: Optional[float] = None


@dataclass
class TestScenario:
    """Test scenario configuration"""
    name: str
    users: int
    spawn_rate: int
    duration: str
    description: str
    user_classes: List[str]
    tags: List[str] = None


# Performance Budgets for Different Environments
PERFORMANCE_BUDGETS = {
    "development": PerformanceBudget(
        max_p99_latency_ms=300,
        max_p95_latency_ms=200,
        max_avg_latency_ms=100,
        max_failure_rate_percent=1.0,
        min_rps=50,
        max_cpu_percent=80,
        max_memory_mb=512
    ),
    
    "staging": PerformanceBudget(
        max_p99_latency_ms=200,
        max_p95_latency_ms=150,
        max_avg_latency_ms=75,
        max_failure_rate_percent=0.5,
        min_rps=100,
        max_cpu_percent=70,
        max_memory_mb=1024
    ),
    
    "production": PerformanceBudget(
        max_p99_latency_ms=150,
        max_p95_latency_ms=100,
        max_avg_latency_ms=50,
        max_failure_rate_percent=0.1,
        min_rps=200,
        max_cpu_percent=60,
        max_memory_mb=2048
    )
}

# Test Scenarios
TEST_SCENARIOS = {
    "smoke": TestScenario(
        name="Smoke Test",
        users=10,
        spawn_rate=5,
        duration="30s",
        description="Quick smoke test to verify basic functionality",
        user_classes=["ProofMessengerUser"],
        tags=["smoke"]
    ),
    
    "normal": TestScenario(
        name="Normal Load",
        users=500,
        spawn_rate=50,
        duration="2m",
        description="Normal production load simulation",
        user_classes=["ProofMessengerUser"],
        tags=["normal"]
    ),
    
    "peak": TestScenario(
        name="Peak Hour Load",
        users=1000,
        spawn_rate=100,
        duration="5m",
        description="Peak hour traffic simulation",
        user_classes=["PeakHourUser", "ProofMessengerUser"],
        tags=["peak"]
    ),
    
    "stress": TestScenario(
        name="Stress Test",
        users=2000,
        spawn_rate=200,
        duration="10m",
        description="Stress test to find breaking point",
        user_classes=["PeakHourUser", "HighVolumeUser", "ProofMessengerUser"],
        tags=["stress"]
    ),
    
    "capacity": TestScenario(
        name="Capacity Test",
        users=5000,
        spawn_rate=500,
        duration="15m",
        description="Maximum capacity test",
        user_classes=["HighVolumeUser"],
        tags=["capacity"]
    ),
    
    "endurance": TestScenario(
        name="Endurance Test",
        users=800,
        spawn_rate=80,
        duration="30m",
        description="Long-running endurance test",
        user_classes=["ProofMessengerUser", "HighVolumeUser"],
        tags=["endurance"]
    )
}

# Endpoint-specific SLOs
ENDPOINT_SLOS = {
    "/api/verify-proof": {
        "max_p99_latency_ms": 150,
        "max_avg_latency_ms": 50,
        "max_failure_rate_percent": 0.1,
        "description": "Core proof verification endpoint"
    },
    
    "/api/verify-biometric-proof": {
        "max_p99_latency_ms": 200,
        "max_avg_latency_ms": 75,
        "max_failure_rate_percent": 0.05,
        "description": "Biometric proof verification (higher security, slightly higher latency allowed)"
    },
    
    "/api/batch-verify-proofs": {
        "max_p99_latency_ms": 500,
        "max_avg_latency_ms": 200,
        "max_failure_rate_percent": 0.1,
        "description": "Batch processing endpoint (higher latency acceptable for batch operations)"
    },
    
    "/api/health": {
        "max_p99_latency_ms": 50,
        "max_avg_latency_ms": 10,
        "max_failure_rate_percent": 0.01,
        "description": "Health check endpoint (must be very fast and reliable)"
    }
}

# Performance Test Configuration
class PerformanceConfig:
    """Central configuration for performance testing"""
    
    @staticmethod
    def get_budget(environment: str = None) -> PerformanceBudget:
        """Get performance budget for environment"""
        env = environment or os.getenv("ENVIRONMENT", "development")
        return PERFORMANCE_BUDGETS.get(env, PERFORMANCE_BUDGETS["development"])
    
    @staticmethod
    def get_scenario(scenario_name: str = None) -> TestScenario:
        """Get test scenario configuration"""
        scenario = scenario_name or os.getenv("TEST_SCENARIO", "normal")
        return TEST_SCENARIOS.get(scenario, TEST_SCENARIOS["normal"])
    
    @staticmethod
    def get_target_host() -> str:
        """Get target host for testing"""
        return os.getenv("TARGET_HOST", "http://localhost:8000")
    
    @staticmethod
    def get_results_dir() -> str:
        """Get results directory"""
        from datetime import datetime
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        return f"results/{timestamp}"
    
    @staticmethod
    def should_fail_build() -> bool:
        """Check if build should fail on performance budget violation"""
        return os.getenv("FAIL_BUILD_ON_PERF_VIOLATION", "true").lower() == "true"
    
    @staticmethod
    def get_notification_config() -> Dict:
        """Get notification configuration for performance alerts"""
        return {
            "slack_webhook": os.getenv("SLACK_WEBHOOK_URL"),
            "email_recipients": os.getenv("PERF_ALERT_EMAILS", "").split(","),
            "pagerduty_key": os.getenv("PAGERDUTY_INTEGRATION_KEY"),
            "enabled": os.getenv("PERF_NOTIFICATIONS_ENABLED", "false").lower() == "true"
        }


# Performance Thresholds for Alerting
PERFORMANCE_ALERTS = {
    "critical": {
        "p99_latency_threshold_ms": 500,
        "failure_rate_threshold_percent": 1.0,
        "rps_drop_threshold_percent": 50,
        "description": "Critical performance degradation - immediate attention required"
    },
    
    "warning": {
        "p99_latency_threshold_ms": 300,
        "failure_rate_threshold_percent": 0.5,
        "rps_drop_threshold_percent": 25,
        "description": "Performance warning - investigation recommended"
    },
    
    "info": {
        "p99_latency_threshold_ms": 200,
        "failure_rate_threshold_percent": 0.2,
        "rps_drop_threshold_percent": 10,
        "description": "Performance info - minor degradation detected"
    }
}

# Resource Monitoring Configuration
RESOURCE_MONITORING = {
    "cpu_threshold_percent": 80,
    "memory_threshold_percent": 85,
    "disk_io_threshold_mbps": 100,
    "network_io_threshold_mbps": 50,
    "monitoring_interval_seconds": 5,
    "enabled": True
}

# Load Testing Patterns
LOAD_PATTERNS = {
    "constant": {
        "description": "Constant load throughout test duration",
        "implementation": "standard_locust"
    },
    
    "ramp_up": {
        "description": "Gradual ramp up to target load",
        "implementation": "custom_shape_class",
        "ramp_duration_percent": 20
    },
    
    "spike": {
        "description": "Sudden spike in load",
        "implementation": "custom_shape_class",
        "spike_multiplier": 3,
        "spike_duration_seconds": 60
    },
    
    "wave": {
        "description": "Wave pattern with peaks and valleys",
        "implementation": "custom_shape_class",
        "wave_period_minutes": 5,
        "wave_amplitude_percent": 50
    }
}

# Test Data Configuration
TEST_DATA_CONFIG = {
    "proof_types": ["login", "transaction", "biometric"],
    "user_pool_size": 10000,
    "transaction_amounts": {
        "min": 1000,
        "max": 1000000,
        "distribution": "log_normal"
    },
    "realistic_delays": {
        "user_think_time_min_ms": 500,
        "user_think_time_max_ms": 2500,
        "network_latency_simulation_ms": 50
    }
}