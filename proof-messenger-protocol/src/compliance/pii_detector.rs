// src/compliance/pii_detector.rs
//! PII Detection Module
//! 
//! This module provides advanced PII detection capabilities to identify
//! personally identifiable information in data values, ensuring that
//! sensitive data is caught even if it appears in unexpected places.

use serde_json::Value;
use regex::Regex;
use std::collections::HashSet;

/// Types of PII that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PIIType {
    EmailAddress,
    PhoneNumber,
    SocialSecurityNumber,
    CreditCardNumber,
    IPAddress,
    MacAddress,
    UUID,
    Base64EncodedData,
    JWTToken,
    APIKey,
    PasswordHash,
    BiometricTemplate,
    DeviceSerial,
    SessionToken,
    PersonalName,
    Address,
    DateOfBirth,
    TaxID,
    PassportNumber,
    DriversLicense,
}

impl PIIType {
    /// Get a human-readable description of the PII type
    pub fn description(&self) -> &'static str {
        match self {
            PIIType::EmailAddress => "Email address",
            PIIType::PhoneNumber => "Phone number",
            PIIType::SocialSecurityNumber => "Social Security Number",
            PIIType::CreditCardNumber => "Credit card number",
            PIIType::IPAddress => "IP address",
            PIIType::MacAddress => "MAC address",
            PIIType::UUID => "UUID identifier",
            PIIType::Base64EncodedData => "Base64 encoded data (potential PII)",
            PIIType::JWTToken => "JWT token",
            PIIType::APIKey => "API key",
            PIIType::PasswordHash => "Password hash",
            PIIType::BiometricTemplate => "Biometric template",
            PIIType::DeviceSerial => "Device serial number",
            PIIType::SessionToken => "Session token",
            PIIType::PersonalName => "Personal name",
            PIIType::Address => "Physical address",
            PIIType::DateOfBirth => "Date of birth",
            PIIType::TaxID => "Tax identification number",
            PIIType::PassportNumber => "Passport number",
            PIIType::DriversLicense => "Driver's license number",
        }
    }

    /// Get the risk level of this PII type
    pub fn risk_level(&self) -> PIIRiskLevel {
        match self {
            PIIType::BiometricTemplate | PIIType::SocialSecurityNumber | PIIType::CreditCardNumber => PIIRiskLevel::Critical,
            PIIType::EmailAddress | PIIType::PhoneNumber | PIIType::PersonalName | PIIType::Address => PIIRiskLevel::High,
            PIIType::IPAddress | PIIType::DeviceSerial | PIIType::SessionToken | PIIType::JWTToken => PIIRiskLevel::Medium,
            PIIType::UUID | PIIType::Base64EncodedData | PIIType::APIKey => PIIRiskLevel::Low,
            _ => PIIRiskLevel::Medium,
        }
    }
}

/// Risk levels for different types of PII
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PIIRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// PII detection result
#[derive(Debug, Clone)]
pub struct PIIDetectionResult {
    pub pii_types: HashSet<PIIType>,
    pub highest_risk_level: PIIRiskLevel,
    pub details: Vec<String>,
}

/// PII detector with configurable patterns and rules
pub struct PIIDetector {
    email_regex: Regex,
    phone_regex: Regex,
    ssn_regex: Regex,
    credit_card_regex: Regex,
    ip_address_regex: Regex,
    mac_address_regex: Regex,
    uuid_regex: Regex,
    base64_regex: Regex,
    jwt_regex: Regex,
    api_key_regex: Regex,
    password_hash_regex: Regex,
    session_token_regex: Regex,
    device_serial_regex: Regex,
    personal_name_patterns: Vec<Regex>,
    address_patterns: Vec<Regex>,
    date_patterns: Vec<Regex>,
}

impl PIIDetector {
    /// Create a new PII detector with default patterns
    pub fn new() -> Self {
        Self {
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
            phone_regex: Regex::new(r"(\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})").unwrap(),
            ssn_regex: Regex::new(r"\b\d{3}-?\d{2}-?\d{4}\b").unwrap(),
            credit_card_regex: Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap(),
            ip_address_regex: Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(),
            mac_address_regex: Regex::new(r"\b([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})\b").unwrap(),
            uuid_regex: Regex::new(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b").unwrap(),
            base64_regex: Regex::new(r"\b[A-Za-z0-9+/]{20,}={0,2}\b").unwrap(),
            jwt_regex: Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").unwrap(),
            api_key_regex: Regex::new(r"\b[A-Za-z0-9]{32,}\b").unwrap(),
            password_hash_regex: Regex::new(r"\$2[aby]?\$\d+\$[./A-Za-z0-9]{53}").unwrap(),
            session_token_regex: Regex::new(r"\b[A-Za-z0-9]{40,}\b").unwrap(),
            device_serial_regex: Regex::new(r"\b[A-Z0-9]{8,20}\b").unwrap(),
            personal_name_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+\b").unwrap(),
                Regex::new(r"\b[A-Z][a-z]+ [A-Z]\. [A-Z][a-z]+\b").unwrap(),
            ],
            address_patterns: vec![
                Regex::new(r"\d+\s+[A-Za-z\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr)").unwrap(),
                Regex::new(r"\b\d{5}(?:-\d{4})?\b").unwrap(), // ZIP codes
            ],
            date_patterns: vec![
                Regex::new(r"\b\d{1,2}/\d{1,2}/\d{4}\b").unwrap(),
                Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap(),
            ],
        }
    }

    /// Detect PII in a JSON value
    pub fn detect_pii(&self, value: &Value) -> Option<HashSet<PIIType>> {
        let mut detected_pii = HashSet::new();

        match value {
            Value::String(s) => {
                detected_pii.extend(self.detect_pii_in_string(s));
            }
            Value::Array(arr) => {
                for item in arr {
                    if let Some(pii) = self.detect_pii(item) {
                        detected_pii.extend(pii);
                    }
                }
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    // Check key names for PII indicators
                    detected_pii.extend(self.detect_pii_in_field_name(key));
                    
                    // Check values
                    if let Some(pii) = self.detect_pii(val) {
                        detected_pii.extend(pii);
                    }
                }
            }
            _ => {} // Numbers, booleans, null don't contain PII patterns
        }

        if detected_pii.is_empty() {
            None
        } else {
            Some(detected_pii)
        }
    }

    /// Detect PII in a string value
    fn detect_pii_in_string(&self, s: &str) -> HashSet<PIIType> {
        let mut detected = HashSet::new();

        // Email addresses
        if self.email_regex.is_match(s) {
            detected.insert(PIIType::EmailAddress);
        }

        // Phone numbers
        if self.phone_regex.is_match(s) {
            detected.insert(PIIType::PhoneNumber);
        }

        // Social Security Numbers
        if self.ssn_regex.is_match(s) {
            detected.insert(PIIType::SocialSecurityNumber);
        }

        // Credit card numbers
        if self.credit_card_regex.is_match(s) && self.is_valid_credit_card(s) {
            detected.insert(PIIType::CreditCardNumber);
        }

        // IP addresses
        if self.ip_address_regex.is_match(s) {
            detected.insert(PIIType::IPAddress);
        }

        // MAC addresses
        if self.mac_address_regex.is_match(s) {
            detected.insert(PIIType::MacAddress);
        }

        // UUIDs
        if self.uuid_regex.is_match(s) {
            detected.insert(PIIType::UUID);
        }

        // Base64 encoded data (potential PII)
        if self.base64_regex.is_match(s) && s.len() > 50 {
            detected.insert(PIIType::Base64EncodedData);
        }

        // JWT tokens
        if self.jwt_regex.is_match(s) {
            detected.insert(PIIType::JWTToken);
        }

        // API keys (long alphanumeric strings)
        if self.api_key_regex.is_match(s) && s.len() >= 32 {
            detected.insert(PIIType::APIKey);
        }

        // Password hashes
        if self.password_hash_regex.is_match(s) {
            detected.insert(PIIType::PasswordHash);
        }

        // Session tokens
        if self.session_token_regex.is_match(s) && s.len() >= 40 {
            detected.insert(PIIType::SessionToken);
        }

        // Device serial numbers
        if self.device_serial_regex.is_match(s) && s.len() >= 8 {
            detected.insert(PIIType::DeviceSerial);
        }

        // Personal names
        for pattern in &self.personal_name_patterns {
            if pattern.is_match(s) {
                detected.insert(PIIType::PersonalName);
                break;
            }
        }

        // Addresses
        for pattern in &self.address_patterns {
            if pattern.is_match(s) {
                detected.insert(PIIType::Address);
                break;
            }
        }

        // Dates (potential DOB)
        for pattern in &self.date_patterns {
            if pattern.is_match(s) {
                detected.insert(PIIType::DateOfBirth);
                break;
            }
        }

        detected
    }

    /// Detect PII based on field names
    fn detect_pii_in_field_name(&self, field_name: &str) -> HashSet<PIIType> {
        let mut detected = HashSet::new();
        let field_lower = field_name.to_lowercase();

        // Check for PII-indicating field names
        if field_lower.contains("email") {
            detected.insert(PIIType::EmailAddress);
        }
        if field_lower.contains("phone") || field_lower.contains("mobile") {
            detected.insert(PIIType::PhoneNumber);
        }
        if field_lower.contains("ssn") || field_lower.contains("social_security") {
            detected.insert(PIIType::SocialSecurityNumber);
        }
        if field_lower.contains("credit_card") || field_lower.contains("card_number") {
            detected.insert(PIIType::CreditCardNumber);
        }
        if field_lower.contains("ip_address") || field_lower.contains("client_ip") {
            detected.insert(PIIType::IPAddress);
        }
        if field_lower.contains("mac_address") || field_lower.contains("hardware_address") {
            detected.insert(PIIType::MacAddress);
        }
        if field_lower.contains("biometric") || field_lower.contains("fingerprint") || field_lower.contains("face_encoding") {
            detected.insert(PIIType::BiometricTemplate);
        }
        if field_lower.contains("device_id") || field_lower.contains("device_serial") {
            detected.insert(PIIType::DeviceSerial);
        }
        if field_lower.contains("session") && (field_lower.contains("id") || field_lower.contains("token")) {
            detected.insert(PIIType::SessionToken);
        }
        if field_lower.contains("password") {
            detected.insert(PIIType::PasswordHash);
        }
        if field_lower.contains("api_key") || field_lower.contains("access_token") {
            detected.insert(PIIType::APIKey);
        }
        if field_lower.contains("name") && !field_lower.contains("username") && !field_lower.contains("service_name") {
            detected.insert(PIIType::PersonalName);
        }
        if field_lower.contains("address") || field_lower.contains("location") {
            detected.insert(PIIType::Address);
        }
        if field_lower.contains("birth") || field_lower.contains("dob") {
            detected.insert(PIIType::DateOfBirth);
        }
        if field_lower == "tax_id" || field_lower == "taxid" || field_lower == "tin" {
            detected.insert(PIIType::TaxID);
        }
        if field_lower.contains("passport") {
            detected.insert(PIIType::PassportNumber);
        }
        if field_lower.contains("license") && field_lower.contains("driver") {
            detected.insert(PIIType::DriversLicense);
        }

        detected
    }

    /// Validate credit card number using Luhn algorithm
    fn is_valid_credit_card(&self, s: &str) -> bool {
        let digits: Vec<u32> = s.chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let mut sum = 0;
        let mut alternate = false;

        for &digit in digits.iter().rev() {
            let mut n = digit;
            if alternate {
                n *= 2;
                if n > 9 {
                    n = (n % 10) + 1;
                }
            }
            sum += n;
            alternate = !alternate;
        }

        sum % 10 == 0
    }

    /// Get detailed PII detection results
    pub fn detect_pii_detailed(&self, value: &Value) -> Option<PIIDetectionResult> {
        if let Some(pii_types) = self.detect_pii(value) {
            let highest_risk_level = pii_types.iter()
                .map(|pii| pii.risk_level())
                .max()
                .unwrap_or(PIIRiskLevel::Low);

            let details = pii_types.iter()
                .map(|pii| format!("{}: {}", pii.description(), match pii.risk_level() {
                    PIIRiskLevel::Critical => "CRITICAL RISK",
                    PIIRiskLevel::High => "HIGH RISK",
                    PIIRiskLevel::Medium => "MEDIUM RISK",
                    PIIRiskLevel::Low => "LOW RISK",
                }))
                .collect();

            Some(PIIDetectionResult {
                pii_types,
                highest_risk_level,
                details,
            })
        } else {
            None
        }
    }

    /// Check if a value contains any critical PII
    pub fn contains_critical_pii(&self, value: &Value) -> bool {
        if let Some(pii_types) = self.detect_pii(value) {
            pii_types.iter().any(|pii| pii.risk_level() == PIIRiskLevel::Critical)
        } else {
            false
        }
    }

    /// Check if a value contains any high-risk PII
    pub fn contains_high_risk_pii(&self, value: &Value) -> bool {
        if let Some(pii_types) = self.detect_pii(value) {
            pii_types.iter().any(|pii| pii.risk_level() >= PIIRiskLevel::High)
        } else {
            false
        }
    }
}

impl Default for PIIDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_email_detection() {
        let detector = PIIDetector::new();
        let value = json!("user@example.com");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::EmailAddress));
    }

    #[test]
    fn test_phone_number_detection() {
        let detector = PIIDetector::new();
        let value = json!("(555) 123-4567");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::PhoneNumber));
    }

    #[test]
    fn test_ssn_detection() {
        let detector = PIIDetector::new();
        let value = json!("123-45-6789");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::SocialSecurityNumber));
    }

    #[test]
    fn test_credit_card_detection() {
        let detector = PIIDetector::new();
        // Valid test credit card number (Luhn algorithm passes)
        let value = json!("4532015112830366");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::CreditCardNumber));
    }

    #[test]
    fn test_ip_address_detection() {
        let detector = PIIDetector::new();
        let value = json!("192.168.1.100");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::IPAddress));
    }

    #[test]
    fn test_uuid_detection() {
        let detector = PIIDetector::new();
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::UUID));
    }

    #[test]
    fn test_jwt_token_detection() {
        let detector = PIIDetector::new();
        let value = json!("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::JWTToken));
    }

    #[test]
    fn test_field_name_pii_detection() {
        let detector = PIIDetector::new();
        let value = json!({
            "email_address": "safe_value",
            "user_phone": "safe_value",
            "biometric_template": "safe_value"
        });
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::EmailAddress));
        assert!(pii_types.contains(&PIIType::PhoneNumber));
        assert!(pii_types.contains(&PIIType::BiometricTemplate));
    }

    #[test]
    fn test_nested_object_pii_detection() {
        let detector = PIIDetector::new();
        let value = json!({
            "user": {
                "profile": {
                    "contact": "user@example.com"
                }
            },
            "metadata": {
                "client_ip": "192.168.1.100"
            }
        });
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::EmailAddress));
        assert!(pii_types.contains(&PIIType::IPAddress));
    }

    #[test]
    fn test_array_pii_detection() {
        let detector = PIIDetector::new();
        let value = json!([
            "user@example.com",
            "192.168.1.100",
            "normal_value"
        ]);
        
        let pii_types = detector.detect_pii(&value).unwrap();
        assert!(pii_types.contains(&PIIType::EmailAddress));
        assert!(pii_types.contains(&PIIType::IPAddress));
    }

    #[test]
    fn test_no_pii_detection() {
        let detector = PIIDetector::new();
        let value = json!({
            "action": "wire_transfer",
            "amount": 1000000,
            "timestamp": 1678886400
        });
        
        let pii_result = detector.detect_pii(&value);
        assert!(pii_result.is_none());
    }

    #[test]
    fn test_detailed_pii_detection() {
        let detector = PIIDetector::new();
        let value = json!({
            "email": "user@example.com",
            "ssn": "123-45-6789"
        });
        
        let result = detector.detect_pii_detailed(&value).unwrap();
        assert_eq!(result.highest_risk_level, PIIRiskLevel::Critical);
        assert!(result.pii_types.contains(&PIIType::EmailAddress));
        assert!(result.pii_types.contains(&PIIType::SocialSecurityNumber));
        assert!(result.details.len() >= 2);
    }

    #[test]
    fn test_critical_pii_detection() {
        let detector = PIIDetector::new();
        
        // Critical PII
        let critical_value = json!("123-45-6789"); // SSN
        assert!(detector.contains_critical_pii(&critical_value));
        
        // High risk but not critical
        let high_risk_value = json!("user@example.com");
        assert!(!detector.contains_critical_pii(&high_risk_value));
        assert!(detector.contains_high_risk_pii(&high_risk_value));
        
        // No PII
        let safe_value = json!("wire_transfer");
        assert!(!detector.contains_critical_pii(&safe_value));
        assert!(!detector.contains_high_risk_pii(&safe_value));
    }

    #[test]
    fn test_pii_risk_levels() {
        assert_eq!(PIIType::BiometricTemplate.risk_level(), PIIRiskLevel::Critical);
        assert_eq!(PIIType::SocialSecurityNumber.risk_level(), PIIRiskLevel::Critical);
        assert_eq!(PIIType::CreditCardNumber.risk_level(), PIIRiskLevel::Critical);
        
        assert_eq!(PIIType::EmailAddress.risk_level(), PIIRiskLevel::High);
        assert_eq!(PIIType::PhoneNumber.risk_level(), PIIRiskLevel::High);
        
        assert_eq!(PIIType::IPAddress.risk_level(), PIIRiskLevel::Medium);
        assert_eq!(PIIType::SessionToken.risk_level(), PIIRiskLevel::Medium);
        
        assert_eq!(PIIType::UUID.risk_level(), PIIRiskLevel::Low);
    }
}