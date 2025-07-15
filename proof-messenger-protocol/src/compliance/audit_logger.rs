// src/compliance/audit_logger.rs
//! Compliance Audit Logger
//! 
//! This module provides comprehensive audit logging for compliance operations,
//! ensuring that all data sanitization activities are properly tracked and
//! can be reviewed for compliance audits.

use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::compliance::pii_detector::PIIType;

/// Audit event types for compliance tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    SanitizationAttempt,
    SanitizationSuccess,
    SanitizationFailure,
    PolicyViolation,
    PIIDetection,
    ContextValidation,
    PolicyApplication,
    ComplianceCheck,
}

/// Audit log entry for compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub context_type: String,
    pub event_details: HashMap<String, Value>,
    pub risk_level: String,
    pub compliance_status: String,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        context_type: String,
        event_details: HashMap<String, Value>,
        risk_level: String,
        compliance_status: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            context_type,
            event_details,
            risk_level,
            compliance_status,
            session_id: None,
            user_id: None,
        }
    }

    /// Set session ID for tracking
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set user ID for tracking
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Compliance audit logger
pub struct ComplianceAuditLogger {
    entries: Vec<AuditLogEntry>,
    session_id: Option<String>,
    user_id: Option<String>,
}

impl ComplianceAuditLogger {
    /// Create a new compliance audit logger
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            session_id: None,
            user_id: None,
        }
    }

    /// Set session ID for all subsequent log entries
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Set user ID for all subsequent log entries
    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    /// Log a sanitization attempt
    pub fn log_sanitization_attempt(&mut self, context_type: &str, raw_input: &Value) {
        let mut details = HashMap::new();
        details.insert("input_size".to_string(), Value::Number(serde_json::Number::from(
            raw_input.to_string().len()
        )));
        details.insert("input_type".to_string(), Value::String(
            match raw_input {
                Value::Object(_) => "object",
                Value::Array(_) => "array",
                Value::String(_) => "string",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::Null => "null",
            }.to_string()
        ));

        // Count fields if it's an object
        if let Value::Object(obj) = raw_input {
            details.insert("field_count".to_string(), Value::Number(serde_json::Number::from(obj.len())));
            details.insert("field_names".to_string(), Value::Array(
                obj.keys().map(|k| Value::String(k.clone())).collect()
            ));
        }

        let entry = AuditLogEntry::new(
            AuditEventType::SanitizationAttempt,
            context_type.to_string(),
            details,
            "INFO".to_string(),
            "IN_PROGRESS".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log a successful sanitization
    pub fn log_sanitization_success(&mut self, context_type: &str, clean_output: &Value) {
        let mut details = HashMap::new();
        details.insert("output_size".to_string(), Value::Number(serde_json::Number::from(
            clean_output.to_string().len()
        )));

        // Count fields if it's an object
        if let Value::Object(obj) = clean_output {
            details.insert("clean_field_count".to_string(), Value::Number(serde_json::Number::from(obj.len())));
            details.insert("clean_field_names".to_string(), Value::Array(
                obj.keys().map(|k| Value::String(k.clone())).collect()
            ));
        }

        let entry = AuditLogEntry::new(
            AuditEventType::SanitizationSuccess,
            context_type.to_string(),
            details,
            "INFO".to_string(),
            "COMPLIANT".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log a sanitization failure
    pub fn log_sanitization_failure(&mut self, context_type: &str, failure_reason: &str) {
        let mut details = HashMap::new();
        details.insert("failure_reason".to_string(), Value::String(failure_reason.to_string()));

        let entry = AuditLogEntry::new(
            AuditEventType::SanitizationFailure,
            context_type.to_string(),
            details,
            "ERROR".to_string(),
            "NON_COMPLIANT".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log a policy violation
    pub fn log_policy_violation(&mut self, context_type: &str, field_name: &str, violation_type: &str) {
        let mut details = HashMap::new();
        details.insert("field_name".to_string(), Value::String(field_name.to_string()));
        details.insert("violation_type".to_string(), Value::String(violation_type.to_string()));

        let entry = AuditLogEntry::new(
            AuditEventType::PolicyViolation,
            context_type.to_string(),
            details,
            "WARNING".to_string(),
            "POLICY_VIOLATION".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log PII detection
    pub fn log_pii_detection(&mut self, context_type: &str, field_name: &str, pii_types: &[PIIType]) {
        let mut details = HashMap::new();
        details.insert("field_name".to_string(), Value::String(field_name.to_string()));
        details.insert("pii_types".to_string(), Value::Array(
            pii_types.iter().map(|pii| Value::String(format!("{:?}", pii))).collect()
        ));
        details.insert("pii_count".to_string(), Value::Number(serde_json::Number::from(pii_types.len())));

        // Determine risk level based on PII types
        let risk_level = if pii_types.iter().any(|pii| pii.risk_level() == crate::compliance::pii_detector::PIIRiskLevel::Critical) {
            "CRITICAL"
        } else if pii_types.iter().any(|pii| pii.risk_level() == crate::compliance::pii_detector::PIIRiskLevel::High) {
            "HIGH"
        } else {
            "MEDIUM"
        };

        let entry = AuditLogEntry::new(
            AuditEventType::PIIDetection,
            context_type.to_string(),
            details,
            risk_level.to_string(),
            "PII_DETECTED".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log context validation
    pub fn log_context_validation(&mut self, context_type: &str, validation_result: &str, errors: &[String]) {
        let mut details = HashMap::new();
        details.insert("validation_result".to_string(), Value::String(validation_result.to_string()));
        details.insert("error_count".to_string(), Value::Number(serde_json::Number::from(errors.len())));
        
        if !errors.is_empty() {
            details.insert("errors".to_string(), Value::Array(
                errors.iter().map(|e| Value::String(e.clone())).collect()
            ));
        }

        let (risk_level, compliance_status) = if errors.is_empty() {
            ("INFO", "VALID")
        } else {
            ("WARNING", "VALIDATION_FAILED")
        };

        let entry = AuditLogEntry::new(
            AuditEventType::ContextValidation,
            context_type.to_string(),
            details,
            risk_level.to_string(),
            compliance_status.to_string(),
        );

        self.add_entry(entry);
    }

    /// Log policy application
    pub fn log_policy_application(&mut self, context_type: &str, policy_version: &str, fields_removed: &[String]) {
        let mut details = HashMap::new();
        details.insert("policy_version".to_string(), Value::String(policy_version.to_string()));
        details.insert("fields_removed_count".to_string(), Value::Number(serde_json::Number::from(fields_removed.len())));
        
        if !fields_removed.is_empty() {
            details.insert("fields_removed".to_string(), Value::Array(
                fields_removed.iter().map(|f| Value::String(f.clone())).collect()
            ));
        }

        let entry = AuditLogEntry::new(
            AuditEventType::PolicyApplication,
            context_type.to_string(),
            details,
            "INFO".to_string(),
            "POLICY_APPLIED".to_string(),
        );

        self.add_entry(entry);
    }

    /// Log compliance check
    pub fn log_compliance_check(&mut self, context_type: &str, check_type: &str, result: bool, details: HashMap<String, Value>) {
        let mut entry_details = details;
        entry_details.insert("check_type".to_string(), Value::String(check_type.to_string()));
        entry_details.insert("check_result".to_string(), Value::Bool(result));

        let (risk_level, compliance_status) = if result {
            ("INFO", "COMPLIANT")
        } else {
            ("ERROR", "NON_COMPLIANT")
        };

        let entry = AuditLogEntry::new(
            AuditEventType::ComplianceCheck,
            context_type.to_string(),
            entry_details,
            risk_level.to_string(),
            compliance_status.to_string(),
        );

        self.add_entry(entry);
    }

    /// Add an entry to the audit log
    fn add_entry(&mut self, mut entry: AuditLogEntry) {
        // Add session and user IDs if available
        if let Some(ref session_id) = self.session_id {
            entry.session_id = Some(session_id.clone());
        }
        if let Some(ref user_id) = self.user_id {
            entry.user_id = Some(user_id.clone());
        }

        self.entries.push(entry);
    }

    /// Get all audit log entries
    pub fn get_entries(&self) -> &[AuditLogEntry] {
        &self.entries
    }

    /// Get entries by event type
    pub fn get_entries_by_type(&self, event_type: AuditEventType) -> Vec<&AuditLogEntry> {
        self.entries.iter().filter(|entry| entry.event_type == event_type).collect()
    }

    /// Get entries by risk level
    pub fn get_entries_by_risk_level(&self, risk_level: &str) -> Vec<&AuditLogEntry> {
        self.entries.iter().filter(|entry| entry.risk_level == risk_level).collect()
    }

    /// Get entries by compliance status
    pub fn get_entries_by_compliance_status(&self, status: &str) -> Vec<&AuditLogEntry> {
        self.entries.iter().filter(|entry| entry.compliance_status == status).collect()
    }

    /// Generate compliance summary
    pub fn generate_compliance_summary(&self) -> ComplianceSummary {
        let total_entries = self.entries.len();
        let mut event_counts = HashMap::new();
        let mut risk_level_counts = HashMap::new();
        let mut compliance_status_counts = HashMap::new();

        for entry in &self.entries {
            *event_counts.entry(entry.event_type.clone()).or_insert(0) += 1;
            *risk_level_counts.entry(entry.risk_level.clone()).or_insert(0) += 1;
            *compliance_status_counts.entry(entry.compliance_status.clone()).or_insert(0) += 1;
        }

        ComplianceSummary {
            total_entries,
            event_counts,
            risk_level_counts,
            compliance_status_counts,
            generated_at: Utc::now(),
        }
    }

    /// Export audit log as JSON
    pub fn export_as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }

    /// Clear all audit log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for ComplianceAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Compliance summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_entries: usize,
    pub event_counts: HashMap<AuditEventType, usize>,
    pub risk_level_counts: HashMap<String, usize>,
    pub compliance_status_counts: HashMap<String, usize>,
    pub generated_at: DateTime<Utc>,
}

impl ComplianceSummary {
    /// Check if there are any critical compliance issues
    pub fn has_critical_issues(&self) -> bool {
        self.risk_level_counts.get("CRITICAL").unwrap_or(&0) > &0 ||
        self.compliance_status_counts.get("NON_COMPLIANT").unwrap_or(&0) > &0
    }

    /// Get compliance score (0-100)
    pub fn get_compliance_score(&self) -> f64 {
        if self.total_entries == 0 {
            return 100.0;
        }

        let compliant_entries = self.compliance_status_counts.get("COMPLIANT").unwrap_or(&0) +
                               self.compliance_status_counts.get("VALID").unwrap_or(&0) +
                               self.compliance_status_counts.get("POLICY_APPLIED").unwrap_or(&0);

        (compliant_entries as f64 / self.total_entries as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_logger_creation() {
        let logger = ComplianceAuditLogger::new();
        assert_eq!(logger.entry_count(), 0);
        assert!(logger.session_id.is_none());
        assert!(logger.user_id.is_none());
    }

    #[test]
    fn test_sanitization_attempt_logging() {
        let mut logger = ComplianceAuditLogger::new();
        let input = json!({
            "action": "wire_transfer",
            "amount": 1000000,
            "user_ip": "192.168.1.100"
        });

        logger.log_sanitization_attempt("fintech_transfer", &input);
        
        assert_eq!(logger.entry_count(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, AuditEventType::SanitizationAttempt);
        assert_eq!(entries[0].context_type, "fintech_transfer");
        assert_eq!(entries[0].compliance_status, "IN_PROGRESS");
    }

    #[test]
    fn test_sanitization_success_logging() {
        let mut logger = ComplianceAuditLogger::new();
        let output = json!({
            "action": "wire_transfer",
            "amount": 1000000
        });

        logger.log_sanitization_success("fintech_transfer", &output);
        
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, AuditEventType::SanitizationSuccess);
        assert_eq!(entries[0].compliance_status, "COMPLIANT");
    }

    #[test]
    fn test_policy_violation_logging() {
        let mut logger = ComplianceAuditLogger::new();
        
        logger.log_policy_violation("fintech_transfer", "user_ip", "forbidden_field");
        
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, AuditEventType::PolicyViolation);
        assert_eq!(entries[0].compliance_status, "POLICY_VIOLATION");
        assert_eq!(entries[0].risk_level, "WARNING");
    }

    #[test]
    fn test_pii_detection_logging() {
        let mut logger = ComplianceAuditLogger::new();
        let pii_types = vec![PIIType::EmailAddress, PIIType::SocialSecurityNumber];
        
        logger.log_pii_detection("fintech_transfer", "user_email", &pii_types);
        
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, AuditEventType::PIIDetection);
        assert_eq!(entries[0].compliance_status, "PII_DETECTED");
        assert_eq!(entries[0].risk_level, "CRITICAL"); // SSN is critical
    }

    #[test]
    fn test_context_validation_logging() {
        let mut logger = ComplianceAuditLogger::new();
        let errors = vec!["Missing required field: timestamp".to_string()];
        
        logger.log_context_validation("fintech_transfer", "FAILED", &errors);
        
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, AuditEventType::ContextValidation);
        assert_eq!(entries[0].compliance_status, "VALIDATION_FAILED");
    }

    #[test]
    fn test_session_and_user_tracking() {
        let mut logger = ComplianceAuditLogger::new();
        logger.set_session_id("session-123".to_string());
        logger.set_user_id("user-456".to_string());
        
        logger.log_sanitization_attempt("fintech_transfer", &json!({}));
        
        let entries = logger.get_entries();
        assert_eq!(entries[0].session_id, Some("session-123".to_string()));
        assert_eq!(entries[0].user_id, Some("user-456".to_string()));
    }

    #[test]
    fn test_entries_filtering() {
        let mut logger = ComplianceAuditLogger::new();
        
        logger.log_sanitization_attempt("fintech_transfer", &json!({}));
        logger.log_policy_violation("fintech_transfer", "user_ip", "forbidden_field");
        logger.log_sanitization_success("fintech_transfer", &json!({}));
        
        let policy_violations = logger.get_entries_by_type(AuditEventType::PolicyViolation);
        assert_eq!(policy_violations.len(), 1);
        
        let warning_entries = logger.get_entries_by_risk_level("WARNING");
        assert_eq!(warning_entries.len(), 1);
        
        let compliant_entries = logger.get_entries_by_compliance_status("COMPLIANT");
        assert_eq!(compliant_entries.len(), 1);
    }

    #[test]
    fn test_compliance_summary() {
        let mut logger = ComplianceAuditLogger::new();
        
        logger.log_sanitization_success("fintech_transfer", &json!({}));
        logger.log_policy_violation("fintech_transfer", "user_ip", "forbidden_field");
        logger.log_pii_detection("fintech_transfer", "ssn", &vec![PIIType::SocialSecurityNumber]);
        
        let summary = logger.generate_compliance_summary();
        
        assert_eq!(summary.total_entries, 3);
        assert!(summary.has_critical_issues()); // Due to SSN detection
        assert!(summary.get_compliance_score() < 100.0);
        
        // Check event counts
        assert_eq!(summary.event_counts.get(&AuditEventType::SanitizationSuccess), Some(&1));
        assert_eq!(summary.event_counts.get(&AuditEventType::PolicyViolation), Some(&1));
        assert_eq!(summary.event_counts.get(&AuditEventType::PIIDetection), Some(&1));
    }

    #[test]
    fn test_json_export() {
        let mut logger = ComplianceAuditLogger::new();
        logger.log_sanitization_success("fintech_transfer", &json!({}));
        
        let json_export = logger.export_as_json().unwrap();
        assert!(json_export.contains("SanitizationSuccess"));
        assert!(json_export.contains("fintech_transfer"));
        assert!(json_export.contains("COMPLIANT"));
    }

    #[test]
    fn test_compliance_score_calculation() {
        let mut logger = ComplianceAuditLogger::new();
        
        // All compliant entries
        logger.log_sanitization_success("fintech_transfer", &json!({}));
        logger.log_context_validation("fintech_transfer", "PASSED", &vec![]);
        
        let summary = logger.generate_compliance_summary();
        assert_eq!(summary.get_compliance_score(), 100.0);
        
        // Add a non-compliant entry
        logger.log_policy_violation("fintech_transfer", "user_ip", "forbidden_field");
        
        let summary = logger.generate_compliance_summary();
        assert!(summary.get_compliance_score() < 100.0);
        assert!(summary.get_compliance_score() > 0.0);
    }

    #[test]
    fn test_clear_and_entry_count() {
        let mut logger = ComplianceAuditLogger::new();
        
        logger.log_sanitization_success("fintech_transfer", &json!({}));
        assert_eq!(logger.entry_count(), 1);
        
        logger.clear();
        assert_eq!(logger.entry_count(), 0);
    }
}