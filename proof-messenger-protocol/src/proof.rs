use ed25519_dalek::{Keypair, PublicKey, Signature, SignatureError, Signer, Verifier};
use thiserror::Error;

/// Dedicated error enum for proof operations
#[derive(Debug, Error)]
pub enum ProofError {
    /// Proof verification failed due to invalid signature
    #[error("Proof verification failed: invalid signature")]
    VerificationFailed(#[from] SignatureError),
    
    /// Invalid proof data or format
    #[error("Invalid proof data: {0}")]
    InvalidData(String),
    
    /// Proof generation failed
    #[error("Proof generation failed: {0}")]
    GenerationFailed(String),
}

#[derive(Clone)]
pub struct Invite {
    pub data: Vec<u8>, // e.g., group ID, timestamp, etc.
}

impl Invite {
    pub fn new_with_seed(seed: u64) -> Self {
        let data = seed.to_be_bytes().to_vec();
        Invite { data }
    }
}

// Legacy API for Invite-based proofs
pub fn make_proof(keypair: &Keypair, invite: &Invite) -> Signature {
    keypair.sign(&invite.data)
}

pub fn verify_proof(sig: &Signature, public: &PublicKey, invite: &Invite) -> bool {
    public.verify(&invite.data, sig).is_ok()
}

// TDD Step 2: New Result-based API for better error handling

/// Create a proof (signature) for arbitrary context data
pub fn make_proof_context(keypair: &Keypair, context: &[u8]) -> Signature {
    keypair.sign(context)
}

/// Verify a proof with Result-based error handling
/// 
/// This function returns a Result instead of a bool, providing detailed
/// error information when verification fails.
pub fn verify_proof_result(
    pubkey: &PublicKey,
    context: &[u8],
    sig: &Signature,
) -> Result<(), ProofError> {
    pubkey.verify(context, sig)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::generate_keypair_with_seed;

    #[test]
    fn test_proof_roundtrip() {
        let keypair = generate_keypair_with_seed(42);
        let invite = Invite::new_with_seed(123);
        
        let sig = make_proof(&keypair, &invite);
        assert!(verify_proof(&sig, &keypair.public, &invite));
    }

    #[test]
    fn test_proof_fails_with_wrong_key() {
        let keypair1 = generate_keypair_with_seed(42);
        let keypair2 = generate_keypair_with_seed(43);
        let invite = Invite::new_with_seed(123);
        
        let sig = make_proof(&keypair1, &invite);
        assert!(!verify_proof(&sig, &keypair2.public, &invite));
    }

    #[test]
    fn test_proof_fails_with_tampered_invite() {
        let keypair = generate_keypair_with_seed(42);
        let invite = Invite::new_with_seed(123);
        let mut tampered_invite = Invite::new_with_seed(123);
        tampered_invite.data.push(0xFF); // Tamper with the data
        
        let sig = make_proof(&keypair, &invite);
        assert!(!verify_proof(&sig, &keypair.public, &tampered_invite));
    }

    #[test]
    fn test_invite_from_seed() {
        let invite1 = Invite::new_with_seed(42);
        let invite2 = Invite::new_with_seed(42);
        let invite3 = Invite::new_with_seed(43);
        
        // Same seed should produce same data
        assert_eq!(invite1.data, invite2.data);
        // Different seed should produce different data
        assert_ne!(invite1.data, invite3.data);
    }

    // TDD Step 1: Write the failing test first
    #[test]
    fn verify_proof_returns_specific_error_on_failure() {
        // ARRANGE: Create two different keypairs
        let signing_keypair = crate::key::generate_keypair_with_seed(42);
        let wrong_keypair = crate::key::generate_keypair_with_seed(43);
        let context = b"a critical message";
        
        // Create a proof with the first keypair
        let signature = make_proof_context(&signing_keypair, context);
        
        // ACT: Attempt to verify the proof with the second keypair's public key
        let result = verify_proof_result(&wrong_keypair.public, context, &signature);
        
        // ASSERT: The function should fail, and the error should be the specific
        // `VerificationFailed` variant of our new `ProofError` enum
        assert!(matches!(result, Err(ProofError::VerificationFailed(_))));
    }

    #[test]
    fn verify_proof_succeeds_with_correct_key() {
        // ARRANGE: Create keypair and context
        let keypair = crate::key::generate_keypair_with_seed(42);
        let context = b"a critical message";
        
        // Create a proof with the keypair
        let signature = make_proof_context(&keypair, context);
        
        // ACT: Verify the proof with the correct public key
        let result = verify_proof_result(&keypair.public, context, &signature);
        
        // ASSERT: The function should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn verify_proof_fails_with_tampered_context() {
        // ARRANGE: Create keypair and context
        let keypair = crate::key::generate_keypair_with_seed(42);
        let original_context = b"original message";
        let tampered_context = b"tampered message";
        
        // Create a proof with the original context
        let signature = make_proof_context(&keypair, original_context);
        
        // ACT: Attempt to verify the proof with tampered context
        let result = verify_proof_result(&keypair.public, tampered_context, &signature);
        
        // ASSERT: The function should fail with verification error
        assert!(matches!(result, Err(ProofError::VerificationFailed(_))));
    }

    #[test]
    fn test_error_message_content() {
        // Test that error messages are informative
        let keypair1 = crate::key::generate_keypair_with_seed(42);
        let keypair2 = crate::key::generate_keypair_with_seed(43);
        let context = b"test message";
        
        let signature = make_proof_context(&keypair1, context);
        let result = verify_proof_result(&keypair2.public, context, &signature);
        
        match result {
            Err(ProofError::VerificationFailed(sig_err)) => {
                // Verify that we get the underlying signature error
                let error_string = format!("{}", sig_err);
                assert!(!error_string.is_empty());
            }
            _ => panic!("Expected VerificationFailed error"),
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        // Test that errors can be properly debugged
        let keypair1 = crate::key::generate_keypair_with_seed(42);
        let keypair2 = crate::key::generate_keypair_with_seed(43);
        let context = b"debug test";
        
        let signature = make_proof_context(&keypair1, context);
        let result = verify_proof_result(&keypair2.public, context, &signature);
        
        match result {
            Err(err) => {
                let debug_string = format!("{:?}", err);
                assert!(debug_string.contains("VerificationFailed"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        // Test that errors have user-friendly display messages
        let keypair1 = crate::key::generate_keypair_with_seed(42);
        let keypair2 = crate::key::generate_keypair_with_seed(43);
        let context = b"display test";
        
        let signature = make_proof_context(&keypair1, context);
        let result = verify_proof_result(&keypair2.public, context, &signature);
        
        match result {
            Err(err) => {
                let display_string = format!("{}", err);
                assert!(display_string.contains("Proof verification failed"));
                assert!(display_string.contains("invalid signature"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_backwards_compatibility() {
        // Ensure the old API still works alongside the new one
        let keypair = crate::key::generate_keypair_with_seed(42);
        let invite = Invite::new_with_seed(123);
        
        // Old API
        let sig_old = make_proof(&keypair, &invite);
        assert!(verify_proof(&sig_old, &keypair.public, &invite));
        
        // New API with same data
        let sig_new = make_proof_context(&keypair, &invite.data);
        let result = verify_proof_result(&keypair.public, &invite.data, &sig_new);
        assert!(result.is_ok());
        
        // Cross-compatibility: signature from old API should work with new verification
        let result_cross = verify_proof_result(&keypair.public, &invite.data, &sig_old);
        assert!(result_cross.is_ok());
    }
}