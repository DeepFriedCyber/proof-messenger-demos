//! Tests for secure key handling with automatic memory zeroization

use ed25519_dalek::{Signer, Verifier};
use proof_messenger_protocol::key::{
    SecureKeypair, generate_secure_keypair, generate_secure_keypair_with_seed
};

#[test]
fn test_secure_keypair_generation() {
    // ARRANGE & ACT: Generate a secure keypair
    let keypair = SecureKeypair::generate();
    
    // ASSERT: We can get the public key
    let public_key = keypair.public_key();
    let public_bytes = keypair.public_key_bytes();
    
    assert_eq!(public_key.to_bytes(), public_bytes);
    assert_eq!(public_bytes.len(), 32);
}

#[test]
fn test_secure_keypair_deterministic_generation() {
    // ARRANGE: Use the same seed
    let seed = 42u64;
    
    // ACT: Generate two keypairs with the same seed
    let keypair1 = SecureKeypair::generate_with_seed(seed);
    let keypair2 = SecureKeypair::generate_with_seed(seed);
    
    // ASSERT: They should have the same public key
    assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
}

#[test]
fn test_secure_keypair_signing() {
    // ARRANGE: Generate a keypair and a message
    let keypair = SecureKeypair::generate_with_seed(123);
    let message = b"test message for signing";
    
    // ACT: Sign the message
    let signature = keypair.sign(message);
    
    // ASSERT: The signature should be valid
    let public_key = keypair.public_key();
    assert!(public_key.verify(message, &signature).is_ok());
}

#[test]
fn test_secure_keypair_from_bytes() {
    // ARRANGE: Generate a keypair and get its bytes
    let original_keypair = SecureKeypair::generate_with_seed(456);
    let keypair_bytes = original_keypair.to_bytes();
    
    // ACT: Recreate keypair from bytes
    let recreated_keypair = SecureKeypair::from_bytes(&keypair_bytes)
        .expect("Should create keypair from valid bytes");
    
    // ASSERT: Public keys should match
    assert_eq!(
        original_keypair.public_key_bytes(),
        recreated_keypair.public_key_bytes()
    );
    
    // ASSERT: Both should sign the same message identically
    let message = b"consistency test";
    let sig1 = original_keypair.sign(message);
    let sig2 = recreated_keypair.sign(message);
    assert_eq!(sig1.to_bytes(), sig2.to_bytes());
}

#[test]
fn test_secure_keypair_from_invalid_bytes() {
    // ARRANGE: Invalid byte arrays
    let too_short = [0u8; 32];
    let too_long = [0u8; 128];
    let wrong_length = [0u8; 63];
    
    // ACT & ASSERT: Should reject invalid lengths
    assert!(SecureKeypair::from_bytes(&too_short).is_err());
    assert!(SecureKeypair::from_bytes(&too_long).is_err());
    assert!(SecureKeypair::from_bytes(&wrong_length).is_err());
}

#[test]
fn test_secure_keypair_clone() {
    // ARRANGE: Generate a keypair
    let original = SecureKeypair::generate_with_seed(789);
    
    // ACT: Clone the keypair
    let cloned = original.clone();
    
    // ASSERT: Both should have the same public key
    assert_eq!(original.public_key_bytes(), cloned.public_key_bytes());
    
    // ASSERT: Both should sign identically
    let message = b"clone test message";
    let sig1 = original.sign(message);
    let sig2 = cloned.sign(message);
    assert_eq!(sig1.to_bytes(), sig2.to_bytes());
}

#[test]
fn test_secure_keypair_as_keypair_compatibility() {
    // ARRANGE: Generate a secure keypair
    let secure_keypair = SecureKeypair::generate_with_seed(999);
    
    // ACT: Get the underlying keypair for compatibility
    let regular_keypair = secure_keypair.as_keypair();
    
    // ASSERT: Public keys should match
    assert_eq!(
        secure_keypair.public_key_bytes(),
        regular_keypair.public.to_bytes()
    );
    
    // ASSERT: Signatures should match
    let message = b"compatibility test";
    let secure_sig = secure_keypair.sign(message);
    let regular_sig = regular_keypair.sign(message);
    assert_eq!(secure_sig.to_bytes(), regular_sig.to_bytes());
}

#[test]
fn test_convenience_functions() {
    // ARRANGE & ACT: Use convenience functions
    let keypair1 = generate_secure_keypair();
    let keypair2 = generate_secure_keypair_with_seed(111);
    let keypair3 = generate_secure_keypair_with_seed(111);
    
    // ASSERT: Random generation produces different keys
    assert_ne!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    
    // ASSERT: Seeded generation is deterministic
    assert_eq!(keypair2.public_key_bytes(), keypair3.public_key_bytes());
}

#[test]
fn test_memory_safety_simulation() {
    // This test simulates the memory safety benefits of SecureKeypair
    // In a real scenario, the zeroize would prevent key recovery from memory dumps
    
    let message = b"memory safety test";
    let signature_bytes: [u8; 64];
    let public_key_bytes: [u8; 32];
    
    // ARRANGE: Create a scope where the keypair exists
    {
        let keypair = SecureKeypair::generate_with_seed(555);
        signature_bytes = keypair.sign(message).to_bytes();
        public_key_bytes = keypair.public_key_bytes();
        
        // The keypair will be dropped and zeroed here
    }
    
    // ACT & ASSERT: We can still verify the signature with the public key
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .expect("Valid public key");
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes)
        .expect("Valid signature");
    
    assert!(public_key.verify(message, &signature).is_ok());
    
    // The private key material has been securely zeroed from memory
    // This prevents potential recovery by memory analysis tools
}

#[test]
fn test_different_keypairs_produce_different_signatures() {
    // ARRANGE: Generate two different keypairs
    let keypair1 = SecureKeypair::generate_with_seed(100);
    let keypair2 = SecureKeypair::generate_with_seed(200);
    let message = b"uniqueness test";
    
    // ACT: Sign the same message with both keypairs
    let sig1 = keypair1.sign(message);
    let sig2 = keypair2.sign(message);
    
    // ASSERT: Signatures should be different
    assert_ne!(sig1.to_bytes(), sig2.to_bytes());
    
    // ASSERT: Each signature is only valid for its corresponding public key
    assert!(keypair1.public_key().verify(message, &sig1).is_ok());
    assert!(keypair1.public_key().verify(message, &sig2).is_err());
    assert!(keypair2.public_key().verify(message, &sig2).is_ok());
    assert!(keypair2.public_key().verify(message, &sig1).is_err());
}

#[test]
fn test_secure_keypair_to_bytes_roundtrip() {
    // ARRANGE: Generate a keypair
    let original = SecureKeypair::generate_with_seed(333);
    
    // ACT: Convert to bytes and back
    let bytes = original.to_bytes();
    let reconstructed = SecureKeypair::from_bytes(&bytes)
        .expect("Should reconstruct from valid bytes");
    
    // ASSERT: Should be functionally identical
    assert_eq!(original.public_key_bytes(), reconstructed.public_key_bytes());
    
    let message = b"roundtrip test";
    let sig1 = original.sign(message);
    let sig2 = reconstructed.sign(message);
    assert_eq!(sig1.to_bytes(), sig2.to_bytes());
}

#[test]
fn secure_keypair_can_sign_and_verify() {
    // Conceptual test: Ensures the SecureKeypair wrapper functions correctly
    // Note: This test validates functional correctness, not memory zeroization
    // Memory zeroization assurance comes from the zeroize crate's own guarantees
    let secure_kp = SecureKeypair::generate();
    let kp = secure_kp.as_keypair();
    let context = b"test context";
    let signature = kp.sign(context);
    assert!(kp.verify(context, &signature).is_ok());
}