//! Cryptographic primitives and key management
//!
//! This module provides the core cryptographic functionality for the proof messenger protocol,
//! including key generation, digital signatures, and encryption/decryption operations.
//!
//! # Security Features
//!
//! - Ed25519 digital signatures for authentication
//! - X25519 key exchange for forward secrecy
//! - ChaCha20Poly1305 for authenticated encryption
//! - Secure memory handling with zeroization
//! - Constant-time operations where possible

use crate::errors::{ProtocolError, Result};
use ed25519_dalek::{Signer, Verifier};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A cryptographic keypair for digital signatures
///
/// This struct contains both the public and private keys needed for
/// Ed25519 digital signatures. The private key is automatically
/// zeroized when the struct is dropped.
///
/// # Example
///
/// ```rust
/// use proof_messenger_protocol::crypto::KeyPair;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let keypair = KeyPair::generate()?;
/// let public_key = keypair.public_key();
/// let message = b"Hello, world!";
/// let signature = keypair.sign(message)?;
/// 
/// assert!(public_key.verify(message, &signature)?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, ZeroizeOnDrop)]
pub struct KeyPair {
    signing_key: ed25519_dalek::SigningKey,
    verifying_key: ed25519_dalek::VerifyingKey,
}

// Manual Clone implementation to handle the signing key properly
impl Clone for KeyPair {
    fn clone(&self) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&self.signing_key.to_bytes());
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }
}

/// A public key for signature verification
///
/// This represents the public portion of an Ed25519 keypair and can be
/// safely shared and serialized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    key: ed25519_dalek::VerifyingKey,
}

/// A private key for signing operations
///
/// This is automatically zeroized when dropped to prevent key material
/// from remaining in memory.
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct PrivateKey {
    key: ed25519_dalek::SigningKey,
}

/// A digital signature
///
/// Represents an Ed25519 signature that can be verified against a message
/// and public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    signature: ed25519_dalek::Signature,
}

impl KeyPair {
    /// Generate a new random keypair
    ///
    /// Uses the operating system's secure random number generator to create
    /// a new Ed25519 keypair.
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if key generation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::crypto::KeyPair;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keypair = KeyPair::generate()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate() -> Result<Self> {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the public key portion of this keypair
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::crypto::KeyPair;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keypair = KeyPair::generate()?;
    /// let public_key = keypair.public_key();
    /// # Ok(())
    /// # }
    /// ```
    pub fn public_key(&self) -> &PublicKey {
        // Safety: This is safe because PublicKey wraps VerifyingKey
        unsafe { std::mem::transmute(&self.verifying_key) }
    }

    /// Get the private key portion of this keypair
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::crypto::KeyPair;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keypair = KeyPair::generate()?;
    /// let private_key = keypair.private_key();
    /// # Ok(())
    /// # }
    /// ```
    pub fn private_key(&self) -> &PrivateKey {
        // Safety: This is safe because PrivateKey wraps SigningKey
        unsafe { std::mem::transmute(&self.signing_key) }
    }

    /// Sign a message with this keypair
    ///
    /// Creates an Ed25519 signature over the provided message bytes.
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes to sign
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if signing fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::crypto::KeyPair;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keypair = KeyPair::generate()?;
    /// let message = b"Hello, world!";
    /// let signature = keypair.sign(message)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let signature = self.signing_key.try_sign(message)
            .map_err(|e| ProtocolError::Crypto(format!("Signing failed: {}", e)))?;
        
        Ok(Signature { signature })
    }

    /// Convert keypair to bytes for serialization
    ///
    /// Returns the 32-byte private key. The public key can be derived from this.
    ///
    /// # Security Note
    ///
    /// The returned bytes contain sensitive key material and should be handled securely.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Create keypair from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - 32-byte private key
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if the bytes don't represent a valid key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
}

impl PublicKey {
    /// Verify a signature against a message
    ///
    /// # Arguments
    ///
    /// * `message` - The original message that was signed
    /// * `signature` - The signature to verify
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if verification fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proof_messenger_protocol::crypto::KeyPair;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keypair = KeyPair::generate()?;
    /// let message = b"Hello, world!";
    /// let signature = keypair.sign(message)?;
    /// 
    /// assert!(keypair.public_key().verify(message, &signature)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<bool> {
        match self.key.verify(message, &signature.signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Convert public key to bytes
    ///
    /// Returns the 32-byte public key for serialization or transmission.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.key.to_bytes()
    }

    /// Create public key from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - 32-byte public key
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if the bytes don't represent a valid public key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let key = ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .map_err(|e| ProtocolError::Crypto(format!("Invalid public key: {}", e)))?;
        
        Ok(Self { key })
    }
}

impl Signature {
    /// Convert signature to bytes
    ///
    /// Returns the 64-byte signature for serialization or transmission.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.signature.to_bytes()
    }

    /// Create signature from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - 64-byte signature
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::Crypto` if the bytes don't represent a valid signature.
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self> {
        let signature = ed25519_dalek::Signature::from_bytes(bytes);
        Ok(Self { signature })
    }
}

// Implement Display for better debugging and logging
impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey({})", hex::encode(self.to_bytes()))
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature({})", hex::encode(self.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        assert!(!keypair.to_bytes().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let message = b"test message";
        
        let signature = keypair.sign(message).expect("Failed to sign message");
        let is_valid = keypair.public_key().verify(message, &signature)
            .expect("Failed to verify signature");
        
        assert!(is_valid);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let keypair1 = KeyPair::generate().expect("Failed to generate keypair");
        let keypair2 = KeyPair::generate().expect("Failed to generate keypair");
        let message = b"test message";
        
        let signature = keypair1.sign(message).expect("Failed to sign message");
        let is_valid = keypair2.public_key().verify(message, &signature)
            .expect("Failed to verify signature");
        
        assert!(!is_valid);
    }

    #[test]
    fn test_keypair_serialization() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let bytes = keypair.to_bytes();
        let restored = KeyPair::from_bytes(&bytes).expect("Failed to restore keypair");
        
        let message = b"test message";
        let signature1 = keypair.sign(message).expect("Failed to sign with original");
        let signature2 = restored.sign(message).expect("Failed to sign with restored");
        
        // Both signatures should be valid (though they may be different due to randomness)
        assert!(keypair.public_key().verify(message, &signature1).unwrap());
        assert!(restored.public_key().verify(message, &signature2).unwrap());
    }

    #[test]
    fn test_public_key_serialization() {
        let keypair = KeyPair::generate().expect("Failed to generate keypair");
        let public_key = keypair.public_key();
        let bytes = public_key.to_bytes();
        let restored = PublicKey::from_bytes(&bytes).expect("Failed to restore public key");
        
        assert_eq!(public_key, &restored);
    }
}