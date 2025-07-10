//! Message types and builders for the proof messenger protocol
//!
//! This module provides the core message types used in the protocol, along with
//! convenient builders for creating messages with various properties and proofs.
//!
//! # Example
//!
//! ```rust
//! use proof_messenger_protocol::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let sender_keypair = KeyPair::generate()?;
//! let recipient_keypair = KeyPair::generate()?;
//!
//! let message = MessageBuilder::new()
//!     .sender(sender_keypair.public_key().clone())
//!     .recipient(recipient_keypair.public_key().clone())
//!     .content("Hello, secure world!".to_string())
//!     .sign_with(&sender_keypair)?
//!     .build()?;
//!
//! assert!(message.verify_signature()?);
//! # Ok(())
//! # }
//! ```

use crate::crypto::{KeyPair, PublicKey, Signature};
use crate::errors::{ProtocolError, Result};
use crate::proofs::{Proof, ProofType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message in the proof messenger protocol
///
/// This struct represents a complete message with sender, recipient,
/// content, and optional cryptographic proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for this message
    pub id: Uuid,
    /// Public key of the message sender
    pub sender: PublicKey,
    /// Public key of the message recipient
    pub recipient: PublicKey,
    /// The message content
    pub content: String,
    /// When this message was created
    pub timestamp: DateTime<Utc>,
    /// Optional signature over the message
    pub signature: Option<Signature>,
    /// Optional proofs attached to this message
    pub proofs: Vec<Proof>,
    /// Message metadata
    pub metadata: MessageMetadata,
}

/// Metadata associated with a message
///
/// This struct contains additional information about the message
/// that may be useful for processing or display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Message type classification
    pub message_type: MessageType,
    /// Priority level of the message
    pub priority: MessagePriority,
    /// Whether this message requires a read receipt
    pub requires_receipt: bool,
    /// Optional expiration time for the message
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional reply-to message ID
    pub reply_to: Option<Uuid>,
    /// Optional thread ID for message threading
    pub thread_id: Option<Uuid>,
}

/// Types of messages supported by the protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Regular text message
    Text,
    /// System or protocol message
    System,
    /// Invitation message
    Invitation,
    /// Receipt or acknowledgment
    Receipt,
    /// Error message
    Error,
}

/// Priority levels for messages
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority message
    Low,
    /// Normal priority message (default)
    Normal,
    /// High priority message
    High,
    /// Urgent message requiring immediate attention
    Urgent,
}

/// Builder for creating messages with various options
///
/// This builder provides a fluent interface for constructing messages
/// with different properties, proofs, and signatures.
pub struct MessageBuilder {
    sender: Option<PublicKey>,
    recipient: Option<PublicKey>,
    content: Option<String>,
    message_type: MessageType,
    priority: MessagePriority,
    requires_receipt: bool,
    expires_at: Option<DateTime<Utc>>,
    reply_to: Option<Uuid>,
    thread_id: Option<Uuid>,
    proofs: Vec<Proof>,
    keypair_for_signing: Option<KeyPair>,
}

impl Message {
    /// Create a new message with basic information
    ///
    /// # Arguments
    ///
    /// * `sender` - Public key of the sender
    /// * `recipient` - Public key of the recipient
    /// * `content` - Message content
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let sender = KeyPair::generate()?.public_key().clone();
    /// let recipient = KeyPair::generate()?.public_key().clone();
    /// 
    /// let message = Message::new(sender, recipient, "Hello!".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(sender: PublicKey, recipient: PublicKey, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            content,
            timestamp: Utc::now(),
            signature: None,
            proofs: Vec::new(),
            metadata: MessageMetadata::default(),
        }
    }

    /// Sign this message with a keypair
    ///
    /// # Arguments
    ///
    /// * `keypair` - The keypair to sign with (should match the sender's key)
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if signing fails or if the keypair
    /// doesn't match the sender's public key.
    pub fn sign(&mut self, keypair: &KeyPair) -> Result<()> {
        // Verify that the keypair matches the sender
        if keypair.public_key() != &self.sender {
            return Err(ProtocolError::crypto(
                "Keypair public key doesn't match message sender"
            ));
        }

        let message_bytes = self.to_bytes_for_signing()?;
        let signature = keypair.sign(&message_bytes)?;
        self.signature = Some(signature);
        
        Ok(())
    }

    /// Verify the signature on this message
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if verification fails.
    pub fn verify_signature(&self) -> Result<bool> {
        match &self.signature {
            Some(signature) => {
                let message_bytes = self.to_bytes_for_signing()?;
                self.sender.verify(&message_bytes, signature)
            }
            None => Ok(false), // Unsigned message
        }
    }

    /// Check if this message is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Add a proof to this message
    ///
    /// # Arguments
    ///
    /// * `proof` - The proof to attach
    pub fn add_proof(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    /// Get all proofs of a specific type
    ///
    /// # Arguments
    ///
    /// * `proof_type` - The type of proofs to retrieve
    pub fn get_proofs_by_type(&self, proof_type: &ProofType) -> Vec<&Proof> {
        self.proofs.iter()
            .filter(|proof| &proof.proof_type == proof_type)
            .collect()
    }

    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        match self.metadata.expires_at {
            Some(expiry) => Utc::now() > expiry,
            None => false,
        }
    }

    /// Check if this message is a reply to another message
    pub fn is_reply(&self) -> bool {
        self.metadata.reply_to.is_some()
    }

    /// Get the age of this message
    pub fn age(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.timestamp)
    }

    /// Convert message to bytes for signing (excludes signature field)
    fn to_bytes_for_signing(&self) -> Result<Vec<u8>> {
        let signing_data = MessageSigningData {
            id: self.id,
            sender: &self.sender,
            recipient: &self.recipient,
            content: &self.content,
            timestamp: self.timestamp,
            metadata: &self.metadata,
        };
        
        bincode::serialize(&signing_data)
            .map_err(|e| ProtocolError::invalid_message(format!("Failed to serialize message for signing: {}", e)))
    }
}

/// Data structure used for message signing (excludes signature to prevent circular dependency)
#[derive(Serialize)]
struct MessageSigningData<'a> {
    id: Uuid,
    sender: &'a PublicKey,
    recipient: &'a PublicKey,
    content: &'a str,
    timestamp: DateTime<Utc>,
    metadata: &'a MessageMetadata,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            sender: None,
            recipient: None,
            content: None,
            message_type: MessageType::Text,
            priority: MessagePriority::Normal,
            requires_receipt: false,
            expires_at: None,
            reply_to: None,
            thread_id: None,
            proofs: Vec::new(),
            keypair_for_signing: None,
        }
    }

    /// Set the message sender
    pub fn sender(mut self, sender: PublicKey) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Set the message recipient
    pub fn recipient(mut self, recipient: PublicKey) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Set the message content
    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    /// Set the message type
    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Set the message priority
    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set whether this message requires a receipt
    pub fn requires_receipt(mut self, requires_receipt: bool) -> Self {
        self.requires_receipt = requires_receipt;
        self
    }

    /// Set an expiration time for the message
    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set this message as a reply to another message
    pub fn reply_to(mut self, message_id: Uuid) -> Self {
        self.reply_to = Some(message_id);
        self
    }

    /// Set the thread ID for message threading
    pub fn thread_id(mut self, thread_id: Uuid) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    /// Add a proof to the message
    pub fn add_proof(mut self, proof: Proof) -> Self {
        self.proofs.push(proof);
        self
    }

    /// Set the keypair for signing the message
    pub fn sign_with(mut self, keypair: &KeyPair) -> Result<Self> {
        self.keypair_for_signing = Some(keypair.clone());
        Ok(self)
    }

    /// Build the message
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::InvalidMessage` if required fields are missing.
    pub fn build(self) -> Result<Message> {
        let sender = self.sender
            .ok_or_else(|| ProtocolError::invalid_message("Sender is required"))?;
        let recipient = self.recipient
            .ok_or_else(|| ProtocolError::invalid_message("Recipient is required"))?;
        let content = self.content
            .ok_or_else(|| ProtocolError::invalid_message("Content is required"))?;

        let metadata = MessageMetadata {
            message_type: self.message_type,
            priority: self.priority,
            requires_receipt: self.requires_receipt,
            expires_at: self.expires_at,
            reply_to: self.reply_to,
            thread_id: self.thread_id,
        };

        let mut message = Message {
            id: Uuid::new_v4(),
            sender,
            recipient,
            content,
            timestamp: Utc::now(),
            signature: None,
            proofs: self.proofs,
            metadata,
        };

        // Sign the message if a keypair was provided
        if let Some(keypair) = self.keypair_for_signing {
            message.sign(&keypair)?;
        }

        Ok(message)
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            message_type: MessageType::Text,
            priority: MessagePriority::Normal,
            requires_receipt: false,
            expires_at: None,
            reply_to: None,
            thread_id: None,
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message[{}] from {} to {}: \"{}\"",
            self.id,
            self.sender,
            self.recipient,
            self.content
        )
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "Text"),
            MessageType::System => write!(f, "System"),
            MessageType::Invitation => write!(f, "Invitation"),
            MessageType::Receipt => write!(f, "Receipt"),
            MessageType::Error => write!(f, "Error"),
        }
    }
}

impl std::fmt::Display for MessagePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagePriority::Low => write!(f, "Low"),
            MessagePriority::Normal => write!(f, "Normal"),
            MessagePriority::High => write!(f, "High"),
            MessagePriority::Urgent => write!(f, "Urgent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_message_creation() {
        let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
        let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
        
        let message = Message::new(
            sender_keypair.public_key().clone(),
            recipient_keypair.public_key().clone(),
            "Hello, world!".to_string(),
        );
        
        assert_eq!(message.content, "Hello, world!");
        assert!(!message.is_signed());
    }

    #[test]
    fn test_message_signing() {
        let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
        let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
        
        let mut message = Message::new(
            sender_keypair.public_key().clone(),
            recipient_keypair.public_key().clone(),
            "Hello, world!".to_string(),
        );
        
        message.sign(&sender_keypair).expect("Failed to sign message");
        assert!(message.is_signed());
        assert!(message.verify_signature().expect("Failed to verify signature"));
    }

    #[test]
    fn test_message_builder() {
        let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
        let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
        
        let message = MessageBuilder::new()
            .sender(sender_keypair.public_key().clone())
            .recipient(recipient_keypair.public_key().clone())
            .content("Hello from builder!".to_string())
            .message_type(MessageType::Text)
            .priority(MessagePriority::High)
            .requires_receipt(true)
            .sign_with(&sender_keypair)
            .expect("Failed to set signing keypair")
            .build()
            .expect("Failed to build message");
        
        assert_eq!(message.content, "Hello from builder!");
        assert_eq!(message.metadata.message_type, MessageType::Text);
        assert_eq!(message.metadata.priority, MessagePriority::High);
        assert!(message.metadata.requires_receipt);
        assert!(message.is_signed());
        assert!(message.verify_signature().expect("Failed to verify signature"));
    }

    #[test]
    fn test_message_with_proofs() {
        let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
        let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
        
        let proof = Proof::new(ProofType::Message, b"proof data".to_vec())
            .expect("Failed to create proof");
        
        let message = MessageBuilder::new()
            .sender(sender_keypair.public_key().clone())
            .recipient(recipient_keypair.public_key().clone())
            .content("Message with proof".to_string())
            .add_proof(proof)
            .build()
            .expect("Failed to build message");
        
        assert_eq!(message.proofs.len(), 1);
        assert_eq!(message.proofs[0].proof_type, ProofType::Message);
    }

    #[test]
    fn test_message_expiration() {
        let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
        let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
        
        let past_time = Utc::now() - chrono::Duration::hours(1);
        let future_time = Utc::now() + chrono::Duration::hours(1);
        
        let expired_message = MessageBuilder::new()
            .sender(sender_keypair.public_key().clone())
            .recipient(recipient_keypair.public_key().clone())
            .content("Expired message".to_string())
            .expires_at(past_time)
            .build()
            .expect("Failed to build expired message");
        
        let valid_message = MessageBuilder::new()
            .sender(sender_keypair.public_key().clone())
            .recipient(recipient_keypair.public_key().clone())
            .content("Valid message".to_string())
            .expires_at(future_time)
            .build()
            .expect("Failed to build valid message");
        
        assert!(expired_message.is_expired());
        assert!(!valid_message.is_expired());
    }
}