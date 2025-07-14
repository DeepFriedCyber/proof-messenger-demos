use proof_messenger_relay::secure_logger::{SecureLogger, LogLevel, LogEntry};
use std::collections::HashMap;
use chrono::Utc;

/// This example demonstrates the secure logging module with AES-GCM encryption
/// showing complete round-trip encryption/decryption of sensitive security events
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Secure Logging Module Demo");
    println!("=============================");

    // 1. Generate a cryptographically secure key
    let encryption_key = SecureLogger::generate_key();
    let secure_logger = SecureLogger::new(&encryption_key);
    println!("✅ Secure logger initialized with AES-GCM encryption");

    // 2. Demonstrate logging various security events
    println!("\n📝 Logging Security Events:");

    // Authentication success
    let mut auth_metadata = HashMap::new();
    auth_metadata.insert("ip_address".to_string(), "192.168.1.100".to_string());
    auth_metadata.insert("user_agent".to_string(), "Mozilla/5.0 (Enterprise Browser)".to_string());
    auth_metadata.insert("endpoint".to_string(), "/relay".to_string());

    let auth_encrypted = secure_logger.audit_log(
        "JWT authentication successful".to_string(),
        "enterprise-user-456".to_string(),
        Some("req-abc123".to_string()),
        auth_metadata,
    )?;
    println!("   🔒 Authentication event encrypted and logged");

    // Authorization failure (critical security event)
    let mut authz_metadata = HashMap::new();
    authz_metadata.insert("required_scope".to_string(), "proof:create".to_string());
    authz_metadata.insert("user_scopes".to_string(), "message:read".to_string());
    authz_metadata.insert("threat_level".to_string(), "medium".to_string());

    let authz_encrypted = secure_logger.critical_security_event(
        "Authorization denied - insufficient scope for proof creation".to_string(),
        Some("suspicious-user-789".to_string()),
        Some("attack-req-999".to_string()),
        authz_metadata,
    )?;
    println!("   🚨 Authorization failure encrypted and logged");

    // High-value transaction
    let mut transaction_metadata = HashMap::new();
    transaction_metadata.insert("transaction_amount".to_string(), "$1,000,000".to_string());
    transaction_metadata.insert("destination_account".to_string(), "987654321".to_string());
    transaction_metadata.insert("verification_method".to_string(), "cryptographic_proof".to_string());
    transaction_metadata.insert("compliance_check".to_string(), "passed".to_string());

    let transaction_encrypted = secure_logger.log_security_event(
        LogLevel::Audit,
        "High-value transaction approved with cryptographic proof".to_string(),
        Some("executive-user-001".to_string()),
        Some("transaction-req-555".to_string()),
        transaction_metadata,
    )?;
    println!("   💰 High-value transaction encrypted and logged");

    // 3. Demonstrate round-trip decryption
    println!("\n🔓 Decrypting and Verifying Logged Events:");

    // Decrypt authentication event
    let auth_decrypted = secure_logger.decrypt_log_entry(&auth_encrypted)?;
    println!("   ✅ Authentication event decrypted:");
    println!("      User: {}", auth_decrypted.user_id.as_deref().unwrap_or("unknown"));
    println!("      Message: {}", auth_decrypted.message);
    println!("      IP: {}", auth_decrypted.metadata.get("ip_address").unwrap_or(&"unknown".to_string()));
    println!("      Timestamp: {}", auth_decrypted.timestamp);

    // Decrypt authorization failure
    let authz_decrypted = secure_logger.decrypt_log_entry(&authz_encrypted)?;
    println!("   ✅ Authorization failure decrypted:");
    println!("      User: {}", authz_decrypted.user_id.as_deref().unwrap_or("unknown"));
    println!("      Message: {}", authz_decrypted.message);
    println!("      Required Scope: {}", authz_decrypted.metadata.get("required_scope").unwrap_or(&"unknown".to_string()));
    println!("      Threat Level: {}", authz_decrypted.metadata.get("threat_level").unwrap_or(&"unknown".to_string()));

    // Decrypt transaction event
    let transaction_decrypted = secure_logger.decrypt_log_entry(&transaction_encrypted)?;
    println!("   ✅ Transaction event decrypted:");
    println!("      User: {}", transaction_decrypted.user_id.as_deref().unwrap_or("unknown"));
    println!("      Message: {}", transaction_decrypted.message);
    println!("      Amount: {}", transaction_decrypted.metadata.get("transaction_amount").unwrap_or(&"unknown".to_string()));
    println!("      Account: {}", transaction_decrypted.metadata.get("destination_account").unwrap_or(&"unknown".to_string()));

    // 4. Demonstrate security properties
    println!("\n🛡️  Security Properties Demonstration:");

    // Show that ciphertext is different from plaintext
    let original_json = serde_json::to_vec(&auth_decrypted)?;
    println!("   📊 Original log size: {} bytes", original_json.len());
    println!("   🔒 Encrypted log size: {} bytes", auth_encrypted.ciphertext.len());
    println!("   🔑 Nonce size: {} bytes", auth_encrypted.nonce.len());
    
    // Verify that identical logs produce different ciphertexts (due to unique nonces)
    let duplicate_encrypted = secure_logger.audit_log(
        "JWT authentication successful".to_string(),
        "enterprise-user-456".to_string(),
        Some("req-abc123".to_string()),
        HashMap::new(),
    )?;
    
    println!("   🎲 Unique nonces ensure different ciphertexts:");
    println!("      First nonce:  {:02x?}", &auth_encrypted.nonce[..4]);
    println!("      Second nonce: {:02x?}", &duplicate_encrypted.nonce[..4]);
    println!("      Ciphertexts are different: {}", auth_encrypted.ciphertext != duplicate_encrypted.ciphertext);

    // 5. Demonstrate tamper detection
    println!("\n🔍 Tamper Detection:");
    let mut tampered_log = auth_encrypted.clone();
    if !tampered_log.ciphertext.is_empty() {
        tampered_log.ciphertext[0] ^= 0x01; // Flip one bit
    }
    
    match secure_logger.decrypt_log_entry(&tampered_log) {
        Ok(_) => println!("   ❌ ERROR: Tampered log was accepted!"),
        Err(e) => println!("   ✅ Tampered log correctly rejected: {}", e),
    }

    // 6. Demonstrate key isolation
    println!("\n🔐 Key Isolation:");
    let different_key = SecureLogger::generate_key();
    let different_logger = SecureLogger::new(&different_key);
    
    match different_logger.decrypt_log_entry(&auth_encrypted) {
        Ok(_) => println!("   ❌ ERROR: Wrong key was accepted!"),
        Err(e) => println!("   ✅ Wrong key correctly rejected: {}", e),
    }

    println!("\n🎉 Secure Logging Demo Complete!");
    println!("   • All security events encrypted with AES-GCM");
    println!("   • Complete round-trip encryption/decryption verified");
    println!("   • Tamper detection working correctly");
    println!("   • Key isolation enforced");
    println!("   • Unique nonces prevent replay attacks");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proof_messenger_relay::secure_logger::{SecureLogger, LogLevel};

    /// TDD Test Case: Complete round-trip test as specified in the requirements
    /// This test proves the entire lifecycle works: encrypt -> decrypt -> verify
    #[test]
    fn test_log_encryption_decryption_roundtrip() {
        // ARRANGE: Setup secure logger with test key
        let key = b"an example very very secret key."; // 32 bytes for AES-256
        let secure_logger = SecureLogger::new(key);
        
        // Create a realistic log entry with sensitive financial data
        let mut metadata = HashMap::new();
        metadata.insert("transaction_id".to_string(), "TXN-789123".to_string());
        metadata.insert("account_number".to_string(), "4532-1234-5678-9012".to_string());
        metadata.insert("routing_number".to_string(), "021000021".to_string());
        
        let original_log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Audit,
            message: "User-123 approved $1M transfer".to_string(),
            user_id: Some("user-123".to_string()),
            request_id: Some("req-456".to_string()),
            metadata,
        };

        // ACT: 1. Encrypt the log entry
        let encrypted_log = secure_logger.encrypt_log_entry(&original_log_entry)
            .expect("encryption failed!");

        // Verify encryption actually occurred (ciphertext != plaintext)
        let original_json = serde_json::to_vec(&original_log_entry).unwrap();
        assert_ne!(encrypted_log.ciphertext, original_json);
        assert_eq!(encrypted_log.nonce.len(), 12); // AES-GCM nonce size

        // ACT: 2. Decrypt the log entry
        let decrypted_log = secure_logger.decrypt_log_entry(&encrypted_log)
            .expect("decryption failed!");

        // ASSERT: 3. Verify that the decrypted data matches the original
        assert_eq!(decrypted_log, original_log_entry);
        assert_eq!(decrypted_log.message, "User-123 approved $1M transfer");
        assert_eq!(decrypted_log.user_id, Some("user-123".to_string()));
        assert_eq!(decrypted_log.level, LogLevel::Audit);
        assert_eq!(decrypted_log.metadata.get("transaction_id"), Some(&"TXN-789123".to_string()));
        assert_eq!(decrypted_log.metadata.get("account_number"), Some(&"4532-1234-5678-9012".to_string()));
    }

    /// TDD Test Case: Verify AES-GCM provides authentication (tamper detection)
    #[test]
    fn test_aes_gcm_authentication_tamper_detection() {
        // ARRANGE: Setup and encrypt a log
        let key = SecureLogger::generate_key();
        let secure_logger = SecureLogger::new(&key);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Critical,
            message: "Security breach detected".to_string(),
            user_id: Some("admin".to_string()),
            request_id: None,
            metadata: HashMap::new(),
        };

        let mut encrypted_log = secure_logger.encrypt_log_entry(&log_entry).unwrap();

        // ACT: Tamper with the ciphertext (simulate attacker modification)
        if !encrypted_log.ciphertext.is_empty() {
            encrypted_log.ciphertext[0] ^= 0x01; // Flip one bit
        }

        // ASSERT: AES-GCM should detect tampering and reject decryption
        let result = secure_logger.decrypt_log_entry(&encrypted_log);
        assert!(result.is_err(), "Tampered ciphertext should be rejected");
    }

    /// TDD Test Case: Verify unique nonces prevent identical ciphertexts
    #[test]
    fn test_unique_nonces_prevent_identical_ciphertexts() {
        // ARRANGE: Setup secure logger
        let key = SecureLogger::generate_key();
        let secure_logger = SecureLogger::new(&key);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Identical message".to_string(),
            user_id: Some("user-123".to_string()),
            request_id: None,
            metadata: HashMap::new(),
        };

        // ACT: Encrypt the same log entry twice
        let encrypted1 = secure_logger.encrypt_log_entry(&log_entry).unwrap();
        let encrypted2 = secure_logger.encrypt_log_entry(&log_entry).unwrap();

        // ASSERT: Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        
        // But both should decrypt to the same plaintext
        let decrypted1 = secure_logger.decrypt_log_entry(&encrypted1).unwrap();
        let decrypted2 = secure_logger.decrypt_log_entry(&encrypted2).unwrap();
        assert_eq!(decrypted1.message, decrypted2.message);
    }

    /// TDD Test Case: Verify key isolation (wrong key rejection)
    #[test]
    fn test_key_isolation_wrong_key_rejection() {
        // ARRANGE: Setup two loggers with different keys
        let key1 = SecureLogger::generate_key();
        let key2 = SecureLogger::generate_key();
        let logger1 = SecureLogger::new(&key1);
        let logger2 = SecureLogger::new(&key2);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warning,
            message: "Sensitive operation".to_string(),
            user_id: Some("user-456".to_string()),
            request_id: None,
            metadata: HashMap::new(),
        };

        // ACT: Encrypt with logger1, try to decrypt with logger2
        let encrypted = logger1.encrypt_log_entry(&log_entry).unwrap();
        let result = logger2.decrypt_log_entry(&encrypted);

        // ASSERT: Wrong key should be rejected
        assert!(result.is_err(), "Wrong key should be rejected");
    }

    /// Integration Test: Verify secure logging works with OAuth2.0 security events
    #[test]
    fn test_oauth_security_event_logging() {
        // ARRANGE: Setup secure logger
        let key = SecureLogger::generate_key();
        let secure_logger = SecureLogger::new(&key);
        
        // Simulate OAuth2.0 security events
        let mut oauth_metadata = HashMap::new();
        oauth_metadata.insert("jwt_issuer".to_string(), "https://okta.com".to_string());
        oauth_metadata.insert("jwt_audience".to_string(), "proof-messenger-api".to_string());
        oauth_metadata.insert("scopes".to_string(), "proof:create message:read".to_string());
        oauth_metadata.insert("client_id".to_string(), "enterprise-app-123".to_string());

        // ACT: Log OAuth2.0 authentication success
        let encrypted = secure_logger.audit_log(
            "OAuth2.0 JWT validation successful".to_string(),
            "enterprise-user-789".to_string(),
            Some("oauth-req-456".to_string()),
            oauth_metadata.clone(),
        ).unwrap();

        // ASSERT: Decrypt and verify OAuth2.0 event
        let decrypted = secure_logger.decrypt_log_entry(&encrypted).unwrap();
        assert_eq!(decrypted.level, LogLevel::Audit);
        assert_eq!(decrypted.message, "OAuth2.0 JWT validation successful");
        assert_eq!(decrypted.user_id, Some("enterprise-user-789".to_string()));
        assert_eq!(decrypted.metadata.get("jwt_issuer"), Some(&"https://okta.com".to_string()));
        assert_eq!(decrypted.metadata.get("scopes"), Some(&"proof:create message:read".to_string()));
    }
}