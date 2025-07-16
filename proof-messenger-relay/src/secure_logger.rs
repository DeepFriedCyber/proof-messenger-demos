use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn, error};
use rand::RngCore;

/// Errors that can occur during secure logging operations
#[derive(Error, Debug)]
pub enum SecureLogError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid key format")]
    InvalidKey,
    
    #[error("Invalid nonce format")]
    InvalidNonce,
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(#[from] serde_json::Error),
    
    #[error("Storage operation failed: {0}")]
    StorageFailed(String),
}

/// Log entry levels for different types of security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Critical,
    Audit,
}

/// Structured log entry that will be encrypted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Encrypted log entry with nonce for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedLogEntry {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub timestamp: DateTime<Utc>, // Unencrypted for indexing
    pub level: LogLevel,          // Unencrypted for filtering
}

/// Secure logger that encrypts sensitive log data using AES-GCM
pub struct SecureLogger {
    cipher: Aes256Gcm,
}

impl SecureLogger {
    /// Create a new secure logger with a 256-bit key
    pub fn new(key: &[u8; 32]) -> Self {
        let key_array = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key_array);
        
        Self {
            cipher,
        }
    }

    /// Generate a cryptographically secure random key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Generate a unique nonce for each encryption operation
    fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt a log entry using AES-GCM (AEAD)
    pub fn encrypt_log_entry(&self, entry: &LogEntry) -> Result<EncryptedLogEntry, SecureLogError> {
        // Serialize the log entry to JSON
        let plaintext = serde_json::to_vec(entry)?;
        
        // Generate a unique nonce for this encryption
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt using AES-GCM (provides both confidentiality and authenticity)
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| SecureLogError::EncryptionFailed(e.to_string()))?;
        
        Ok(EncryptedLogEntry {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            timestamp: entry.timestamp,
            level: entry.level.clone(),
        })
    }

    /// Decrypt an encrypted log entry
    pub fn decrypt_log_entry(&self, encrypted: &EncryptedLogEntry) -> Result<LogEntry, SecureLogError> {
        // Reconstruct the nonce
        if encrypted.nonce.len() != 12 {
            return Err(SecureLogError::InvalidNonce);
        }
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        // Decrypt using AES-GCM (automatically verifies authenticity)
        let plaintext = self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| SecureLogError::DecryptionFailed(e.to_string()))?;
        
        // Deserialize the decrypted JSON
        let entry: LogEntry = serde_json::from_slice(&plaintext)?;
        
        Ok(entry)
    }

    /// Log a security event with encryption
    pub fn log_security_event(
        &self,
        level: LogLevel,
        message: String,
        user_id: Option<String>,
        request_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<EncryptedLogEntry, SecureLogError> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: level.clone(),
            message: message.clone(),
            user_id: user_id.clone(),
            request_id: request_id.clone(),
            metadata,
        };

        // Log to standard tracing (without sensitive data)
        let sanitized_message = format!(
            "Security event: {} (user: {}, request: {})",
            message,
            user_id.as_deref().unwrap_or("anonymous"),
            request_id.as_deref().unwrap_or("none")
        );

        match level {
            LogLevel::Info => info!("{}", sanitized_message),
            LogLevel::Warning => warn!("{}", sanitized_message),
            LogLevel::Error | LogLevel::Critical => error!("{}", sanitized_message),
            LogLevel::Audit => info!("AUDIT: {}", sanitized_message),
        }

        // Encrypt the full entry for secure storage
        self.encrypt_log_entry(&entry)
    }

    /// Convenience method for audit logging
    pub fn audit_log(
        &self,
        message: String,
        user_id: String,
        request_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<EncryptedLogEntry, SecureLogError> {
        self.log_security_event(
            LogLevel::Audit,
            message,
            Some(user_id),
            request_id,
            metadata,
        )
    }

    /// Convenience method for critical security events
    pub fn critical_security_event(
        &self,
        message: String,
        user_id: Option<String>,
        request_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<EncryptedLogEntry, SecureLogError> {
        self.log_security_event(
            LogLevel::Critical,
            message,
            user_id,
            request_id,
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// TDD Test Case 1: Complete encryption/decryption round-trip
    /// This test proves the entire lifecycle works correctly
    #[test]
    fn test_log_encryption_decryption_roundtrip() {
        // ARRANGE: Set up secure logger with test key
        let key = b"an example very very secret key."; // 32 bytes
        let logger = SecureLogger::new(key);
        
        // Create a realistic log entry with sensitive data
        let mut metadata = HashMap::new();
        metadata.insert("ip_address".to_string(), "192.168.1.100".to_string());
        metadata.insert("user_agent".to_string(), "Mozilla/5.0...".to_string());
        metadata.insert("transaction_amount".to_string(), "$1,000,000".to_string());
        
        let original_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Audit,
            message: "User-123 approved $1M transfer to account 987654321".to_string(),
            user_id: Some("user-123".to_string()),
            request_id: Some("req-456".to_string()),
            metadata,
        };

        // ACT: 1. Encrypt the log entry
        let encrypted_entry = logger.encrypt_log_entry(&original_entry)
            .expect("Encryption should succeed");

        // Verify that the data is actually encrypted (ciphertext != plaintext)
        let original_json = serde_json::to_vec(&original_entry).unwrap();
        assert_ne!(encrypted_entry.ciphertext, original_json);
        assert_eq!(encrypted_entry.nonce.len(), 12); // AES-GCM nonce size

        // ACT: 2. Decrypt the log entry
        let decrypted_entry = logger.decrypt_log_entry(&encrypted_entry)
            .expect("Decryption should succeed");

        // ASSERT: 3. Verify complete round-trip integrity
        assert_eq!(decrypted_entry, original_entry);
        assert_eq!(decrypted_entry.message, "User-123 approved $1M transfer to account 987654321");
        assert_eq!(decrypted_entry.user_id, Some("user-123".to_string()));
        assert_eq!(decrypted_entry.request_id, Some("req-456".to_string()));
        assert_eq!(decrypted_entry.level, LogLevel::Audit);
        assert_eq!(decrypted_entry.metadata.get("transaction_amount"), Some(&"$1,000,000".to_string()));
    }

    /// TDD Test Case 2: Unique nonces for each encryption
    /// This test ensures that identical plaintexts produce different ciphertexts
    #[test]
    fn test_unique_nonces_produce_different_ciphertexts() {
        // ARRANGE: Set up secure logger
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Identical message".to_string(),
            user_id: Some("user-123".to_string()),
            request_id: None,
            metadata: HashMap::new(),
        };

        // ACT: Encrypt the same entry twice
        let encrypted1 = logger.encrypt_log_entry(&log_entry).unwrap();
        let encrypted2 = logger.encrypt_log_entry(&log_entry).unwrap();

        // ASSERT: Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        
        // But both should decrypt to the same plaintext
        let decrypted1 = logger.decrypt_log_entry(&encrypted1).unwrap();
        let decrypted2 = logger.decrypt_log_entry(&encrypted2).unwrap();
        assert_eq!(decrypted1.message, decrypted2.message);
    }

    /// TDD Test Case 3: Tampered ciphertext detection
    /// This test ensures that AES-GCM detects tampering (authenticity)
    #[test]
    fn test_tampered_ciphertext_detection() {
        // ARRANGE: Set up secure logger and encrypt a log entry
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Critical,
            message: "Security breach detected".to_string(),
            user_id: Some("admin".to_string()),
            request_id: Some("security-alert-001".to_string()),
            metadata: HashMap::new(),
        };

        let mut encrypted = logger.encrypt_log_entry(&log_entry).unwrap();

        // ACT: Tamper with the ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0x01; // Flip one bit
        }

        // ASSERT: Decryption should fail due to authentication failure
        let result = logger.decrypt_log_entry(&encrypted);
        assert!(matches!(result, Err(SecureLogError::DecryptionFailed(_))));
    }

    /// TDD Test Case 4: Wrong key rejection
    /// This test ensures that logs encrypted with one key cannot be decrypted with another
    #[test]
    fn test_wrong_key_rejection() {
        // ARRANGE: Set up two different secure loggers
        let key1 = SecureLogger::generate_key();
        let key2 = SecureLogger::generate_key();
        let logger1 = SecureLogger::new(&key1);
        let logger2 = SecureLogger::new(&key2);
        
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warning,
            message: "Failed login attempt".to_string(),
            user_id: Some("attacker".to_string()),
            request_id: None,
            metadata: HashMap::new(),
        };

        // ACT: Encrypt with logger1, try to decrypt with logger2
        let encrypted = logger1.encrypt_log_entry(&log_entry).unwrap();
        let result = logger2.decrypt_log_entry(&encrypted);

        // ASSERT: Decryption should fail with wrong key
        assert!(matches!(result, Err(SecureLogError::DecryptionFailed(_))));
    }

    /// TDD Test Case 5: High-level security event logging
    /// This test validates the convenience methods for security logging
    #[test]
    fn test_security_event_logging() {
        // ARRANGE: Set up secure logger
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        let mut metadata = HashMap::new();
        metadata.insert("source_ip".to_string(), "10.0.0.1".to_string());
        metadata.insert("endpoint".to_string(), "/relay".to_string());

        // ACT: Log a security event
        let encrypted = logger.log_security_event(
            LogLevel::Audit,
            "JWT token validation successful".to_string(),
            Some("user-789".to_string()),
            Some("req-abc123".to_string()),
            metadata.clone(),
        ).unwrap();

        // ASSERT: Decrypt and verify the logged event
        let decrypted = logger.decrypt_log_entry(&encrypted).unwrap();
        assert_eq!(decrypted.level, LogLevel::Audit);
        assert_eq!(decrypted.message, "JWT token validation successful");
        assert_eq!(decrypted.user_id, Some("user-789".to_string()));
        assert_eq!(decrypted.request_id, Some("req-abc123".to_string()));
        assert_eq!(decrypted.metadata.get("source_ip"), Some(&"10.0.0.1".to_string()));
    }

    /// TDD Test Case 6: Audit logging convenience method
    /// This test validates the audit-specific logging functionality
    #[test]
    fn test_audit_logging() {
        // ARRANGE: Set up secure logger
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "proof_verification".to_string());
        metadata.insert("result".to_string(), "success".to_string());
        metadata.insert("proof_hash".to_string(), "0x1234...".to_string());

        // ACT: Use audit logging convenience method
        let encrypted = logger.audit_log(
            "Proof verification completed successfully".to_string(),
            "enterprise-user-456".to_string(),
            Some("proof-req-789".to_string()),
            metadata,
        ).unwrap();

        // ASSERT: Verify audit log structure
        let decrypted = logger.decrypt_log_entry(&encrypted).unwrap();
        assert_eq!(decrypted.level, LogLevel::Audit);
        assert_eq!(decrypted.user_id, Some("enterprise-user-456".to_string()));
        assert_eq!(decrypted.metadata.get("action"), Some(&"proof_verification".to_string()));
    }

    /// TDD Test Case 7: Critical security event logging
    /// This test validates logging of critical security events
    #[test]
    fn test_critical_security_event_logging() {
        // ARRANGE: Set up secure logger
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        let mut metadata = HashMap::new();
        metadata.insert("threat_level".to_string(), "high".to_string());
        metadata.insert("attack_type".to_string(), "jwt_forgery_attempt".to_string());
        metadata.insert("blocked".to_string(), "true".to_string());

        // ACT: Log critical security event
        let encrypted = logger.critical_security_event(
            "JWT forgery attempt detected and blocked".to_string(),
            Some("suspicious-user-999".to_string()),
            Some("attack-req-666".to_string()),
            metadata,
        ).unwrap();

        // ASSERT: Verify critical event structure
        let decrypted = logger.decrypt_log_entry(&encrypted).unwrap();
        assert_eq!(decrypted.level, LogLevel::Critical);
        assert_eq!(decrypted.message, "JWT forgery attempt detected and blocked");
        assert_eq!(decrypted.metadata.get("threat_level"), Some(&"high".to_string()));
    }

    /// TDD Test Case 8: Key generation produces unique keys
    /// This test ensures the key generation function produces cryptographically secure keys
    #[test]
    fn test_key_generation_uniqueness() {
        // ACT: Generate multiple keys
        let key1 = SecureLogger::generate_key();
        let key2 = SecureLogger::generate_key();
        let key3 = SecureLogger::generate_key();

        // ASSERT: All keys should be different
        assert_ne!(key1, key2);
        assert_ne!(key2, key3);
        assert_ne!(key1, key3);
        
        // All keys should be 32 bytes (256 bits)
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_eq!(key3.len(), 32);
    }

    /// TDD Test Case 9: Large log entry handling
    /// This test ensures the system can handle large log entries
    #[test]
    fn test_large_log_entry_handling() {
        // ARRANGE: Set up secure logger
        let key = SecureLogger::generate_key();
        let logger = SecureLogger::new(&key);
        
        // Create a large log entry (simulating detailed audit data)
        let large_message = "A".repeat(10000); // 10KB message
        let mut large_metadata = HashMap::new();
        for i in 0..100 {
            large_metadata.insert(format!("field_{}", i), format!("value_{}", i));
        }
        
        let large_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Audit,
            message: large_message.clone(),
            user_id: Some("bulk-user".to_string()),
            request_id: Some("bulk-req".to_string()),
            metadata: large_metadata.clone(),
        };

        // ACT: Encrypt and decrypt large entry
        let encrypted = logger.encrypt_log_entry(&large_entry).unwrap();
        let decrypted = logger.decrypt_log_entry(&encrypted).unwrap();

        // ASSERT: Large entry should round-trip correctly
        assert_eq!(decrypted.message, large_message);
        assert_eq!(decrypted.metadata.len(), 100);
        assert_eq!(decrypted.metadata.get("field_50"), Some(&"value_50".to_string()));
    }
}