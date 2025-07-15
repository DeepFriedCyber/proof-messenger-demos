// tests/compliance_tdd_tests.rs
//! Compliance TDD Test Suite
//! 
//! This test suite demonstrates the complete TDD workflow for compliance:
//! 1. Define the Data Policy (The "Test") - Strict policy requirements
//! 2. Write the Test Case - Tests with "dirty" data containing PII
//! 3. Implement the Sanitizer - Functions that enforce policy and pass tests
//!
//! These tests serve as both validation and documentation of the compliance system.

use proof_messenger_protocol::compliance::{
    create_secure_context, create_secure_context_advanced, validate_context_compliance,
    sanitize_existing_context, ContextBuildResult, DataPolicy, PIIDetector,
    create_fintech_policy, create_biometric_policy, create_audit_policy,
    create_login_policy, create_transaction_policy, PolicyRegistry
};
use serde_json::json;

/// Test the original TDD workflow example from the specification
#[test]
fn test_fintech_context_sanitization_tdd_workflow() {
    // 1. Define the policy for this specific context (The "Test")
    let fintech_policy = vec![
        "action".to_string(),
        "amount_usd_cents".to_string(),
        "destination_account".to_string(),
    ];

    // 2. Create a "dirty" input object with extra PII (The "Implementation")
    let dirty_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 5000000,
        "destination_account": "ACME-123",
        "user_ip": "192.168.1.100",    // PII to be stripped
        "session_id": "abc-xyz-789"    // PII to be stripped
    });

    // 3. Define the expected "clean" output (The "Assertion")
    let expected_clean_context = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 5000000,
        "destination_account": "ACME-123"
    });

    // 4. Call the function and assert that the output is correct
    let clean_context = create_secure_context(&dirty_input, &fintech_policy);
    assert_eq!(clean_context, expected_clean_context);
}

/// Test comprehensive FinTech policy with all forbidden fields
#[test]
fn test_comprehensive_fintech_policy_enforcement() {
    let policy = create_fintech_policy();
    
    // Test input with multiple types of forbidden PII
    let dirty_input = json!({
        // Required fields
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        
        // Optional fields (should be kept)
        "transaction_id": "txn-789",
        "reference_number": "REF-123",
        
        // Network and session PII (should be removed)
        "user_ip": "192.168.1.100",
        "session_id": "session-abc-123",
        "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        "device_id": "device-xyz-789",
        "client_ip": "10.0.0.1",
        
        // Personal identifiers (should be removed)
        "email": "user@example.com",
        "phone_number": "(555) 123-4567",
        "ssn": "123-45-6789",
        
        // Biometric data (should be removed)
        "fingerprint_template": "base64-encoded-template",
        "face_encoding": "biometric-data-here",
        
        // Authentication secrets (should be removed)
        "password": "hashed-password",
        "api_key": "api-key-12345",
        "access_token": "token-abc-123"
    });

    let result = create_secure_context_advanced(&dirty_input, &policy, "fintech_transfer");
    
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            // Should detect multiple policy violations
            assert!(violations.len() >= 10); // At least 10 forbidden fields
            
            // Check that specific forbidden fields are detected
            assert!(violations.iter().any(|v| v.contains("user_ip")));
            assert!(violations.iter().any(|v| v.contains("session_id")));
            assert!(violations.iter().any(|v| v.contains("email")));
            assert!(violations.iter().any(|v| v.contains("fingerprint_template")));
            assert!(violations.iter().any(|v| v.contains("password")));
        }
        _ => panic!("Expected PolicyViolation result, got: {:?}", result),
    }
}

/// Test biometric policy strictly prohibits biometric data
#[test]
fn test_biometric_policy_prohibits_biometric_data() {
    let policy = create_biometric_policy();
    
    // Test input with various types of biometric data (all forbidden)
    let biometric_input = json!({
        // Required fields
        "action": "biometric_approval",
        "user_id": "user-789",
        "timestamp": 1678886400,
        "device_attestation": "platform-authenticator",
        
        // Optional fields (should be kept)
        "authenticator_type": "webauthn",
        "challenge_id": "challenge-123",
        
        // Biometric data (absolutely forbidden)
        "fingerprint_template": "base64-fingerprint-data",
        "face_encoding": "face-biometric-data",
        "voice_print": "voice-biometric-data",
        "biometric_template": "generic-biometric-template",
        "biometric_data": "raw-biometric-data",
        "raw_biometric": "raw-data",
        "biometric_hash": "hashed-biometric-data",
        
        // Device identifiers (forbidden)
        "device_id": "device-12345",
        "device_serial": "SERIAL-ABC-123",
        "mac_address": "00:11:22:33:44:55",
        
        // Network data (forbidden)
        "user_ip": "192.168.1.100",
        "session_id": "session-xyz-789"
    });

    let result = create_secure_context_advanced(&biometric_input, &policy, "biometric_auth");
    
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            // Should detect all biometric data as forbidden
            assert!(violations.iter().any(|v| v.contains("fingerprint_template")));
            assert!(violations.iter().any(|v| v.contains("face_encoding")));
            assert!(violations.iter().any(|v| v.contains("voice_print")));
            assert!(violations.iter().any(|v| v.contains("biometric_template")));
            assert!(violations.iter().any(|v| v.contains("biometric_data")));
            assert!(violations.iter().any(|v| v.contains("device_id")));
            assert!(violations.iter().any(|v| v.contains("user_ip")));
        }
        _ => panic!("Expected PolicyViolation result for biometric data, got: {:?}", result),
    }
}

/// Test PII detection in field values
#[test]
fn test_pii_detection_in_field_values() {
    let policy = create_fintech_policy();
    
    // Test input with PII in allowed field values
    let pii_in_values = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "user@example.com", // Email in business field
        "initiator_id": "123-45-6789",             // SSN as user ID
        "timestamp": 1678886400
    });

    let result = create_secure_context_advanced(&pii_in_values, &policy, "fintech_transfer");
    
    match result {
        ContextBuildResult::PIIDetected(pii_issues) => {
            // Should detect PII in field values
            assert!(pii_issues.iter().any(|issue| issue.contains("destination_account")));
            assert!(pii_issues.iter().any(|issue| issue.contains("initiator_id")));
            assert!(pii_issues.iter().any(|issue| issue.contains("EmailAddress")));
            assert!(pii_issues.iter().any(|issue| issue.contains("SocialSecurityNumber")));
        }
        _ => panic!("Expected PIIDetected result, got: {:?}", result),
    }
}

/// Test missing required fields detection
#[test]
fn test_missing_required_fields_detection() {
    let policy = create_fintech_policy();
    
    // Test input missing required fields
    let incomplete_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        // Missing: destination_account, initiator_id, timestamp
        "optional_field": "some_value"
    });

    let result = create_secure_context_advanced(&incomplete_input, &policy, "fintech_transfer");
    
    match result {
        ContextBuildResult::MissingRequiredFields(missing) => {
            assert!(missing.contains(&"destination_account".to_string()));
            assert!(missing.contains(&"initiator_id".to_string()));
            assert!(missing.contains(&"timestamp".to_string()));
        }
        _ => panic!("Expected MissingRequiredFields result, got: {:?}", result),
    }
}

/// Test successful context creation with valid data
#[test]
fn test_successful_context_creation() {
    let policy = create_fintech_policy();
    
    // Test input with all required fields and no forbidden data
    let valid_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        "transaction_id": "txn-789",  // Optional field
        "currency": "USD"             // Optional field
    });

    let result = create_secure_context_advanced(&valid_input, &policy, "fintech_transfer");
    
    match result {
        ContextBuildResult::Success(context) => {
            // Should contain all required fields
            assert_eq!(context["action"], "wire_transfer");
            assert_eq!(context["amount_usd_cents"], 1000000);
            assert_eq!(context["destination_account"], "ACME-123");
            assert_eq!(context["initiator_id"], "user-456");
            assert_eq!(context["timestamp"], 1678886400);
            
            // Should contain optional fields
            assert_eq!(context["transaction_id"], "txn-789");
            assert_eq!(context["currency"], "USD");
            
            // Should not contain any other fields
            assert!(context.as_object().unwrap().len() == 7);
        }
        _ => panic!("Expected Success result, got: {:?}", result),
    }
}

/// Test context validation function
#[test]
fn test_context_validation() {
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
    assert!(errors.iter().any(|e| e.contains("user_ip")));

    // Context missing required fields
    let incomplete_context = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000
        // Missing required fields
    });

    let errors = validate_context_compliance(&incomplete_context, &policy);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.contains("missing required field")));
}

/// Test context sanitization function
#[test]
fn test_context_sanitization() {
    let policy = create_fintech_policy();
    
    // Context with extra fields that should be removed
    let dirty_context = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        "transaction_id": "txn-789",        // Optional - should be kept
        "user_ip": "192.168.1.100",         // Forbidden - should be removed
        "session_id": "session-123",        // Forbidden - should be removed
        "unknown_field": "should_be_removed" // Unknown - should be removed
    });

    let sanitized = sanitize_existing_context(&dirty_context, &policy);
    
    // Should contain all required fields
    assert_eq!(sanitized["action"], "wire_transfer");
    assert_eq!(sanitized["amount_usd_cents"], 1000000);
    assert_eq!(sanitized["destination_account"], "ACME-123");
    assert_eq!(sanitized["initiator_id"], "user-456");
    assert_eq!(sanitized["timestamp"], 1678886400);
    
    // Should contain optional fields
    assert_eq!(sanitized["transaction_id"], "txn-789");
    
    // Should not contain forbidden or unknown fields
    assert!(sanitized.get("user_ip").is_none());
    assert!(sanitized.get("session_id").is_none());
    assert!(sanitized.get("unknown_field").is_none());
    
    // Should only contain expected fields
    assert_eq!(sanitized.as_object().unwrap().len(), 6);
}

/// Test multiple context types with their specific policies
#[test]
fn test_multiple_context_types() {
    // Test login policy
    let login_policy = create_login_policy();
    let login_input = json!({
        "action": "login",
        "user_id": "user-123",
        "timestamp": 1678886400,
        "authentication_method": "webauthn",
        "password": "should-be-forbidden"  // Forbidden in login context
    });

    let result = create_secure_context_advanced(&login_input, &login_policy, "login");
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            assert!(violations.iter().any(|v| v.contains("password")));
        }
        _ => panic!("Expected policy violation for password in login context"),
    }

    // Test transaction policy
    let transaction_policy = create_transaction_policy();
    let transaction_input = json!({
        "action": "transaction_approval",
        "transaction_type": "transfer",
        "amount": 50000,
        "currency": "USD",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        "user_ip": "192.168.1.100"  // Forbidden in transaction context
    });

    let result = create_secure_context_advanced(&transaction_input, &transaction_policy, "transaction");
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            assert!(violations.iter().any(|v| v.contains("user_ip")));
        }
        _ => panic!("Expected policy violation for user_ip in transaction context"),
    }

    // Test audit policy
    let audit_policy = create_audit_policy();
    let audit_input = json!({
        "action": "audit_log",
        "event_type": "user_login",
        "timestamp": 1678886400,
        "user_id": "user-789",
        "resource": "proof_verification_service",
        "outcome": "success"
    });

    let result = create_secure_context_advanced(&audit_input, &audit_policy, "audit_event");
    match result {
        ContextBuildResult::Success(context) => {
            assert_eq!(context["action"], "audit_log");
            assert_eq!(context["event_type"], "user_login");
            assert_eq!(context["outcome"], "success");
        }
        _ => panic!("Expected success for valid audit context"),
    }
}

/// Test policy registry functionality
#[test]
fn test_policy_registry() {
    let mut registry = PolicyRegistry::new();
    
    // Test that standard policies are available
    assert!(registry.get_policy("fintech_transfer").is_some());
    assert!(registry.get_policy("biometric_auth").is_some());
    assert!(registry.get_policy("audit_log").is_some());
    assert!(registry.get_policy("login").is_some());
    assert!(registry.get_policy("transaction").is_some());
    
    // Test that unknown policy returns None
    assert!(registry.get_policy("unknown_policy").is_none());
    
    // Test custom policy registration
    let custom_policy = DataPolicy::new(
        vec!["required_field".to_string()],
        vec!["optional_field".to_string()],
        vec!["forbidden_field".to_string()],
        "Custom test policy".to_string(),
        "1.0.0".to_string(),
    );
    
    registry.register_policy("custom_policy".to_string(), custom_policy);
    
    let retrieved_policy = registry.get_policy("custom_policy").unwrap();
    assert!(retrieved_policy.is_field_required("required_field"));
    assert!(retrieved_policy.is_field_allowed("optional_field"));
    assert!(retrieved_policy.is_field_forbidden("forbidden_field"));
    
    // Test policy types listing
    let policy_types = registry.list_policy_types();
    assert!(policy_types.contains(&"fintech_transfer".to_string()));
    assert!(policy_types.contains(&"custom_policy".to_string()));
}

/// Test PII detector functionality
#[test]
fn test_pii_detector() {
    let detector = PIIDetector::new();
    
    // Test various PII types
    let test_cases = vec![
        (json!("user@example.com"), "EmailAddress"),
        (json!("(555) 123-4567"), "PhoneNumber"),
        (json!("123-45-6789"), "SocialSecurityNumber"),
        (json!("192.168.1.100"), "IPAddress"),
        (json!("550e8400-e29b-41d4-a716-446655440000"), "UUID"),
    ];
    
    for (value, expected_pii_type) in test_cases {
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(!pii_types.is_empty(), "Should detect PII in: {}", value);
        
        let pii_names: Vec<String> = pii_types.iter().map(|pii| format!("{:?}", pii)).collect();
        assert!(pii_names.iter().any(|name| name.contains(expected_pii_type)), 
                "Should detect {} in: {}", expected_pii_type, value);
    }
    
    // Test nested object PII detection
    let nested_value = json!({
        "user": {
            "profile": {
                "contact": "user@example.com"
            }
        },
        "metadata": {
            "client_ip": "192.168.1.100"
        }
    });
    
    let pii_types = detector.detect_pii(&nested_value).unwrap();
    assert!(pii_types.len() >= 2); // Should detect both email and IP
    
    // Test no PII detection
    let safe_value = json!({
        "action": "wire_transfer",
        "amount": 1000000,
        "timestamp": 1678886400
    });
    
    let pii_result = detector.detect_pii(&safe_value);
    assert!(pii_result.is_none(), "Should not detect PII in safe data");
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases() {
    let policy = create_fintech_policy();
    
    // Test with null input
    let null_input = json!(null);
    let result = create_secure_context_advanced(&null_input, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::PolicyViolation(_) => {
            // Expected - null is not a valid object
        }
        _ => panic!("Expected policy violation for null input"),
    }
    
    // Test with array input
    let array_input = json!(["not", "an", "object"]);
    let result = create_secure_context_advanced(&array_input, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::PolicyViolation(_) => {
            // Expected - array is not a valid object
        }
        _ => panic!("Expected policy violation for array input"),
    }
    
    // Test with empty object
    let empty_input = json!({});
    let result = create_secure_context_advanced(&empty_input, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::MissingRequiredFields(missing) => {
            assert!(missing.len() == policy.required_fields.len());
        }
        _ => panic!("Expected missing required fields for empty input"),
    }
}

/// Integration test: Complete workflow from dirty input to clean context
#[test]
fn test_complete_compliance_workflow() {
    let policy = create_fintech_policy();
    
    // Step 1: Start with dirty input containing various types of PII
    let dirty_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 5000000,
        "destination_account": "ACME-123",
        "initiator_id": "usr456",
        "timestamp": 1678886400,
        "transaction_id": "txn789",
        
        // Various types of PII that should be removed
        "user_ip": "192.168.1.100",
        "session_id": "session-abc-123",
        "user_agent": "Mozilla/5.0...",
        "device_id": "device-xyz-789",
        "email": "user@example.com",
        "phone_number": "(555) 123-4567",
        "fingerprint_template": "base64-biometric-data",
        "password": "hashed-password",
        "api_key": "api-key-12345"
    });
    
    // Step 2: Apply advanced sanitization
    let result = create_secure_context_advanced(&dirty_input, &policy, "fintech_transfer");
    
    // Step 3: Should fail due to policy violations
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            assert!(violations.len() >= 8); // Multiple violations expected
        }
        _ => panic!("Expected policy violations for dirty input"),
    }
    
    // Step 4: Use basic sanitization to get clean context
    let clean_context = create_secure_context(&dirty_input, &vec![
        "action".to_string(),
        "amount_usd_cents".to_string(),
        "destination_account".to_string(),
        "initiator_id".to_string(),
        "timestamp".to_string(),
        "transaction_id".to_string(),
    ]);
    
    // Step 5: Verify clean context contains only allowed fields
    assert_eq!(clean_context["action"], "wire_transfer");
    assert_eq!(clean_context["amount_usd_cents"], 5000000);
    assert_eq!(clean_context["destination_account"], "ACME-123");
    assert_eq!(clean_context["initiator_id"], "usr456");
    assert_eq!(clean_context["timestamp"], 1678886400);
    assert_eq!(clean_context["transaction_id"], "txn789");
    
    // Step 6: Verify no PII remains
    assert!(clean_context.get("user_ip").is_none());
    assert!(clean_context.get("session_id").is_none());
    assert!(clean_context.get("email").is_none());
    assert!(clean_context.get("fingerprint_template").is_none());
    assert!(clean_context.get("password").is_none());
    
    // Step 7: Validate the clean context against policy
    let validation_errors = validate_context_compliance(&clean_context, &policy);
    assert!(validation_errors.is_empty(), "Clean context should be valid: {:?}", validation_errors);
    
    // Step 8: Verify PII detector finds no PII in clean context
    let pii_detector = PIIDetector::new();
    let pii_result = pii_detector.detect_pii(&clean_context);
    if let Some(pii_types) = &pii_result {
        println!("Clean context: {}", serde_json::to_string_pretty(&clean_context).unwrap());
        println!("PII detected: {:?}", pii_types);
    }
    assert!(pii_result.is_none(), "Clean context should contain no PII, but found: {:?}", pii_result);
}