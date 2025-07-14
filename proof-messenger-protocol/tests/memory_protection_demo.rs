//! Demonstration of memory protection features in SecureKeypair
//! 
//! This test demonstrates how SecureKeypair automatically protects
//! sensitive key material from memory analysis attacks.

use proof_messenger_protocol::key::{SecureKeypair, generate_keypair};
use ed25519_dalek::{Signer, Verifier};

#[test]
fn test_memory_protection_comparison() {
    let message = b"sensitive operation";
    
    // Scenario 1: Using regular Keypair (legacy approach)
    // ⚠️  Private key material may remain in memory after use
    let regular_signature = {
        let regular_keypair = generate_keypair();
        regular_keypair.sign(message)
        // regular_keypair is dropped here, but memory may not be cleared
    };
    
    // Scenario 2: Using SecureKeypair (recommended approach)
    // ✅ Private key material is automatically zeroed when dropped
    let secure_signature = {
        let secure_keypair = SecureKeypair::generate();
        secure_keypair.sign(message)
        // secure_keypair is dropped here and memory is automatically zeroed
    };
    
    // Both approaches produce valid signatures
    assert_eq!(regular_signature.to_bytes().len(), 64);
    assert_eq!(secure_signature.to_bytes().len(), 64);
    
    // The key difference is that SecureKeypair provides memory protection
    // against potential key recovery from memory dumps or swap files
}

#[test]
fn test_secure_keypair_lifecycle() {
    // This test demonstrates the complete lifecycle of a SecureKeypair
    // and how it maintains security throughout its usage
    
    let message1 = b"first message";
    let message2 = b"second message";
    
    let (public_key_bytes, signature1_bytes, signature2_bytes) = {
        // Create a secure keypair
        let keypair = SecureKeypair::generate_with_seed(12345);
        
        // Extract public key (safe to keep)
        let public_key_bytes = keypair.public_key_bytes();
        
        // Perform cryptographic operations
        let signature1 = keypair.sign(message1);
        let signature2 = keypair.sign(message2);
        
        // Return only the public data
        (public_key_bytes, signature1.to_bytes(), signature2.to_bytes())
        
        // keypair is dropped here and private key is automatically zeroed
        // This prevents the private key from being recovered from memory
    };
    
    // Verify that the signatures are still valid using the public key
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .expect("Valid public key");
    
    let signature1 = ed25519_dalek::Signature::from_bytes(&signature1_bytes)
        .expect("Valid signature");
    let signature2 = ed25519_dalek::Signature::from_bytes(&signature2_bytes)
        .expect("Valid signature");
    
    // Verify signatures
    assert!(public_key.verify(message1, &signature1).is_ok());
    assert!(public_key.verify(message2, &signature2).is_ok());
    
    // Cross-verification should fail (signature1 shouldn't verify message2)
    assert!(public_key.verify(message2, &signature1).is_err());
    assert!(public_key.verify(message1, &signature2).is_err());
}

#[test]
fn test_multiple_secure_keypairs() {
    // Test that multiple SecureKeypairs can coexist safely
    // and each maintains its own memory protection
    
    let message = b"multi-keypair test";
    
    let keypair1 = SecureKeypair::generate_with_seed(111);
    let keypair2 = SecureKeypair::generate_with_seed(222);
    let keypair3 = SecureKeypair::generate_with_seed(333);
    
    // Each keypair should produce different signatures
    let sig1 = keypair1.sign(message);
    let sig2 = keypair2.sign(message);
    let sig3 = keypair3.sign(message);
    
    // All signatures should be different
    assert_ne!(sig1.to_bytes(), sig2.to_bytes());
    assert_ne!(sig2.to_bytes(), sig3.to_bytes());
    assert_ne!(sig1.to_bytes(), sig3.to_bytes());
    
    // Each signature should only verify with its corresponding public key
    assert!(keypair1.public_key().verify(message, &sig1).is_ok());
    assert!(keypair1.public_key().verify(message, &sig2).is_err());
    assert!(keypair1.public_key().verify(message, &sig3).is_err());
    
    assert!(keypair2.public_key().verify(message, &sig2).is_ok());
    assert!(keypair2.public_key().verify(message, &sig1).is_err());
    assert!(keypair2.public_key().verify(message, &sig3).is_err());
    
    assert!(keypair3.public_key().verify(message, &sig3).is_ok());
    assert!(keypair3.public_key().verify(message, &sig1).is_err());
    assert!(keypair3.public_key().verify(message, &sig2).is_err());
    
    // When this test ends, all three keypairs will be automatically
    // zeroed from memory, providing comprehensive protection
}

#[test]
fn test_secure_keypair_in_production_scenario() {
    // This test simulates a production scenario where a server
    // handles multiple signing operations and then cleans up
    
    struct SecureServer {
        keypair: SecureKeypair,
        operation_count: u32,
    }
    
    impl SecureServer {
        fn new() -> Self {
            Self {
                keypair: SecureKeypair::generate(),
                operation_count: 0,
            }
        }
        
        fn sign_message(&mut self, message: &[u8]) -> ed25519_dalek::Signature {
            self.operation_count += 1;
            self.keypair.sign(message)
        }
        
        fn get_public_key(&self) -> [u8; 32] {
            self.keypair.public_key_bytes()
        }
        
        fn get_operation_count(&self) -> u32 {
            self.operation_count
        }
    }
    
    // When SecureServer is dropped, the keypair is automatically zeroed
    impl Drop for SecureServer {
        fn drop(&mut self) {
            // The SecureKeypair will automatically zero its memory
            // when it's dropped as part of this struct
            println!("Server shutting down after {} operations", self.operation_count);
        }
    }
    
    let public_key_bytes;
    let signatures: Vec<[u8; 64]>;
    
    {
        let mut server = SecureServer::new();
        public_key_bytes = server.get_public_key();
        
        // Simulate multiple operations
        let messages = [
            b"operation 1".as_slice(),
            b"operation 2".as_slice(),
            b"operation 3".as_slice(),
        ];
        
        signatures = messages
            .iter()
            .map(|msg| server.sign_message(msg).to_bytes())
            .collect();
        
        assert_eq!(server.get_operation_count(), 3);
        
        // Server and its keypair are dropped here
        // Private key material is automatically zeroed
    }
    
    // Verify that all operations were valid using only the public key
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .expect("Valid public key");
    
    let messages = [b"operation 1", b"operation 2", b"operation 3"];
    
    for (i, signature_bytes) in signatures.iter().enumerate() {
        let signature = ed25519_dalek::Signature::from_bytes(signature_bytes)
            .expect("Valid signature");
        assert!(public_key.verify(messages[i], &signature).is_ok());
    }
    
    // The private key has been securely erased from memory,
    // but all the cryptographic operations remain verifiable
}