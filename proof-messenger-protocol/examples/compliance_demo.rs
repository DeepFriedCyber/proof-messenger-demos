// examples/compliance_demo.rs
//! Compliance Check Module Demo
//! 
//! This demo shows the TDD workflow for compliance:
//! 1. Define the Data Policy (The "Test") - Strict policy for required/forbidden fields
//! 2. Write the Test Case - Test with "dirty" data containing PII
//! 3. Implement the Sanitizer - Function that enforces policy and passes tests
//!
//! Strategic Principle: "Shift Left on Compliance"
//! Build data correctly and securely from the very beginning, ensuring sensitive 
//! PII never touches core application logic or logs.

use proof_messenger_protocol::compliance::{
    create_secure_context, create_secure_context_advanced,
    sanitize_existing_context, ContextBuildResult, DataPolicy, PIIDetector, 
    ComplianceAuditLogger, create_fintech_policy, create_biometric_policy,
    create_audit_policy, PolicyRegistry
};
use serde_json::json;


fn main() {
    println!("🔒 Compliance Check Module Demo");
    println!("Strategic Principle: 'Shift Left on Compliance'");
    println!("===============================================\n");

    // Demo 1: Basic TDD Workflow for FinTech Context
    demo_basic_tdd_workflow();
    
    // Demo 2: Advanced Compliance with PII Detection
    demo_advanced_compliance_checking();
    
    // Demo 3: Multiple Context Types
    demo_multiple_context_types();
    
    // Demo 4: Policy Registry and Management
    demo_policy_registry();
    
    // Demo 5: Audit Logging and Compliance Reporting
    demo_audit_logging();
    
    // Demo 6: Real-world Scenarios
    demo_real_world_scenarios();

    println!("\n🎉 Compliance Demo Complete!");
    println!("Key Benefits:");
    println!("  ✅ PII never touches core application logic");
    println!("  ✅ Policy violations caught at data creation");
    println!("  ✅ Comprehensive audit trail for compliance");
    println!("  ✅ Automated sanitization prevents data leaks");
    println!("  ✅ TDD workflow ensures policy correctness");
}

fn demo_basic_tdd_workflow() {
    println!("📋 Demo 1: Basic TDD Workflow for FinTech Context");
    println!("================================================");
    
    // Step 1: Define the Data Policy (The "Test")
    println!("Step 1: Define the Data Policy");
    let fintech_policy = vec![
        "action".to_string(),
        "amount_usd_cents".to_string(),
        "destination_account".to_string(),
    ];
    println!("  Required fields: {:?}", fintech_policy);
    
    // Step 2: Create "dirty" input with PII
    println!("\nStep 2: Create 'dirty' input with PII");
    let dirty_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 5000000,
        "destination_account": "ACME-123",
        "user_ip": "192.168.1.100",    // PII to be stripped
        "session_id": "abc-xyz-789",   // PII to be stripped
        "user_email": "user@example.com", // PII to be stripped
        "device_id": "device-12345"    // PII to be stripped
    });
    println!("  Dirty input: {}", serde_json::to_string_pretty(&dirty_input).unwrap());
    
    // Step 3: Apply sanitization
    println!("\nStep 3: Apply sanitization");
    let clean_context = create_secure_context(&dirty_input, &fintech_policy);
    println!("  Clean output: {}", serde_json::to_string_pretty(&clean_context).unwrap());
    
    // Step 4: Verify expected result
    println!("\nStep 4: Verify expected result");
    let expected_clean_context = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 5000000,
        "destination_account": "ACME-123"
    });
    
    if clean_context == expected_clean_context {
        println!("  ✅ SUCCESS: Sanitization worked correctly!");
        println!("  ✅ PII successfully removed from context");
        println!("  ✅ Only business-critical data remains");
    } else {
        println!("  ❌ FAILURE: Sanitization did not work as expected");
    }
    
    println!("\n{}\n", "=".repeat(60));
}

fn demo_advanced_compliance_checking() {
    println!("🔍 Demo 2: Advanced Compliance with PII Detection");
    println!("=================================================");
    
    let policy = create_fintech_policy();
    let mut audit_logger = ComplianceAuditLogger::new();
    audit_logger.set_session_id("demo-session-123".to_string());
    
    // Test case 1: Input with forbidden fields
    println!("Test Case 1: Input with forbidden fields");
    let input_with_forbidden = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        "user_ip": "192.168.1.100",        // Forbidden
        "session_id": "session-abc-123",   // Forbidden
        "email": "user@example.com"        // Forbidden
    });
    
    let result = create_secure_context_advanced(&input_with_forbidden, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            println!("  ❌ Policy violations detected:");
            for violation in violations {
                println!("    • {}", violation);
            }
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    // Test case 2: Input with PII in values
    println!("\nTest Case 2: Input with PII in values");
    let input_with_pii = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "user@example.com", // Email in destination
        "initiator_id": "123-45-6789",             // SSN as user ID
        "timestamp": 1678886400
    });
    
    let result = create_secure_context_advanced(&input_with_pii, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::PIIDetected(pii_issues) => {
            println!("  ⚠️  PII detected in field values:");
            for issue in pii_issues {
                println!("    • {}", issue);
            }
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    // Test case 3: Valid input
    println!("\nTest Case 3: Valid input");
    let valid_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400
    });
    
    let result = create_secure_context_advanced(&valid_input, &policy, "fintech_transfer");
    match result {
        ContextBuildResult::Success(context) => {
            println!("  ✅ SUCCESS: Valid context created");
            println!("  Clean context: {}", serde_json::to_string_pretty(&context).unwrap());
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    println!("\n{}\n", "=".repeat(60));
}

fn demo_multiple_context_types() {
    println!("🎭 Demo 3: Multiple Context Types");
    println!("=================================");
    
    // Biometric authentication context
    println!("Biometric Authentication Context:");
    let biometric_policy = create_biometric_policy();
    let biometric_input = json!({
        "action": "biometric_approval",
        "user_id": "user-789",
        "timestamp": 1678886400,
        "device_attestation": "platform-authenticator",
        "fingerprint_template": "base64-encoded-template", // Forbidden!
        "challenge_id": "challenge-123"
    });
    
    let result = create_secure_context_advanced(&biometric_input, &biometric_policy, "biometric_auth");
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            println!("  ❌ Biometric policy violations (expected):");
            for violation in violations {
                println!("    • {}", violation);
            }
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    // Clean biometric input
    let clean_biometric_input = json!({
        "action": "biometric_approval",
        "user_id": "user-789",
        "timestamp": 1678886400,
        "device_attestation": "platform-authenticator",
        "challenge_id": "challenge-123"
    });
    
    let result = create_secure_context_advanced(&clean_biometric_input, &biometric_policy, "biometric_auth");
    match result {
        ContextBuildResult::Success(context) => {
            println!("  ✅ Clean biometric context created:");
            println!("    {}", serde_json::to_string_pretty(&context).unwrap());
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    // Audit log context
    println!("\nAudit Log Context:");
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
            println!("  ✅ Audit context created:");
            println!("    {}", serde_json::to_string_pretty(&context).unwrap());
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    println!("\n{}\n", "=".repeat(60));
}

fn demo_policy_registry() {
    println!("📚 Demo 4: Policy Registry and Management");
    println!("=========================================");
    
    let mut registry = PolicyRegistry::new();
    
    println!("Available policy types:");
    for policy_type in registry.list_policy_types() {
        println!("  • {}", policy_type);
    }
    
    // Create a custom policy
    println!("\nCreating custom policy for 'user_registration':");
    let custom_policy = DataPolicy::new(
        vec![
            "action".to_string(),
            "username".to_string(),
            "timestamp".to_string(),
        ],
        vec![
            "referral_code".to_string(),
        ],
        vec![
            "email".to_string(),
            "password".to_string(),
            "ip_address".to_string(),
            "user_agent".to_string(),
        ],
        "User registration context policy".to_string(),
        "1.0.0".to_string(),
    );
    
    registry.register_policy("user_registration".to_string(), custom_policy);
    
    // Test the custom policy
    let registration_input = json!({
        "action": "user_registration",
        "username": "john_doe",
        "timestamp": 1678886400,
        "email": "john@example.com",  // Should be forbidden
        "referral_code": "REF123"     // Optional field
    });
    
    let policy = registry.get_policy("user_registration").unwrap();
    let result = create_secure_context_advanced(&registration_input, policy, "user_registration");
    
    match result {
        ContextBuildResult::PolicyViolation(violations) => {
            println!("  ❌ Custom policy violations:");
            for violation in violations {
                println!("    • {}", violation);
            }
        }
        _ => println!("  Unexpected result: {:?}", result),
    }
    
    println!("\n{}\n", "=".repeat(60));
}

fn demo_audit_logging() {
    println!("📊 Demo 5: Audit Logging and Compliance Reporting");
    println!("==================================================");
    
    let mut audit_logger = ComplianceAuditLogger::new();
    audit_logger.set_session_id("audit-demo-session".to_string());
    audit_logger.set_user_id("demo-user-123".to_string());
    
    let policy = create_fintech_policy();
    
    // Simulate various compliance operations
    println!("Simulating compliance operations...");
    
    // Successful sanitization
    let clean_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400
    });
    
    let _result = create_secure_context_advanced(&clean_input, &policy, "fintech_transfer");
    
    // Policy violation
    let dirty_input = json!({
        "action": "wire_transfer",
        "amount_usd_cents": 1000000,
        "destination_account": "ACME-123",
        "initiator_id": "user-456",
        "timestamp": 1678886400,
        "user_ip": "192.168.1.100"
    });
    
    let _result = create_secure_context_advanced(&dirty_input, &policy, "fintech_transfer");
    
    // Generate compliance summary
    println!("\nCompliance Summary:");
    let summary = audit_logger.generate_compliance_summary();
    println!("  Total audit entries: {}", summary.total_entries);
    println!("  Compliance score: {:.1}%", summary.get_compliance_score());
    println!("  Has critical issues: {}", summary.has_critical_issues());
    
    println!("\nEvent type breakdown:");
    for (event_type, count) in &summary.event_counts {
        println!("  {:?}: {}", event_type, count);
    }
    
    println!("\nRisk level breakdown:");
    for (risk_level, count) in &summary.risk_level_counts {
        println!("  {}: {}", risk_level, count);
    }
    
    println!("\nCompliance status breakdown:");
    for (status, count) in &summary.compliance_status_counts {
        println!("  {}: {}", status, count);
    }
    
    println!("\n{}\n", "=".repeat(60));
}

fn demo_real_world_scenarios() {
    println!("🌍 Demo 6: Real-world Scenarios");
    println!("===============================");
    
    let pii_detector = PIIDetector::new();
    
    // Scenario 1: Processing user registration data
    println!("Scenario 1: User Registration Data Processing");
    let registration_data = json!({
        "username": "john_doe",
        "email": "john.doe@example.com",
        "phone": "(555) 123-4567",
        "password": "hashed_password_here",
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "timestamp": 1678886400,
        "referral_code": "REF123"
    });
    
    println!("  Original data contains PII:");
    if let Some(pii_result) = pii_detector.detect_pii_detailed(&registration_data) {
        println!("    Risk level: {:?}", pii_result.highest_risk_level);
        for detail in pii_result.details {
            println!("    • {}", detail);
        }
    }
    
    // Apply sanitization for audit logging
    let audit_policy = create_audit_policy();
    let sanitized_for_audit = sanitize_existing_context(&registration_data, &audit_policy);
    println!("  Sanitized for audit logging:");
    println!("    {}", serde_json::to_string_pretty(&sanitized_for_audit).unwrap());
    
    // Scenario 2: Financial transaction processing
    println!("\nScenario 2: Financial Transaction Processing");
    let transaction_data = json!({
        "transaction_id": "txn-12345",
        "from_account": "user@example.com",  // PII in business data
        "to_account": "ACME-CORP-123",
        "amount": 50000,
        "currency": "USD",
        "timestamp": 1678886400,
        "user_ip": "192.168.1.100",
        "session_id": "sess-abc-123",
        "device_fingerprint": "fp-xyz-789"
    });
    
    let fintech_policy = create_fintech_policy();
    
    // Check for PII in business fields
    println!("  Checking for PII in transaction data...");
    if pii_detector.contains_high_risk_pii(&transaction_data) {
        println!("    ⚠️  High-risk PII detected in transaction data!");
    }
    
    // Sanitize for proof generation
    let clean_transaction = sanitize_existing_context(&transaction_data, &fintech_policy);
    println!("  Sanitized for proof generation:");
    println!("    {}", serde_json::to_string_pretty(&clean_transaction).unwrap());
    
    // Scenario 3: Biometric authentication flow
    println!("\nScenario 3: Biometric Authentication Flow");
    let biometric_data = json!({
        "user_id": "user-789",
        "authentication_method": "webauthn",
        "device_attestation": "platform-authenticator",
        "challenge": "challenge-abc-123",
        "origin": "https://example.com",
        "fingerprint_template": "base64-biometric-data", // Critical PII!
        "device_serial": "DEVICE-12345",
        "timestamp": 1678886400
    });
    
    println!("  Checking biometric data for critical PII...");
    if pii_detector.contains_critical_pii(&biometric_data) {
        println!("    🚨 CRITICAL PII detected! This data must not be logged or stored!");
    }
    
    let biometric_policy = create_biometric_policy();
    let safe_biometric_context = sanitize_existing_context(&biometric_data, &biometric_policy);
    println!("  Safe biometric context (no PII):");
    println!("    {}", serde_json::to_string_pretty(&safe_biometric_context).unwrap());
    
    println!("\n🔒 Key Security Benefits Demonstrated:");
    println!("  ✅ PII automatically detected and flagged");
    println!("  ✅ Context-specific policies prevent data leaks");
    println!("  ✅ Critical biometric data never reaches application logic");
    println!("  ✅ Business data separated from tracking data");
    println!("  ✅ Compliance requirements enforced at data creation");
    
    println!("\n{}\n", "=".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions_run_without_panic() {
        // These tests ensure the demo functions don't panic
        // In a real application, you'd have more specific assertions
        
        demo_basic_tdd_workflow();
        demo_advanced_compliance_checking();
        demo_multiple_context_types();
        demo_policy_registry();
        demo_audit_logging();
        demo_real_world_scenarios();
    }
}