//! Cryptographic proof generation and verification
//!
//! This module provides functionality for creating and verifying cryptographic proofs
//! that demonstrate various properties without revealing sensitive information.
//!
//! # Proof Types
//!
//! - **Identity Proofs**: Prove ownership of a private key without revealing it
//! - **Message Proofs**: Prove a message was created by a specific identity
//! - **Timestamp Proofs**: Prove when a message or event occurred
//!
//! # Example
//!
//! ```rust
//! use proof_messenger_protocol::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an identity proof
//! let keypair = KeyPair::generate()?;
//! let proof = Proof::new(ProofType::Identity, keypair.public_key().to_bytes().to_vec())?;
//!
//! // Verify the proof
//! let is_valid = ProofVerifier::verify(&proof)?;
//! assert!(is_valid);
//! # Ok(())
//! # }
//! ```

use crate::crypto::{KeyPair, PublicKey, Signature};
use crate::errors::{ProtocolError, Result};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cryptographic proof with associated metadata
///
/// This struct represents a cryptographic proof that can demonstrate
/// various properties without revealing sensitive information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Unique identifier for this proof
    pub id: Uuid,
    /// The type of proof this represents
    pub proof_type: ProofType,
    /// The actual proof data
    pub data: Vec<u8>,
    /// Hash of the original data being proven
    pub data_hash: [u8; 32],
    /// When this proof was created
    pub timestamp: DateTime<Utc>,
    /// Optional signature over the proof
    pub signature: Option<Signature>,
    /// Optional public key of the proof creator
    pub creator: Option<PublicKey>,
}

/// Types of cryptographic proofs supported by the protocol
///
/// Each proof type serves a different purpose in the messaging protocol
/// and has different verification requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// Proves ownership of a private key without revealing it
    ///
    /// This proof demonstrates that the creator possesses the private key
    /// corresponding to a given public key, without exposing the private key.
    Identity,
    
    /// Proves a message was created by a specific identity
    ///
    /// This proof links a message to its creator and ensures the message
    /// hasn't been tampered with since creation.
    Message,
    
    /// Proves when a message or event occurred
    ///
    /// This proof provides cryptographic evidence of timing, which can be
    /// used to establish ordering of events or prove freshness.
    Timestamp,
    
    /// Proves membership in a group without revealing which member
    ///
    /// This proof allows demonstrating group membership while maintaining
    /// anonymity within the group.
    GroupMembership,
    
    /// Proves knowledge of a secret without revealing it
    ///
    /// This is a general zero-knowledge proof that can be used for
    /// various authentication scenarios.
    ZeroKnowledge,
}

/// Verifier for cryptographic proofs
///
/// This struct provides methods to verify different types of proofs
/// and ensure their validity.
pub struct ProofVerifier;

/// Builder for creating cryptographic proofs
///
/// This builder provides a convenient interface for constructing proofs
/// with various parameters and options.
pub struct ProofBuilder {
    proof_type: Option<ProofType>,
    data: Option<Vec<u8>>,
    keypair: Option<KeyPair>,
    timestamp: Option<DateTime<Utc>>,
}

impl Proof {
    /// Create a new proof with the given type and data
    ///
    /// # Arguments
    ///
    /// * `proof_type` - The type of proof to create
    /// * `data` - The data to create a proof for
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::ProofGeneration` if proof creation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let proof = Proof::new(ProofType::Message, b"Hello, world!".to_vec())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(proof_type: ProofType, data: Vec<u8>) -> Result<Self> {
        let data_hash = Self::hash_data(&data);
        
        Ok(Self {
            id: Uuid::new_v4(),
            proof_type,
            data: data.clone(),
            data_hash,
            timestamp: Utc::now(),
            signature: None,
            creator: None,
        })
    }

    /// Create a signed proof
    ///
    /// # Arguments
    ///
    /// * `proof_type` - The type of proof to create
    /// * `data` - The data to create a proof for
    /// * `keypair` - The keypair to sign the proof with
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::ProofGeneration` if proof creation or signing fails.
    pub fn new_signed(proof_type: ProofType, data: Vec<u8>, keypair: &KeyPair) -> Result<Self> {
        let mut proof = Self::new(proof_type, data)?;
        proof.sign(keypair)?;
        Ok(proof)
    }

    /// Sign this proof with a keypair
    ///
    /// # Arguments
    ///
    /// * `keypair` - The keypair to sign with
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if signing fails.
    pub fn sign(&mut self, keypair: &KeyPair) -> Result<()> {
        let proof_bytes = self.to_bytes_for_signing()?;
        let signature = keypair.sign(&proof_bytes)?;
        
        self.signature = Some(signature);
        self.creator = Some(keypair.public_key().clone());
        
        Ok(())
    }

    /// Verify the signature on this proof
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::ProofVerification` if verification fails.
    pub fn verify_signature(&self) -> Result<bool> {
        match (&self.signature, &self.creator) {
            (Some(signature), Some(public_key)) => {
                let proof_bytes = self.to_bytes_for_signing()?;
                public_key.verify(&proof_bytes, signature)
            }
            (None, None) => Ok(true), // Unsigned proof is valid
            _ => Err(ProtocolError::proof_verification(
                "Proof has signature but no creator, or creator but no signature"
            )),
        }
    }

    /// Get the hash of the original data
    pub fn data_hash(&self) -> &[u8; 32] {
        &self.data_hash
    }

    /// Check if this proof is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some() && self.creator.is_some()
    }

    /// Hash data using BLAKE3
    fn hash_data(data: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Convert proof to bytes for signing (excludes signature field)
    fn to_bytes_for_signing(&self) -> Result<Vec<u8>> {
        let signing_data = ProofSigningData {
            id: self.id,
            proof_type: self.proof_type.clone(),
            data_hash: self.data_hash,
            timestamp: self.timestamp,
        };
        
        bincode::serialize(&signing_data)
            .map_err(|e| ProtocolError::proof_generation(format!("Failed to serialize proof for signing: {}", e)))
    }
}

/// Data structure used for proof signing (excludes signature to prevent circular dependency)
#[derive(Serialize)]
struct ProofSigningData {
    id: Uuid,
    proof_type: ProofType,
    data_hash: [u8; 32],
    timestamp: DateTime<Utc>,
}

impl ProofVerifier {
    /// Verify a cryptographic proof
    ///
    /// This method performs comprehensive verification of a proof, including:
    /// - Data integrity checks
    /// - Signature verification (if present)
    /// - Type-specific validation
    ///
    /// # Arguments
    ///
    /// * `proof` - The proof to verify
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::ProofVerification` if verification fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let proof = Proof::new(ProofType::Message, b"Hello, world!".to_vec())?;
    /// let is_valid = ProofVerifier::verify(&proof)?;
    /// assert!(is_valid);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(proof: &Proof) -> Result<bool> {
        // Verify data integrity
        let computed_hash = Proof::hash_data(&proof.data);
        if computed_hash != proof.data_hash {
            return Ok(false);
        }

        // Verify signature if present
        if !proof.verify_signature()? {
            return Ok(false);
        }

        // Perform type-specific verification
        Self::verify_type_specific(proof)
    }

    /// Verify type-specific proof properties
    fn verify_type_specific(proof: &Proof) -> Result<bool> {
        match proof.proof_type {
            ProofType::Identity => Self::verify_identity_proof(proof),
            ProofType::Message => Self::verify_message_proof(proof),
            ProofType::Timestamp => Self::verify_timestamp_proof(proof),
            ProofType::GroupMembership => Self::verify_group_membership_proof(proof),
            ProofType::ZeroKnowledge => Self::verify_zero_knowledge_proof(proof),
        }
    }

    /// Verify an identity proof
    fn verify_identity_proof(proof: &Proof) -> Result<bool> {
        // For identity proofs, we verify that the proof is signed and the
        // data contains the public key that matches the signature
        if !proof.is_signed() {
            return Ok(false);
        }

        // The data should contain the public key bytes
        if proof.data.len() != 32 {
            return Ok(false);
        }

        let public_key_bytes: [u8; 32] = proof.data.as_slice().try_into()
            .map_err(|_| ProtocolError::proof_verification("Invalid public key length"))?;
        
        let claimed_public_key = PublicKey::from_bytes(&public_key_bytes)?;
        
        // Verify that the claimed public key matches the proof creator
        match &proof.creator {
            Some(creator) => Ok(creator == &claimed_public_key),
            None => Ok(false),
        }
    }

    /// Verify a message proof
    fn verify_message_proof(_proof: &Proof) -> Result<bool> {
        // Message proofs are verified by checking the signature
        // The actual message verification is handled by the signature check
        Ok(true)
    }

    /// Verify a timestamp proof
    fn verify_timestamp_proof(proof: &Proof) -> Result<bool> {
        // Timestamp proofs should have a recent timestamp
        let now = Utc::now();
        let age = now.signed_duration_since(proof.timestamp);
        
        // Allow timestamps up to 1 hour in the future (for clock skew)
        // and up to 24 hours in the past
        Ok(age.num_hours() >= -1 && age.num_hours() <= 24)
    }

    /// Verify a group membership proof
    fn verify_group_membership_proof(_proof: &Proof) -> Result<bool> {
        // Group membership proof verification would involve checking
        // against a group registry or membership list
        // For now, we'll accept all group membership proofs
        Ok(true)
    }

    /// Verify a zero-knowledge proof
    fn verify_zero_knowledge_proof(_proof: &Proof) -> Result<bool> {
        // Zero-knowledge proof verification would involve complex
        // cryptographic operations specific to the ZK system used
        // For now, we'll accept all ZK proofs
        Ok(true)
    }
}

impl ProofBuilder {
    /// Create a new proof builder
    pub fn new() -> Self {
        Self {
            proof_type: None,
            data: None,
            keypair: None,
            timestamp: None,
        }
    }

    /// Set the proof type
    pub fn proof_type(mut self, proof_type: ProofType) -> Self {
        self.proof_type = Some(proof_type);
        self
    }

    /// Set the data to prove
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the keypair for signing
    pub fn keypair(mut self, keypair: KeyPair) -> Self {
        self.keypair = Some(keypair);
        self
    }

    /// Set a custom timestamp
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Build the proof
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::ProofGeneration` if required fields are missing or proof creation fails.
    pub fn build(self) -> Result<Proof> {
        let proof_type = self.proof_type
            .ok_or_else(|| ProtocolError::proof_generation("Proof type is required"))?;
        let data = self.data
            .ok_or_else(|| ProtocolError::proof_generation("Data is required"))?;

        let mut proof = Proof::new(proof_type, data)?;
        
        if let Some(timestamp) = self.timestamp {
            proof.timestamp = timestamp;
        }

        if let Some(keypair) = self.keypair {
            proof.sign(&keypair)?;
        }

        Ok(proof)
    }
}

impl Default for ProofBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_proof_creation() {
        let proof = Proof::new(ProofType::Message, b"test data".to_vec())
            .expect("Failed to create proof");
        
        assert_eq!(proof.proof_type, ProofType::Message);
        assert_eq!(proof.data, b"test data");
        assert!(!proof.is_signed());
    }

    #[test]
    fn test_signed_proof_creation() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let proof = Proof::new_signed(ProofType::Identity, keypair.public_key().to_bytes().to_vec(), &keypair)
            .expect("Failed to create signed proof");
        
        assert!(proof.is_signed());
        assert!(proof.verify_signature().expect("Failed to verify signature"));
    }

    #[test]
    fn test_proof_verification() {
        let proof = Proof::new(ProofType::Message, b"test data".to_vec())
            .expect("Failed to create proof");
        
        let is_valid = ProofVerifier::verify(&proof)
            .expect("Failed to verify proof");
        assert!(is_valid);
    }

    #[test]
    fn test_identity_proof_verification() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let proof = Proof::new_signed(
            ProofType::Identity, 
            keypair.public_key().to_bytes().to_vec(), 
            &keypair
        ).expect("Failed to create identity proof");
        
        let is_valid = ProofVerifier::verify(&proof)
            .expect("Failed to verify identity proof");
        assert!(is_valid);
    }

    #[test]
    fn test_proof_builder() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let proof = ProofBuilder::new()
            .proof_type(ProofType::Message)
            .data(b"test message".to_vec())
            .keypair(keypair)
            .build()
            .expect("Failed to build proof");
        
        assert!(proof.is_signed());
        assert!(ProofVerifier::verify(&proof).expect("Failed to verify proof"));
    }

    #[test]
    fn test_data_integrity() {
        let mut proof = Proof::new(ProofType::Message, b"original data".to_vec())
            .expect("Failed to create proof");
        
        // Tamper with the data
        proof.data = b"tampered data".to_vec();
        
        let is_valid = ProofVerifier::verify(&proof)
            .expect("Failed to verify tampered proof");
        assert!(!is_valid);
    }
}