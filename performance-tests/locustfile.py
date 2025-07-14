"""
Performance Budget Test Suite for Proof Messenger
Locust load testing script that simulates realistic user behavior

This script implements the TDD workflow for performance:
1. Define the Requirement (SLO): p99 < 150ms, error rate < 0.1%
2. Write the Test Script: This file simulates realistic load
3. Automate and Assert: Runner script enforces the budget
"""

import json
import time
import random
import hashlib
from locust import HttpUser, task, between, events
from locust.runners import MasterRunner


class ProofMessengerUser(HttpUser):
    """
    Simulates a high-traffic client hitting the proof verification endpoints.
    Models realistic user behavior patterns for different proof types.
    """
    wait_time = between(0.5, 2.5)  # Wait 0.5-2.5s between tasks (realistic user behavior)
    
    def on_start(self):
        """Initialize user session with authentication"""
        self.user_id = f"user-{random.randint(1000, 9999)}"
        self.session_token = self.generate_session_token()
        
    def generate_session_token(self):
        """Generate a realistic session token"""
        timestamp = str(int(time.time()))
        return hashlib.sha256(f"{self.user_id}-{timestamp}".encode()).hexdigest()[:32]
    
    def generate_proof_payload(self, proof_type="login"):
        """Generate realistic proof payloads for different scenarios"""
        timestamp = int(time.time())
        
        if proof_type == "login":
            context = {
                "action": "login",
                "user_id": self.user_id,
                "timestamp": timestamp,
                "ip_address": f"192.168.1.{random.randint(1, 254)}",
                "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            }
        elif proof_type == "transaction":
            context = {
                "action": "wire_transfer",
                "user_id": self.user_id,
                "amount": random.randint(1000, 1000000),
                "destination": f"account-{random.randint(10000, 99999)}",
                "timestamp": timestamp,
                "request_id": f"txn-{random.randint(100000, 999999)}"
            }
        elif proof_type == "biometric":
            context = {
                "action": "biometric_approval",
                "user_id": self.user_id,
                "transaction_amount": random.randint(100000, 10000000),
                "timestamp": timestamp,
                "device_id": f"device-{random.randint(1000, 9999)}"
            }
        else:
            context = {
                "action": "generic_proof",
                "user_id": self.user_id,
                "timestamp": timestamp
            }
        
        # Generate a realistic signature (in production, this would be cryptographically valid)
        context_json = json.dumps(context, sort_keys=True)
        signature = hashlib.sha256(f"{context_json}-{self.session_token}".encode()).hexdigest()
        
        return {
            "proof_bundle": {
                "context": context_json,
                "signature": signature,
                "public_key": f"pk-{self.user_id}",
                "algorithm": "ECDSA-SHA256"
            },
            "metadata": {
                "client_version": "1.0.0",
                "timestamp": timestamp
            }
        }

    @task(50)  # 50% of requests - most common operation
    def verify_login_proof(self):
        """Verify login proof - highest frequency operation"""
        payload = self.generate_proof_payload("login")
        headers = {
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {self.session_token}',
            'User-Agent': 'ProofMessenger-Client/1.0.0'
        }
        
        with self.client.post(
            "/api/verify-proof",
            data=json.dumps(payload),
            headers=headers,
            name="verify_login_proof",
            catch_response=True
        ) as response:
            if response.status_code == 200:
                try:
                    result = response.json()
                    if result.get("verified") is True:
                        response.success()
                    else:
                        response.failure(f"Proof verification failed: {result.get('error', 'Unknown error')}")
                except json.JSONDecodeError:
                    response.failure("Invalid JSON response")
            else:
                response.failure(f"HTTP {response.status_code}: {response.text}")

    @task(30)  # 30% of requests - transaction verifications
    def verify_transaction_proof(self):
        """Verify transaction proof - medium frequency operation"""
        payload = self.generate_proof_payload("transaction")
        headers = {
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {self.session_token}'
        }
        
        with self.client.post(
            "/api/verify-proof",
            data=json.dumps(payload),
            headers=headers,
            name="verify_transaction_proof",
            catch_response=True
        ) as response:
            if response.status_code == 200:
                try:
                    result = response.json()
                    if result.get("verified") is True:
                        response.success()
                    else:
                        response.failure(f"Transaction proof verification failed")
                except json.JSONDecodeError:
                    response.failure("Invalid JSON response")
            else:
                response.failure(f"HTTP {response.status_code}")

    @task(15)  # 15% of requests - biometric approvals
    def verify_biometric_proof(self):
        """Verify biometric proof - lower frequency, higher value operation"""
        payload = self.generate_proof_payload("biometric")
        headers = {
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {self.session_token}'
        }
        
        with self.client.post(
            "/api/verify-biometric-proof",
            data=json.dumps(payload),
            headers=headers,
            name="verify_biometric_proof",
            catch_response=True
        ) as response:
            if response.status_code == 200:
                try:
                    result = response.json()
                    if result.get("verified") is True:
                        response.success()
                    else:
                        response.failure(f"Biometric proof verification failed")
                except json.JSONDecodeError:
                    response.failure("Invalid JSON response")
            else:
                response.failure(f"HTTP {response.status_code}")

    @task(5)  # 5% of requests - health checks and status
    def health_check(self):
        """Health check endpoint - monitoring and status"""
        with self.client.get(
            "/api/health",
            name="health_check",
            catch_response=True
        ) as response:
            if response.status_code == 200:
                try:
                    result = response.json()
                    if result.get("status") == "healthy":
                        response.success()
                    else:
                        response.failure("Service unhealthy")
                except json.JSONDecodeError:
                    response.failure("Invalid health check response")
            else:
                response.failure(f"Health check failed: HTTP {response.status_code}")


class HighVolumeUser(HttpUser):
    """
    Simulates high-volume batch processing clients
    Used for stress testing and capacity planning
    """
    wait_time = between(0.1, 0.5)  # Faster requests for batch processing
    weight = 1  # Lower weight - fewer of these users
    
    def on_start(self):
        self.client_id = f"batch-client-{random.randint(1000, 9999)}"
        self.api_key = f"api-key-{hashlib.sha256(self.client_id.encode()).hexdigest()[:16]}"
    
    @task
    def batch_verify_proofs(self):
        """Batch proof verification - simulates high-volume clients"""
        batch_size = random.randint(5, 20)
        proofs = []
        
        for i in range(batch_size):
            proof_type = random.choice(["login", "transaction"])
            user_id = f"batch-user-{random.randint(1000, 9999)}"
            timestamp = int(time.time())
            
            context = {
                "action": proof_type,
                "user_id": user_id,
                "timestamp": timestamp,
                "batch_id": f"batch-{random.randint(10000, 99999)}"
            }
            
            context_json = json.dumps(context, sort_keys=True)
            signature = hashlib.sha256(f"{context_json}-{self.api_key}".encode()).hexdigest()
            
            proofs.append({
                "context": context_json,
                "signature": signature,
                "public_key": f"pk-{user_id}"
            })
        
        payload = {
            "proofs": proofs,
            "client_id": self.client_id
        }
        
        headers = {
            'Content-Type': 'application/json',
            'X-API-Key': self.api_key
        }
        
        with self.client.post(
            "/api/batch-verify-proofs",
            data=json.dumps(payload),
            headers=headers,
            name="batch_verify_proofs",
            catch_response=True
        ) as response:
            if response.status_code == 200:
                try:
                    result = response.json()
                    verified_count = result.get("verified_count", 0)
                    total_count = result.get("total_count", 0)
                    
                    if verified_count == total_count:
                        response.success()
                    else:
                        response.failure(f"Batch verification incomplete: {verified_count}/{total_count}")
                except json.JSONDecodeError:
                    response.failure("Invalid batch response")
            else:
                response.failure(f"Batch verification failed: HTTP {response.status_code}")


# Performance monitoring events
@events.request.add_listener
def on_request(request_type, name, response_time, response_length, exception, context, **kwargs):
    """
    Custom performance monitoring
    Tracks detailed metrics for performance budget enforcement
    """
    if exception:
        print(f"Request failed: {name} - {exception}")
    
    # Log slow requests for debugging
    if response_time > 1000:  # Log requests slower than 1 second
        print(f"Slow request detected: {name} took {response_time}ms")


@events.test_start.add_listener
def on_test_start(environment, **kwargs):
    """Initialize performance test"""
    print("🚀 Starting Performance Budget Test")
    print(f"Target Host: {environment.host}")
    print("SLO: p99 < 150ms, Error Rate < 0.1%")


@events.test_stop.add_listener
def on_test_stop(environment, **kwargs):
    """Performance test completion"""
    print("📊 Performance test completed")
    
    # Get final stats
    stats = environment.stats
    total_requests = stats.total.num_requests
    total_failures = stats.total.num_failures
    
    if total_requests > 0:
        failure_rate = (total_failures / total_requests) * 100
        print(f"Total Requests: {total_requests}")
        print(f"Total Failures: {total_failures}")
        print(f"Failure Rate: {failure_rate:.2f}%")
        print(f"Average Response Time: {stats.total.avg_response_time:.2f}ms")
        print(f"95th Percentile: {stats.total.get_response_time_percentile(0.95):.2f}ms")
        print(f"99th Percentile: {stats.total.get_response_time_percentile(0.99):.2f}ms")


# Custom user classes for different load patterns
class PeakHourUser(ProofMessengerUser):
    """Simulates peak hour traffic patterns"""
    wait_time = between(0.2, 1.0)  # Faster during peak hours
    weight = 3  # More of these users during peak testing


class OffPeakUser(ProofMessengerUser):
    """Simulates off-peak traffic patterns"""
    wait_time = between(2.0, 5.0)  # Slower during off-peak
    weight = 1  # Fewer of these users


# Configuration for different test scenarios
def get_user_classes_for_scenario(scenario):
    """Return appropriate user classes for different test scenarios"""
    scenarios = {
        "normal": [ProofMessengerUser],
        "peak": [PeakHourUser, HighVolumeUser],
        "stress": [PeakHourUser, HighVolumeUser, ProofMessengerUser],
        "capacity": [HighVolumeUser]
    }
    return scenarios.get(scenario, [ProofMessengerUser])