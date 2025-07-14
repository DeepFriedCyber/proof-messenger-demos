#!/usr/bin/env python3
"""
Mock Proof Messenger Server for Performance Testing
Simulates the actual server behavior for performance budget testing
"""

import json
import time
import random
import hashlib
from datetime import datetime
from flask import Flask, request, jsonify
from werkzeug.serving import WSGIRequestHandler
import threading
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Simulate server state
server_stats = {
    'requests_processed': 0,
    'start_time': datetime.now(),
    'errors': 0
}

# Simulate database/cache latency
def simulate_processing_time(endpoint_type="normal"):
    """Simulate realistic processing times for different endpoints"""
    base_latency = {
        "health": 0.001,      # 1ms for health checks
        "login": 0.025,       # 25ms for login proofs
        "transaction": 0.045,  # 45ms for transaction proofs
        "biometric": 0.065,   # 65ms for biometric proofs
        "batch": 0.120        # 120ms for batch processing
    }
    
    # Add some realistic variance (±20%)
    latency = base_latency.get(endpoint_type, 0.050)
    variance = latency * 0.2 * (random.random() - 0.5)
    final_latency = max(0.001, latency + variance)
    
    # Simulate occasional slow requests (5% chance of 2x latency)
    if random.random() < 0.05:
        final_latency *= 2
    
    time.sleep(final_latency)
    return final_latency

def validate_proof_payload(payload):
    """Validate proof payload structure"""
    required_fields = ['proof_bundle']
    
    if not isinstance(payload, dict):
        return False, "Payload must be a JSON object"
    
    for field in required_fields:
        if field not in payload:
            return False, f"Missing required field: {field}"
    
    proof_bundle = payload['proof_bundle']
    if not isinstance(proof_bundle, dict):
        return False, "proof_bundle must be an object"
    
    bundle_fields = ['context', 'signature']
    for field in bundle_fields:
        if field not in proof_bundle:
            return False, f"Missing required field in proof_bundle: {field}"
    
    return True, "Valid"

@app.route('/api/health', methods=['GET'])
def health_check():
    """Health check endpoint - must be very fast and reliable"""
    processing_time = simulate_processing_time("health")
    
    server_stats['requests_processed'] += 1
    
    uptime = (datetime.now() - server_stats['start_time']).total_seconds()
    
    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.now().isoformat(),
        'uptime_seconds': uptime,
        'requests_processed': server_stats['requests_processed'],
        'processing_time_ms': processing_time * 1000
    })

@app.route('/api/verify-proof', methods=['POST'])
def verify_proof():
    """Main proof verification endpoint"""
    processing_time = simulate_processing_time("login")
    
    server_stats['requests_processed'] += 1
    
    try:
        payload = request.get_json()
        
        # Validate payload
        is_valid, message = validate_proof_payload(payload)
        if not is_valid:
            server_stats['errors'] += 1
            return jsonify({
                'verified': False,
                'error': message,
                'timestamp': datetime.now().isoformat()
            }), 400
        
        # Simulate proof verification logic
        proof_bundle = payload['proof_bundle']
        context = proof_bundle['context']
        signature = proof_bundle['signature']
        
        # Parse context to determine proof type
        try:
            context_obj = json.loads(context)
            proof_type = context_obj.get('action', 'unknown')
        except json.JSONDecodeError:
            proof_type = 'unknown'
        
        # Adjust processing time based on proof type
        if proof_type == 'transaction':
            additional_time = simulate_processing_time("transaction") - processing_time
            time.sleep(max(0, additional_time))
        
        # Simulate verification success rate (99.9% success)
        verification_success = random.random() > 0.001
        
        if not verification_success:
            server_stats['errors'] += 1
            return jsonify({
                'verified': False,
                'error': 'Signature verification failed',
                'timestamp': datetime.now().isoformat(),
                'proof_type': proof_type
            }), 400
        
        # Successful verification
        return jsonify({
            'verified': True,
            'timestamp': datetime.now().isoformat(),
            'proof_type': proof_type,
            'processing_time_ms': processing_time * 1000,
            'verification_id': f"verify-{random.randint(100000, 999999)}"
        })
        
    except Exception as e:
        server_stats['errors'] += 1
        logger.error(f"Error processing proof verification: {e}")
        return jsonify({
            'verified': False,
            'error': 'Internal server error',
            'timestamp': datetime.now().isoformat()
        }), 500

@app.route('/api/verify-biometric-proof', methods=['POST'])
def verify_biometric_proof():
    """Biometric proof verification endpoint - higher security, slightly higher latency"""
    processing_time = simulate_processing_time("biometric")
    
    server_stats['requests_processed'] += 1
    
    try:
        payload = request.get_json()
        
        # Validate payload
        is_valid, message = validate_proof_payload(payload)
        if not is_valid:
            server_stats['errors'] += 1
            return jsonify({
                'verified': False,
                'error': message,
                'timestamp': datetime.now().isoformat()
            }), 400
        
        # Simulate additional biometric verification steps
        # - WebAuthn signature verification
        # - Device attestation validation
        # - Biometric policy checks
        time.sleep(0.020)  # Additional 20ms for biometric-specific processing
        
        # Higher success rate for biometric proofs (99.95% success)
        verification_success = random.random() > 0.0005
        
        if not verification_success:
            server_stats['errors'] += 1
            return jsonify({
                'verified': False,
                'error': 'Biometric verification failed',
                'timestamp': datetime.now().isoformat()
            }), 400
        
        return jsonify({
            'verified': True,
            'timestamp': datetime.now().isoformat(),
            'proof_type': 'biometric',
            'processing_time_ms': processing_time * 1000,
            'verification_id': f"bio-verify-{random.randint(100000, 999999)}",
            'security_level': 'high'
        })
        
    except Exception as e:
        server_stats['errors'] += 1
        logger.error(f"Error processing biometric proof verification: {e}")
        return jsonify({
            'verified': False,
            'error': 'Internal server error',
            'timestamp': datetime.now().isoformat()
        }), 500

@app.route('/api/batch-verify-proofs', methods=['POST'])
def batch_verify_proofs():
    """Batch proof verification endpoint"""
    processing_time = simulate_processing_time("batch")
    
    server_stats['requests_processed'] += 1
    
    try:
        payload = request.get_json()
        
        if 'proofs' not in payload or not isinstance(payload['proofs'], list):
            server_stats['errors'] += 1
            return jsonify({
                'error': 'proofs field must be an array',
                'timestamp': datetime.now().isoformat()
            }), 400
        
        proofs = payload['proofs']
        batch_size = len(proofs)
        
        # Simulate batch processing time (scales with batch size)
        batch_processing_time = 0.010 * batch_size  # 10ms per proof in batch
        time.sleep(batch_processing_time)
        
        # Process each proof in the batch
        verified_count = 0
        failed_proofs = []
        
        for i, proof in enumerate(proofs):
            # Simulate individual proof verification
            if random.random() > 0.002:  # 99.8% success rate for batch
                verified_count += 1
            else:
                failed_proofs.append({
                    'index': i,
                    'error': 'Verification failed'
                })
        
        return jsonify({
            'total_count': batch_size,
            'verified_count': verified_count,
            'failed_count': len(failed_proofs),
            'failed_proofs': failed_proofs,
            'processing_time_ms': (processing_time + batch_processing_time) * 1000,
            'timestamp': datetime.now().isoformat(),
            'batch_id': f"batch-{random.randint(100000, 999999)}"
        })
        
    except Exception as e:
        server_stats['errors'] += 1
        logger.error(f"Error processing batch verification: {e}")
        return jsonify({
            'error': 'Internal server error',
            'timestamp': datetime.now().isoformat()
        }), 500

@app.route('/api/stats', methods=['GET'])
def get_stats():
    """Server statistics endpoint"""
    uptime = (datetime.now() - server_stats['start_time']).total_seconds()
    
    return jsonify({
        'requests_processed': server_stats['requests_processed'],
        'errors': server_stats['errors'],
        'error_rate': server_stats['errors'] / max(1, server_stats['requests_processed']),
        'uptime_seconds': uptime,
        'requests_per_second': server_stats['requests_processed'] / max(1, uptime),
        'start_time': server_stats['start_time'].isoformat(),
        'current_time': datetime.now().isoformat()
    })

# Custom request handler to reduce logging noise during load testing
class QuietRequestHandler(WSGIRequestHandler):
    def log_request(self, code='-', size='-'):
        # Only log errors and important requests
        if str(code).startswith('4') or str(code).startswith('5'):
            super().log_request(code, size)

if __name__ == '__main__':
    print("🚀 Starting Mock Proof Messenger Server for Performance Testing")
    print("Available endpoints:")
    print("  GET  /api/health - Health check")
    print("  POST /api/verify-proof - Proof verification")
    print("  POST /api/verify-biometric-proof - Biometric proof verification")
    print("  POST /api/batch-verify-proofs - Batch proof verification")
    print("  GET  /api/stats - Server statistics")
    print("")
    print("Server will simulate realistic latencies and error rates")
    print("Use Ctrl+C to stop the server")
    print("")
    
    # Start the server
    app.run(
        host='0.0.0.0',
        port=8000,
        debug=False,
        threaded=True,
        request_handler=QuietRequestHandler
    )