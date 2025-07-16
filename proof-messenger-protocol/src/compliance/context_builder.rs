// src/compliance/context_builder.rs
//! Policy-Driven Context Builder
//! 
//! This module implements the TDD workflow for compliance:
//! 1. Define the Data Policy (The "Test") - Strict policy for required/forbidden fields
//! 2. Write the Test Case - Test with "dirty" data containing PII
//! 3. Implement the Sanitizer - Function that enforces policy and passes tests

use serde_json::{Value, Map};
use std::collections::HashSet;
use crate::compliance::data_policies::DataPolicy;
use crate::compliance::pii_detector::{PIIDetector, PIIType};
use crate::compliance::audit_logger::ComplianceAuditLogger;

/// Result type for context building operations
#[derive(Debug, Clone, PartialEq)]
pub enum ContextBuildResult {
    Success(Value),
    PolicyViolation(Vec<String>),
    PIIDetected(Vec<String>),
    MissingRequiredFields(Vec<String>),
}

/// Creates a secure context object by only including fields specified in a policy.
/// This prevents accidental leakage of PII into proofs or logs.
/// 
/// # Arguments
/// * `raw_input` - A serde_json::Value object containing potentially sensitive data
/// * `policy` - A slice of strings representing the only keys allowed in the final context
/// 
/// # Returns
/// A clean Value object containing only policy-approved fields
pub fn create_secure_context(raw_input: &Value, policy: &[String]) -> Value {
    let mut clean_context = Map::new();
    
    if let Some(input_map) = raw_input.as_object() {
        for key in policy {
            if let Some(value) = input_map.get(key) {
                // Only copy the value if the key is in our policy
                clean_context.insert(key.clone(), value.clone());
            }
        }
    }
    
    Value::Object(clean_context)
}

/// Advanced secure context builder with comprehensive compliance checking
/// 
/// # Arguments
/// * `raw_input` - Raw input data that may contain PII
/// * `policy` - Data policy defining allowed/forbidden fields and validation rules
/// * `context_type` - Type of context being created (for audit logging)
/// 
/// # Returns
/// ContextBuildResult indicating success or specific compliance violations
pub fn create_secure_context_advanced(
    raw_input: &Value,
    policy: &DataPolicy,
    context_type: &str,
) -> ContextBuildResult {
    let mut audit_logger = ComplianceAuditLogger::new();
    let pii_detector = PIIDetector::new();
    
    // Log the sanitization attempt
    audit_logger.log_sanitization_attempt(context_type, raw_input);
    
    if let Some(input_map) = raw_input.as_object() {
        let mut violations = Vec::new();
        let mut pii_detected = Vec::new();
        let mut missing_required = Vec::new();
        let mut clean_context = Map::new();
        
        // Check for forbidden fields
        for (key, value) in input_map {
            if policy.forbidden_fields.contains(key) {
                violations.push(format!("Forbidden field detected: {}", key));
                audit_logger.log_policy_violation(context_type, key, "forbidden_field");
                continue;
            }
            
            // Check for PII in field values
            if let Some(pii_types) = pii_detector.detect_pii(value) {
                if !pii_types.is_empty() {
                    pii_detected.push(format!("PII detected in field '{}': {:?}", key, pii_types));
                    let pii_vec: Vec<PIIType> = pii_types.into_iter().collect();
                    audit_logger.log_pii_detection(context_type, key, &pii_vec);
                    continue;
                }
            }
            
            // Include field if it's in the allowed list
            if policy.required_fields.contains(key) || policy.optional_fields.contains(key) {
                clean_context.insert(key.clone(), value.clone());
            }
        }
        
        // Check for missing required fields
        for required_field in &policy.required_fields {
            if !input_map.contains_key(required_field) {
                missing_required.push(required_field.clone());
            }
        }
        
        // Return appropriate result based on violations
        if !violations.is_empty() {
            audit_logger.log_sanitization_failure(context_type, "policy_violation");
            return ContextBuildResult::PolicyViolation(violations);
        }
        
        if !pii_detected.is_empty() {
            audit_logger.log_sanitization_failure(context_type, "pii_detected");
            return ContextBuildResult::PIIDetected(pii_detected);
        }
        
        if !missing_required.is_empty() {
            audit_logger.log_sanitization_failure(context_type, "missing_required_fields");
            return ContextBuildResult::MissingRequiredFields(missing_required);
        }
        
        let clean_value = Value::Object(clean_context);
        audit_logger.log_sanitization_success(context_type, &clean_value);
        ContextBuildResult::Success(clean_value)
    } else {
        audit_logger.log_sanitization_failure(context_type, "invalid_input_format");
        ContextBuildResult::PolicyViolation(vec!["Input must be a JSON object".to_string()])
    }
}

/// Validates that a context object complies with a given policy
/// 
/// # Arguments
/// * `context` - The context object to validate
/// * `policy` - The data policy to validate against
/// 
/// # Returns
/// Vec of validation errors (empty if valid)
pub fn validate_context_compliance(context: &Value, policy: &DataPolicy) -> Vec<String> {
    let mut errors = Vec::new();
    
    if let Some(context_map) = context.as_object() {
        // Check for forbidden fields
        for key in context_map.keys() {
            if policy.forbidden_fields.contains(key) {
                errors.push(format!("Context contains forbidden field: {}", key));
            }
        }
        
        // Check for missing required fields
        for required_field in &policy.required_fields {
            if !context_map.contains_key(required_field) {
                errors.push(format!("Context missing required field: {}", required_field));
            }
        }
        
        // Check for unknown fields (not in required or optional)
        let allowed_fields: HashSet<_> = policy.required_fields
            .iter()
            .chain(policy.optional_fields.iter())
            .collect();
        
        for key in context_map.keys() {
            if !allowed_fields.contains(key) {
                errors.push(format!("Context contains unknown field: {}", key));
            }
        }
    } else {
        errors.push("Context must be a JSON object".to_string());
    }
    
    errors
}

/// Sanitizes a context object by removing any fields not in the policy
/// 
/// # Arguments
/// * `context` - The context object to sanitize
/// * `policy` - The data policy defining allowed fields
/// 
/// # Returns
/// A sanitized context object containing only policy-approved fields
pub fn sanitize_existing_context(context: &Value, policy: &DataPolicy) -> Value {
    let mut clean_context = Map::new();
    
    if let Some(context_map) = context.as_object() {
        let allowed_fields: HashSet<_> = policy.required_fields
            .iter()
            .chain(policy.optional_fields.iter())
            .collect();
        
        for (key, value) in context_map {
            if allowed_fields.contains(key) && !policy.forbidden_fields.contains(key) {
                clean_context.insert(key.clone(), value.clone());
            }
        }
    }
    
    Value::Object(clean_context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::compliance::data_policies::{create_fintech_policy, create_biometric_policy, create_audit_policy};

    #[test]
    fn test_fintech_context_sanitization() {
        // 1. Define the policy for this specific context
        let fintech_policy = vec![
            "action".to_string(),
            "amount_usd_cents".to_string(),
            "destination_account".to_string(),
        ];

        // 2. Create a "dirty" input object with extra PII
        let dirty_input = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 5000000,
            "destination_account": "ACME-123",
            "user_ip": "192.168.1.100",    // PII to be stripped
            "session_id": "abc-xyz-789"    // PII to be stripped
        });

        // 3. Define the expected "clean" output
        let expected_clean_context = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 5000000,
            "destination_account": "ACME-123"
        });

        // 4. Call the function and assert that the output is correct
        let clean_context = create_secure_context(&dirty_input, &fintech_policy);
        assert_eq!(clean_context, expected_clean_context);
    }

    #[test]
    fn test_biometric_context_sanitization() {
        // Policy for biometric authentication context
        let biometric_policy = vec![
            "action".to_string(),
            "user_id".to_string(),
            "timestamp".to_string(),
            "device_attestation".to_string(),
        ];

        // Dirty input with biometric PII that should be stripped
        let dirty_input = json!({
            "action": "biometric_approval",
            "user_id": "user-12345",
            "timestamp": 1678886400,
            "device_attestation": "platform-authenticator",
            "fingerprint_template": "base64-encoded-template", // PII to be stripped
            "face_encoding": "biometric-data-here",           // PII to be stripped
            "device_serial": "ABC123XYZ"                      // PII to be stripped
        });

        let expected_clean_context = json!({
            "action": "biometric_approval",
            "user_id": "user-12345",
            "timestamp": 1678886400,
            "device_attestation": "platform-authenticator"
        });

        let clean_context = create_secure_context(&dirty_input, &biometric_policy);
        assert_eq!(clean_context, expected_clean_context);
    }

    #[test]
    fn test_advanced_context_builder_with_policy_violations() {
        let policy = create_fintech_policy();
        
        // Input with forbidden fields
        let dirty_input = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            "destination_account": "ACME-123",
            "initiator_id": "user-456",
            "timestamp": 1678886400,
            "user_ip": "192.168.1.100",        // Forbidden field
            "session_id": "session-abc-123"    // Forbidden field
        });

        let result = create_secure_context_advanced(&dirty_input, &policy, "fintech_transfer");
        
        match result {
            ContextBuildResult::PolicyViolation(violations) => {
                assert!(violations.len() >= 2);
                assert!(violations.iter().any(|v| v.contains("user_ip")));
                assert!(violations.iter().any(|v| v.contains("session_id")));
            }
            _ => panic!("Expected PolicyViolation result"),
        }
    }

    #[test]
    fn test_advanced_context_builder_with_missing_required_fields() {
        let policy = create_fintech_policy();
        
        // Input missing required fields
        let incomplete_input = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            // Missing: destination_account, initiator_id, timestamp
        });

        let result = create_secure_context_advanced(&incomplete_input, &policy, "fintech_transfer");
        
        match result {
            ContextBuildResult::MissingRequiredFields(missing) => {
                assert!(missing.contains(&"destination_account".to_string()));
                assert!(missing.contains(&"initiator_id".to_string()));
                assert!(missing.contains(&"timestamp".to_string()));
            }
            _ => panic!("Expected MissingRequiredFields result"),
        }
    }

    #[test]
    fn test_advanced_context_builder_success() {
        let policy = create_fintech_policy();
        
        // Clean input that should pass all checks
        let clean_input = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            "destination_account": "ACME-123",
            "initiator_id": "user-456",
            "timestamp": 1678886400
        });

        let result = create_secure_context_advanced(&clean_input, &policy, "fintech_transfer");
        
        match result {
            ContextBuildResult::Success(context) => {
                assert_eq!(context["action"], "wire_transfer");
                assert_eq!(context["amount_usd_cents"], 1000000);
                assert_eq!(context["destination_account"], "ACME-123");
                assert_eq!(context["initiator_id"], "user-456");
                assert_eq!(context["timestamp"], 1678886400);
            }
            _ => panic!("Expected Success result, got: {:?}", result),
        }
    }

    #[test]
    fn test_context_compliance_validation() {
        let policy = create_fintech_policy();
        
        // Valid context
        let valid_context = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            "destination_account": "ACME-123",
            "initiator_id": "user-456",
            "timestamp": 1678886400
        });

        let errors = validate_context_compliance(&valid_context, &policy);
        assert!(errors.is_empty(), "Valid context should have no errors: {:?}", errors);

        // Invalid context with forbidden field
        let invalid_context = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            "destination_account": "ACME-123",
            "initiator_id": "user-456",
            "timestamp": 1678886400,
            "user_ip": "192.168.1.100"  // Forbidden field
        });

        let errors = validate_context_compliance(&invalid_context, &policy);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("forbidden field")));
    }

    #[test]
    fn test_sanitize_existing_context() {
        let policy = create_fintech_policy();
        
        // Context with extra fields that should be removed
        let dirty_context = json!({
            "action": "wire_transfer",
            "amount_usd_cents": 1000000,
            "destination_account": "ACME-123",
            "initiator_id": "user-456",
            "timestamp": 1678886400,
            "user_ip": "192.168.1.100",     // Should be removed
            "session_id": "session-123",    // Should be removed
            "extra_field": "should_be_removed"  // Should be removed
        });

        let sanitized = sanitize_existing_context(&dirty_context, &policy);
        
        // Should only contain policy-approved fields
        assert_eq!(sanitized["action"], "wire_transfer");
        assert_eq!(sanitized["amount_usd_cents"], 1000000);
        assert_eq!(sanitized["destination_account"], "ACME-123");
        assert_eq!(sanitized["initiator_id"], "user-456");
        assert_eq!(sanitized["timestamp"], 1678886400);
        
        // Should not contain forbidden or extra fields
        assert!(sanitized.get("user_ip").is_none());
        assert!(sanitized.get("session_id").is_none());
        assert!(sanitized.get("extra_field").is_none());
    }

    #[test]
    fn test_multiple_context_types() {
        // Test different context types with their specific policies
        
        // Biometric context
        let biometric_policy = create_biometric_policy();
        let biometric_input = json!({
            "action": "biometric_approval",
            "user_id": "user-789",
            "timestamp": 1678886400,
            "device_attestation": "platform-authenticator",
            "biometric_template": "should-be-removed"  // Forbidden
        });

        let result = create_secure_context_advanced(&biometric_input, &biometric_policy, "biometric_auth");
        match result {
            ContextBuildResult::PolicyViolation(_) => {
                // Expected - biometric_template is forbidden
            }
            _ => panic!("Expected policy violation for biometric template"),
        }

        // Audit context
        let audit_policy = create_audit_policy();
        let audit_input = json!({
            "action": "audit_log",
            "event_type": "user_login",
            "timestamp": 1678886400,
            "user_id": "user-789",
            "resource": "proof_verification_service"
        });

        let result = create_secure_context_advanced(&audit_input, &audit_policy, "audit_event");
        match result {
            ContextBuildResult::Success(context) => {
                assert_eq!(context["action"], "audit_log");
                assert_eq!(context["event_type"], "user_login");
            }
            _ => panic!("Expected success for valid audit context"),
        }
    }
}