//! Core protocol message types and handling
//!
//! This module defines the wire protocol for the proof messenger system,
//! including message framing, routing, and protocol-level operations.

use crate::errors::{ProtocolError, Result};
use crate::messages::Message;
use crate::proofs::Proof;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A protocol-level message that can contain various payload types
///
/// This is the top-level message type used for communication between
/// protocol participants. It includes routing information and payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Unique identifier for this protocol message
    pub id: Uuid,
    /// Type of message payload
    pub message_type: MessageType,
    /// When this protocol message was created
    pub timestamp: DateTime<Utc>,
    /// Protocol version for compatibility
    pub version: ProtocolVersion,
    /// Optional routing information
    pub routing: Option<RoutingInfo>,
    /// The actual message payload
    pub payload: MessagePayload,
}

/// Types of protocol messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Initial handshake message
    Handshake,
    /// User message containing actual content
    Message,
    /// Cryptographic proof
    Proof,
    /// Acknowledgment of received message
    Ack,
    /// Error notification
    Error,
    /// Heartbeat/keepalive message
    Heartbeat,
    /// Connection termination
    Disconnect,
}

/// Protocol version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version number
    pub major: u16,
    /// Minor version number
    pub minor: u16,
    /// Patch version number
    pub patch: u16,
}

/// Routing information for message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    /// Source address/identifier
    pub source: String,
    /// Destination address/identifier
    pub destination: String,
    /// Optional relay chain
    pub relay_chain: Vec<String>,
    /// Time-to-live for the message
    pub ttl: u32,
}

/// Message payload containing the actual data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Handshake payload
    Handshake(HandshakePayload),
    /// User message payload
    Message(Message),
    /// Proof payload
    Proof(Proof),
    /// Acknowledgment payload
    Ack(AckPayload),
    /// Error payload
    Error(ErrorPayload),
    /// Heartbeat payload (empty)
    Heartbeat,
    /// Disconnect payload
    Disconnect(DisconnectPayload),
}

/// Handshake message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    /// Protocol version supported by sender
    pub version: ProtocolVersion,
    /// Capabilities supported by sender
    pub capabilities: Vec<String>,
    /// Optional challenge for authentication
    pub challenge: Option<Vec<u8>>,
    /// Optional response to a challenge
    pub challenge_response: Option<Vec<u8>>,
}

/// Acknowledgment message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckPayload {
    /// ID of the message being acknowledged
    pub message_id: Uuid,
    /// Status of the acknowledgment
    pub status: AckStatus,
    /// Optional additional information
    pub info: Option<String>,
}

/// Status of an acknowledgment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    /// Message received successfully
    Received,
    /// Message processed successfully
    Processed,
    /// Message delivery failed
    Failed,
    /// Message was rejected
    Rejected,
}

/// Error message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Optional additional error details
    pub details: Option<serde_json::Value>,
}

/// Protocol error codes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid message format
    InvalidFormat,
    /// Unsupported protocol version
    UnsupportedVersion,
    /// Authentication failed
    AuthenticationFailed,
    /// Authorization failed
    AuthorizationFailed,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Internal server error
    InternalError,
    /// Unknown error
    Unknown,
}

/// Disconnect message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectPayload {
    /// Reason for disconnection
    pub reason: DisconnectReason,
    /// Optional additional information
    pub message: Option<String>,
}

/// Reasons for disconnection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// Normal shutdown
    Normal,
    /// Protocol error
    ProtocolError,
    /// Authentication failure
    AuthenticationFailure,
    /// Timeout
    Timeout,
    /// Resource exhaustion
    ResourceExhaustion,
}

impl ProtocolMessage {
    /// Create a new protocol message
    ///
    /// # Arguments
    ///
    /// * `message_type` - The type of message
    /// * `payload` - The message payload
    pub fn new(message_type: MessageType, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type,
            timestamp: Utc::now(),
            version: ProtocolVersion::current(),
            routing: None,
            payload,
        }
    }

    /// Create a new protocol message with routing information
    ///
    /// # Arguments
    ///
    /// * `message_type` - The type of message
    /// * `payload` - The message payload
    /// * `routing` - Routing information
    pub fn new_with_routing(
        message_type: MessageType,
        payload: MessagePayload,
        routing: RoutingInfo,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type,
            timestamp: Utc::now(),
            version: ProtocolVersion::current(),
            routing: Some(routing),
            payload,
        }
    }

    /// Create a handshake message
    ///
    /// # Arguments
    ///
    /// * `capabilities` - List of supported capabilities
    pub fn handshake(capabilities: Vec<String>) -> Self {
        let payload = MessagePayload::Handshake(HandshakePayload {
            version: ProtocolVersion::current(),
            capabilities,
            challenge: None,
            challenge_response: None,
        });
        
        Self::new(MessageType::Handshake, payload)
    }

    /// Create an acknowledgment message
    ///
    /// # Arguments
    ///
    /// * `message_id` - ID of the message being acknowledged
    /// * `status` - Acknowledgment status
    pub fn ack(message_id: Uuid, status: AckStatus) -> Self {
        let payload = MessagePayload::Ack(AckPayload {
            message_id,
            status,
            info: None,
        });
        
        Self::new(MessageType::Ack, payload)
    }

    /// Create an error message
    ///
    /// # Arguments
    ///
    /// * `code` - Error code
    /// * `message` - Error message
    pub fn error(code: ErrorCode, message: String) -> Self {
        let payload = MessagePayload::Error(ErrorPayload {
            code,
            message,
            details: None,
        });
        
        Self::new(MessageType::Error, payload)
    }

    /// Create a heartbeat message
    pub fn heartbeat() -> Self {
        Self::new(MessageType::Heartbeat, MessagePayload::Heartbeat)
    }

    /// Create a disconnect message
    ///
    /// # Arguments
    ///
    /// * `reason` - Reason for disconnection
    pub fn disconnect(reason: DisconnectReason) -> Self {
        let payload = MessagePayload::Disconnect(DisconnectPayload {
            reason,
            message: None,
        });
        
        Self::new(MessageType::Disconnect, payload)
    }

    /// Serialize the protocol message to bytes
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Serialization` if serialization fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| ProtocolError::protocol(format!("Failed to serialize protocol message: {}", e)))
    }

    /// Deserialize a protocol message from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to deserialize
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Serialization` if deserialization fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| ProtocolError::protocol(format!("Failed to deserialize protocol message: {}", e)))
    }

    /// Check if this message is compatible with a given protocol version
    ///
    /// # Arguments
    ///
    /// * `version` - The version to check compatibility with
    pub fn is_compatible_with(&self, version: &ProtocolVersion) -> bool {
        self.version.is_compatible_with(version)
    }
}

impl ProtocolVersion {
    /// Get the current protocol version
    pub fn current() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
        }
    }

    /// Create a new protocol version
    ///
    /// # Arguments
    ///
    /// * `major` - Major version number
    /// * `minor` - Minor version number
    /// * `patch` - Patch version number
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Check if this version is compatible with another version
    ///
    /// Compatibility rules:
    /// - Same major version is required
    /// - Minor version can be different (backward compatible)
    /// - Patch version can be different
    ///
    /// # Arguments
    ///
    /// * `other` - The other version to check compatibility with
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Handshake => write!(f, "Handshake"),
            MessageType::Message => write!(f, "Message"),
            MessageType::Proof => write!(f, "Proof"),
            MessageType::Ack => write!(f, "Ack"),
            MessageType::Error => write!(f, "Error"),
            MessageType::Heartbeat => write!(f, "Heartbeat"),
            MessageType::Disconnect => write!(f, "Disconnect"),
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InvalidFormat => write!(f, "InvalidFormat"),
            ErrorCode::UnsupportedVersion => write!(f, "UnsupportedVersion"),
            ErrorCode::AuthenticationFailed => write!(f, "AuthenticationFailed"),
            ErrorCode::AuthorizationFailed => write!(f, "AuthorizationFailed"),
            ErrorCode::RateLimitExceeded => write!(f, "RateLimitExceeded"),
            ErrorCode::InternalError => write!(f, "InternalError"),
            ErrorCode::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_message_creation() {
        let message = ProtocolMessage::handshake(vec!["messaging".to_string(), "proofs".to_string()]);
        
        assert_eq!(message.message_type, MessageType::Handshake);
        assert!(matches!(message.payload, MessagePayload::Handshake(_)));
    }

    #[test]
    fn test_protocol_version_compatibility() {
        let v1 = ProtocolVersion::new(1, 0, 0);
        let v1_1 = ProtocolVersion::new(1, 1, 0);
        let v2 = ProtocolVersion::new(2, 0, 0);
        
        assert!(v1.is_compatible_with(&v1_1));
        assert!(v1_1.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
        assert!(!v2.is_compatible_with(&v1));
    }

    #[test]
    fn test_message_serialization() {
        let message = ProtocolMessage::heartbeat();
        let bytes = message.to_bytes().expect("Failed to serialize message");
        let deserialized = ProtocolMessage::from_bytes(&bytes)
            .expect("Failed to deserialize message");
        
        assert_eq!(message.id, deserialized.id);
        assert_eq!(message.message_type, deserialized.message_type);
    }

    #[test]
    fn test_ack_message() {
        let original_id = Uuid::new_v4();
        let ack = ProtocolMessage::ack(original_id, AckStatus::Received);
        
        if let MessagePayload::Ack(ack_payload) = ack.payload {
            assert_eq!(ack_payload.message_id, original_id);
            assert_eq!(ack_payload.status, AckStatus::Received);
        } else {
            panic!("Expected Ack payload");
        }
    }

    #[test]
    fn test_error_message() {
        let error = ProtocolMessage::error(
            ErrorCode::InvalidFormat,
            "Invalid message format".to_string(),
        );
        
        if let MessagePayload::Error(error_payload) = error.payload {
            assert_eq!(error_payload.code, ErrorCode::InvalidFormat);
            assert_eq!(error_payload.message, "Invalid message format");
        } else {
            panic!("Expected Error payload");
        }
    }
}