// src/compliance/data_policies.rs
//! Data Policy Definitions
//! 
//! This module defines strict data policies for different contexts,
//! specifying exactly what data is required, optional, and forbidden.
//! This implements the "Define the Data Policy (The Test)" step of the TDD workflow.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Data policy defining what fields are allowed, required, and forbidden
/// for a specific context type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataPolicy {
    /// Fields that must be present in the context
    pub required_fields: HashSet<String>,
    /// Fields that may be present in the context
    pub optional_fields: HashSet<String>,
    /// Fields that must never be present in the context (PII, sensitive data)
    pub forbidden_fields: HashSet<String>,
    /// Human-readable description of this policy
    pub description: String,
    /// Policy version for auditing and compliance tracking
    pub version: String,
}

impl DataPolicy {
    /// Create a new data policy
    pub fn new(
        required_fields: Vec<String>,
        optional_fields: Vec<String>,
        forbidden_fields: Vec<String>,
        description: String,
        version: String,
    ) -> Self {
        Self {
            required_fields: required_fields.into_iter().collect(),
            optional_fields: optional_fields.into_iter().collect(),
            forbidden_fields: forbidden_fields.into_iter().collect(),
            description,
            version,
        }
    }

    /// Check if a field is allowed in this policy
    pub fn is_field_allowed(&self, field: &str) -> bool {
        !self.forbidden_fields.contains(field) && 
        (self.required_fields.contains(field) || self.optional_fields.contains(field))
    }

    /// Check if a field is required in this policy
    pub fn is_field_required(&self, field: &str) -> bool {
        self.required_fields.contains(field)
    }

    /// Check if a field is forbidden in this policy
    pub fn is_field_forbidden(&self, field: &str) -> bool {
        self.forbidden_fields.contains(field)
    }

    /// Get all allowed fields (required + optional)
    pub fn get_allowed_fields(&self) -> HashSet<String> {
        self.required_fields.union(&self.optional_fields).cloned().collect()
    }
}

/// Policy for FinTech wire transfer contexts
/// 
/// Example Policy for "FinTech Wire Transfer":
/// - Required Fields: action, amount_usd_cents, destination_account, initiator_id, timestamp
/// - Forbidden Fields: user_ip, session_id, user_agent, device_id, or any other PII
pub fn create_fintech_policy() -> DataPolicy {
    DataPolicy::new(
        vec![
            "action".to_string(),
            "amount_usd_cents".to_string(),
            "destination_account".to_string(),
            "initiator_id".to_string(),
            "timestamp".to_string(),
        ],
        vec![
            "transaction_id".to_string(),
            "reference_number".to_string(),
            "currency".to_string(),
        ],
        vec![
            // Network and session PII
            "user_ip".to_string(),
            "session_id".to_string(),
            "user_agent".to_string(),
            "device_id".to_string(),
            "client_ip".to_string(),
            "x_forwarded_for".to_string(),
            "x_real_ip".to_string(),
            
            // Personal identifiers
            "email".to_string(),
            "phone_number".to_string(),
            "ssn".to_string(),
            "tax_id".to_string(),
            "passport_number".to_string(),
            "drivers_license".to_string(),
            
            // Biometric data
            "fingerprint_template".to_string(),
            "face_encoding".to_string(),
            "voice_print".to_string(),
            "biometric_template".to_string(),
            
            // Device and location PII
            "device_serial".to_string(),
            "mac_address".to_string(),
            "imei".to_string(),
            "gps_coordinates".to_string(),
            "location_data".to_string(),
            
            // Authentication secrets
            "password".to_string(),
            "password_hash".to_string(),
            "api_key".to_string(),
            "private_key".to_string(),
            "secret_key".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
        ],
        "FinTech wire transfer context policy - ensures only business-critical data is included".to_string(),
        "1.0.0".to_string(),
    )
}

/// Policy for biometric authentication contexts
/// 
/// Biometric contexts should never contain actual biometric data,
/// only attestations and metadata about the authentication process
pub fn create_biometric_policy() -> DataPolicy {
    DataPolicy::new(
        vec![
            "action".to_string(),
            "user_id".to_string(),
            "timestamp".to_string(),
            "device_attestation".to_string(),
        ],
        vec![
            "authenticator_type".to_string(),
            "challenge_id".to_string(),
            "origin".to_string(),
            "rp_id".to_string(),
        ],
        vec![
            // Biometric data (absolutely forbidden)
            "fingerprint_template".to_string(),
            "face_encoding".to_string(),
            "voice_print".to_string(),
            "biometric_template".to_string(),
            "biometric_data".to_string(),
            "raw_biometric".to_string(),
            "biometric_hash".to_string(),
            
            // Device identifiers
            "device_id".to_string(),
            "device_serial".to_string(),
            "mac_address".to_string(),
            "imei".to_string(),
            "hardware_id".to_string(),
            
            // Network and session data
            "user_ip".to_string(),
            "session_id".to_string(),
            "user_agent".to_string(),
            "client_ip".to_string(),
            
            // Personal identifiers
            "email".to_string(),
            "phone_number".to_string(),
            "full_name".to_string(),
            
            // Location data
            "gps_coordinates".to_string(),
            "location_data".to_string(),
            "geolocation".to_string(),
        ],
        "Biometric authentication context policy - strictly prohibits biometric data".to_string(),
        "1.0.0".to_string(),
    )
}

/// Policy for audit log contexts
/// 
/// Audit logs need specific information for compliance but should
/// avoid detailed PII that could be used for tracking
pub fn create_audit_policy() -> DataPolicy {
    DataPolicy::new(
        vec![
            "action".to_string(),
            "event_type".to_string(),
            "timestamp".to_string(),
            "user_id".to_string(),
            "resource".to_string(),
        ],
        vec![
            "outcome".to_string(),
            "error_code".to_string(),
            "request_id".to_string(),
            "service_name".to_string(),
        ],
        vec![
            // Detailed network information
            "user_ip".to_string(),
            "client_ip".to_string(),
            "x_forwarded_for".to_string(),
            "user_agent".to_string(),
            
            // Session and device data
            "session_id".to_string(),
            "device_id".to_string(),
            "device_fingerprint".to_string(),
            
            // Personal data
            "email".to_string(),
            "phone_number".to_string(),
            "full_name".to_string(),
            
            // Sensitive request/response data
            "request_body".to_string(),
            "response_body".to_string(),
            "password".to_string(),
            "api_key".to_string(),
            "access_token".to_string(),
        ],
        "Audit log context policy - balances compliance needs with privacy protection".to_string(),
        "1.0.0".to_string(),
    )
}

/// Policy for login authentication contexts
/// 
/// Login contexts should contain minimal information needed for
/// authentication proof without exposing sensitive credentials
pub fn create_login_policy() -> DataPolicy {
    DataPolicy::new(
        vec![
            "action".to_string(),
            "user_id".to_string(),
            "timestamp".to_string(),
            "authentication_method".to_string(),
        ],
        vec![
            "challenge_id".to_string(),
            "origin".to_string(),
            "client_version".to_string(),
        ],
        vec![
            // Credentials and secrets
            "password".to_string(),
            "password_hash".to_string(),
            "private_key".to_string(),
            "secret_key".to_string(),
            "api_key".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
            "mfa_secret".to_string(),
            "totp_secret".to_string(),
            
            // Network and device tracking
            "user_ip".to_string(),
            "session_id".to_string(),
            "device_id".to_string(),
            "device_fingerprint".to_string(),
            "user_agent".to_string(),
            
            // Personal identifiers
            "email".to_string(),
            "phone_number".to_string(),
            "full_name".to_string(),
            
            // Biometric data
            "fingerprint_template".to_string(),
            "face_encoding".to_string(),
            "biometric_template".to_string(),
        ],
        "Login authentication context policy - minimal data for authentication proof".to_string(),
        "1.0.0".to_string(),
    )
}

/// Policy for transaction approval contexts
/// 
/// Transaction approvals need business data but should exclude
/// detailed user tracking information
pub fn create_transaction_policy() -> DataPolicy {
    DataPolicy::new(
        vec![
            "action".to_string(),
            "transaction_type".to_string(),
            "amount".to_string(),
            "currency".to_string(),
            "initiator_id".to_string(),
            "timestamp".to_string(),
        ],
        vec![
            "transaction_id".to_string(),
            "reference_number".to_string(),
            "destination_account".to_string(),
            "source_account".to_string(),
            "approval_method".to_string(),
        ],
        vec![
            // Network tracking
            "user_ip".to_string(),
            "session_id".to_string(),
            "user_agent".to_string(),
            "client_ip".to_string(),
            
            // Device identification
            "device_id".to_string(),
            "device_serial".to_string(),
            "mac_address".to_string(),
            "device_fingerprint".to_string(),
            
            // Personal data
            "email".to_string(),
            "phone_number".to_string(),
            "full_name".to_string(),
            "address".to_string(),
            
            // Location data
            "gps_coordinates".to_string(),
            "location_data".to_string(),
            "geolocation".to_string(),
            
            // Authentication secrets
            "password".to_string(),
            "private_key".to_string(),
            "api_key".to_string(),
            "access_token".to_string(),
        ],
        "Transaction approval context policy - business data without user tracking".to_string(),
        "1.0.0".to_string(),
    )
}

/// Get policy by context type name
pub fn get_policy_by_type(context_type: &str) -> Option<DataPolicy> {
    match context_type {
        "fintech_transfer" | "wire_transfer" => Some(create_fintech_policy()),
        "biometric_auth" | "biometric_approval" => Some(create_biometric_policy()),
        "audit_log" | "audit_event" => Some(create_audit_policy()),
        "login" | "authentication" => Some(create_login_policy()),
        "transaction" | "transaction_approval" => Some(create_transaction_policy()),
        _ => None,
    }
}

/// Registry of all available policies
pub struct PolicyRegistry {
    policies: std::collections::HashMap<String, DataPolicy>,
}

impl PolicyRegistry {
    /// Create a new policy registry with all standard policies
    pub fn new() -> Self {
        let mut policies = std::collections::HashMap::new();
        
        policies.insert("fintech_transfer".to_string(), create_fintech_policy());
        policies.insert("biometric_auth".to_string(), create_biometric_policy());
        policies.insert("audit_log".to_string(), create_audit_policy());
        policies.insert("login".to_string(), create_login_policy());
        policies.insert("transaction".to_string(), create_transaction_policy());
        
        Self { policies }
    }
    
    /// Get a policy by type
    pub fn get_policy(&self, policy_type: &str) -> Option<&DataPolicy> {
        self.policies.get(policy_type)
    }
    
    /// Register a custom policy
    pub fn register_policy(&mut self, policy_type: String, policy: DataPolicy) {
        self.policies.insert(policy_type, policy);
    }
    
    /// List all available policy types
    pub fn list_policy_types(&self) -> Vec<String> {
        self.policies.keys().cloned().collect()
    }
}

impl Default for PolicyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fintech_policy_structure() {
        let policy = create_fintech_policy();
        
        // Check required fields
        assert!(policy.is_field_required("action"));
        assert!(policy.is_field_required("amount_usd_cents"));
        assert!(policy.is_field_required("destination_account"));
        assert!(policy.is_field_required("initiator_id"));
        assert!(policy.is_field_required("timestamp"));
        
        // Check forbidden fields
        assert!(policy.is_field_forbidden("user_ip"));
        assert!(policy.is_field_forbidden("session_id"));
        assert!(policy.is_field_forbidden("user_agent"));
        assert!(policy.is_field_forbidden("device_id"));
        assert!(policy.is_field_forbidden("email"));
        assert!(policy.is_field_forbidden("fingerprint_template"));
        
        // Check optional fields
        assert!(policy.is_field_allowed("transaction_id"));
        assert!(policy.is_field_allowed("reference_number"));
        assert!(!policy.is_field_required("transaction_id"));
    }

    #[test]
    fn test_biometric_policy_structure() {
        let policy = create_biometric_policy();
        
        // Check required fields
        assert!(policy.is_field_required("action"));
        assert!(policy.is_field_required("user_id"));
        assert!(policy.is_field_required("timestamp"));
        assert!(policy.is_field_required("device_attestation"));
        
        // Check that biometric data is strictly forbidden
        assert!(policy.is_field_forbidden("fingerprint_template"));
        assert!(policy.is_field_forbidden("face_encoding"));
        assert!(policy.is_field_forbidden("voice_print"));
        assert!(policy.is_field_forbidden("biometric_template"));
        assert!(policy.is_field_forbidden("biometric_data"));
        
        // Check optional fields
        assert!(policy.is_field_allowed("authenticator_type"));
        assert!(policy.is_field_allowed("challenge_id"));
    }

    #[test]
    fn test_audit_policy_structure() {
        let policy = create_audit_policy();
        
        // Check required fields for audit compliance
        assert!(policy.is_field_required("action"));
        assert!(policy.is_field_required("event_type"));
        assert!(policy.is_field_required("timestamp"));
        assert!(policy.is_field_required("user_id"));
        assert!(policy.is_field_required("resource"));
        
        // Check that detailed tracking data is forbidden
        assert!(policy.is_field_forbidden("user_ip"));
        assert!(policy.is_field_forbidden("session_id"));
        assert!(policy.is_field_forbidden("request_body"));
        assert!(policy.is_field_forbidden("response_body"));
    }

    #[test]
    fn test_policy_registry() {
        let registry = PolicyRegistry::new();
        
        // Test that all standard policies are available
        assert!(registry.get_policy("fintech_transfer").is_some());
        assert!(registry.get_policy("biometric_auth").is_some());
        assert!(registry.get_policy("audit_log").is_some());
        assert!(registry.get_policy("login").is_some());
        assert!(registry.get_policy("transaction").is_some());
        
        // Test that unknown policy returns None
        assert!(registry.get_policy("unknown_policy").is_none());
        
        // Test policy types listing
        let policy_types = registry.list_policy_types();
        assert!(policy_types.contains(&"fintech_transfer".to_string()));
        assert!(policy_types.contains(&"biometric_auth".to_string()));
    }

    #[test]
    fn test_custom_policy_registration() {
        let mut registry = PolicyRegistry::new();
        
        // Create a custom policy
        let custom_policy = DataPolicy::new(
            vec!["custom_field".to_string()],
            vec!["optional_field".to_string()],
            vec!["forbidden_field".to_string()],
            "Custom test policy".to_string(),
            "1.0.0".to_string(),
        );
        
        // Register the custom policy
        registry.register_policy("custom_policy".to_string(), custom_policy);
        
        // Verify it's available
        let retrieved_policy = registry.get_policy("custom_policy").unwrap();
        assert!(retrieved_policy.is_field_required("custom_field"));
        assert!(retrieved_policy.is_field_forbidden("forbidden_field"));
    }

    #[test]
    fn test_get_policy_by_type() {
        // Test standard policy type mappings
        assert!(get_policy_by_type("fintech_transfer").is_some());
        assert!(get_policy_by_type("wire_transfer").is_some());
        assert!(get_policy_by_type("biometric_auth").is_some());
        assert!(get_policy_by_type("biometric_approval").is_some());
        assert!(get_policy_by_type("audit_log").is_some());
        assert!(get_policy_by_type("audit_event").is_some());
        
        // Test unknown type
        assert!(get_policy_by_type("unknown_type").is_none());
    }

    #[test]
    fn test_policy_field_operations() {
        let policy = create_fintech_policy();
        
        // Test allowed fields collection
        let allowed_fields = policy.get_allowed_fields();
        assert!(allowed_fields.contains("action"));
        assert!(allowed_fields.contains("amount_usd_cents"));
        assert!(allowed_fields.contains("transaction_id")); // Optional field
        assert!(!allowed_fields.contains("user_ip")); // Forbidden field
        
        // Test field status checks
        assert!(policy.is_field_allowed("action"));
        assert!(!policy.is_field_allowed("user_ip"));
        assert!(policy.is_field_required("action"));
        assert!(!policy.is_field_required("transaction_id"));
        assert!(policy.is_field_forbidden("user_ip"));
        assert!(!policy.is_field_forbidden("action"));
    }
}