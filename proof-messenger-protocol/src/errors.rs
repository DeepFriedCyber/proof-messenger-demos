//! Error types for the proof messenger protocol
//!
//! This module defines all error types that can occur during protocol operations,
//! providing detailed error information for debugging and error handling.

use thiserror::Error;

/// Result type alias for protocol operations
///
/// This is a convenience type alias that uses [`ProtocolError`] as the error type.
/// Most functions in this crate return this type.
pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Main error type for protocol operations
///
/// This enum covers all possible errors that can occur during protocol operations,
/// from cryptographic failures to serialization issues.
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Cryptographic operation failed
    ///
    /// This error occurs when any cryptographic operation fails, such as:
    /// - Key generation
    /// - Signing operations
    /// - Signature verification
    /// - Encryption/decryption
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    /// Invalid message format or content
    ///
    /// This error occurs when a message doesn't conform to the expected format
    /// or contains invalid data.
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    /// Proof verification failed
    ///
    /// This error occurs when a cryptographic proof fails verification,
    /// indicating the proof is invalid or has been tampered with.
    #[error("Proof verification failed: {0}")]
    ProofVerification(String),
    
    /// Proof generation failed
    ///
    /// This error occurs when generating a cryptographic proof fails,
    /// usually due to invalid input data or cryptographic issues.
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    
    /// Serialization or deserialization error
    ///
    /// This error occurs when converting data to/from various formats
    /// such as JSON, binary, or other serialization formats.
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Invalid protocol state
    ///
    /// This error occurs when the protocol is in an invalid state for
    /// the requested operation.
    #[error("Invalid protocol state: {0}")]
    InvalidState(String),
    
    /// Network or communication error
    ///
    /// This error occurs during network operations or communication
    /// with other protocol participants.
    #[error("Network error: {0}")]
    Network(String),
    
    /// Invalid input parameters
    ///
    /// This error occurs when function parameters are invalid or
    /// don't meet the required constraints.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Generic protocol error
    ///
    /// This is a catch-all error for protocol-related issues that
    /// don't fit into other categories.
    #[error("Protocol error: {0}")]
    Protocol(String),
}

impl ProtocolError {
    /// Create a new cryptographic error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the cryptographic failure
    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        Self::Crypto(msg.into())
    }
    
    /// Create a new invalid message error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the message format issue
    pub fn invalid_message<S: Into<String>>(msg: S) -> Self {
        Self::InvalidMessage(msg.into())
    }
    
    /// Create a new proof verification error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the verification failure
    pub fn proof_verification<S: Into<String>>(msg: S) -> Self {
        Self::ProofVerification(msg.into())
    }
    
    /// Create a new proof generation error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the generation failure
    pub fn proof_generation<S: Into<String>>(msg: S) -> Self {
        Self::ProofGeneration(msg.into())
    }
    
    /// Create a new invalid state error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the invalid state
    pub fn invalid_state<S: Into<String>>(msg: S) -> Self {
        Self::InvalidState(msg.into())
    }
    
    /// Create a new network error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the network issue
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }
    
    /// Create a new invalid input error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the invalid input
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Self::InvalidInput(msg.into())
    }
    
    /// Create a new generic protocol error
    ///
    /// # Arguments
    ///
    /// * `msg` - Error message describing the protocol issue
    pub fn protocol<S: Into<String>>(msg: S) -> Self {
        Self::Protocol(msg.into())
    }
}

// Implement conversion from common error types
impl From<ed25519_dalek::SignatureError> for ProtocolError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        Self::Crypto(format!("Ed25519 signature error: {}", err))
    }
}

#[cfg(feature = "wasm")]
impl From<ProtocolError> for wasm_bindgen::JsValue {
    fn from(err: ProtocolError) -> Self {
        wasm_bindgen::JsValue::from_str(&err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let crypto_err = ProtocolError::crypto("Test crypto error");
        assert!(matches!(crypto_err, ProtocolError::Crypto(_)));
        
        let message_err = ProtocolError::invalid_message("Test message error");
        assert!(matches!(message_err, ProtocolError::InvalidMessage(_)));
    }

    #[test]
    fn test_error_display() {
        let err = ProtocolError::crypto("Test error");
        let display = format!("{}", err);
        assert!(display.contains("Cryptographic error"));
        assert!(display.contains("Test error"));
    }

    #[test]
    fn test_serialization_error_creation() {
        let err = ProtocolError::Serialization("Test serialization error".to_string());
        assert!(matches!(err, ProtocolError::Serialization(_)));
        let display = format!("{}", err);
        assert!(display.contains("Serialization error"));
        assert!(display.contains("Test serialization error"));
    }
}