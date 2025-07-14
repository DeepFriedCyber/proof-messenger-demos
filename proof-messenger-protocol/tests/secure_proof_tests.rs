//! Tests for secure proof generation and validation

use proof_messenger_protocol::key::generate_secure_keypair_with_seed;
use proof_messenger_protocol::proof::{
    make_secure_proof, make_secure_proof_strict, verify_proof_secure, verify_proof_strict,
    ProofError, Invite, MAX_CONTEXT_SIZE
};
use ed25519_dalek::Verifier;

#[test]
fn test_secure_proof_generation() {
    // ARRANGE: Create a secure keypair and context
    let keypair = generate_secure_keypair_with_seed(42);
    let context = b"test message for secure proof";
    
    // ACT: Generate a secure proof
    let result = make_secure_proof(&keypair, context);
    
    // ASSERT: Proof generation should succeed
    assert!(result.is_ok());
    let signature = result.unwrap();
    
    // ASSERT: Signature should be valid
    let public_key = keypair.public_key();
    assert!(public_key.verify(context, &signature).is_ok());
}

#[test]
fn test_secure_proof_strict_generation() {
    // ARRANGE: Create a secure keypair and non-empty context
    let keypair = generate_secure_keypair_with_seed(123);
    let context = b"non-empty context";
    
    // ACT: Generate a strict secure proof
    let result = make_secure_proof_strict(&keypair, context);
    
    // ASSERT: Proof generation should succeed
    assert!(result.is_ok());
    let signature = result.unwrap();
    
    // ASSERT: Signature should be valid
    let public_key = keypair.public_key();
    assert!(public_key.verify(context, &signature).is_ok());
}

#[test]
fn test_secure_proof_strict_rejects_empty_context() {
    // ARRANGE: Create a secure keypair and empty context
    let keypair = generate_secure_keypair_with_seed(456);
    let empty_context = b"";
    
    // ACT: Attempt to generate a strict secure proof with empty context
    let result = make_secure_proof_strict(&keypair, empty_context);
    
    // ASSERT: Should fail with EmptyContext error
    assert!(matches!(result, Err(ProofError::EmptyContext)));
}

#[test]
fn test_secure_proof_allows_empty_context() {
    // ARRANGE: Create a secure keypair and empty context
    let keypair = generate_secure_keypair_with_seed(789);
    let empty_context = b"";
    
    // ACT: Generate a secure proof with empty context (non-strict)
    let result = make_secure_proof(&keypair, empty_context);
    
    // ASSERT: Should succeed (backward compatibility)
    assert!(result.is_ok());
    let signature = result.unwrap();
    
    // ASSERT: Signature should be valid
    let public_key = keypair.public_key();
    assert!(public_key.verify(empty_context, &signature).is_ok());
}

#[test]
fn test_secure_proof_rejects_oversized_context() {
    // ARRANGE: Create a secure keypair and oversized context
    let keypair = generate_secure_keypair_with_seed(999);
    let oversized_context = vec![0u8; MAX_CONTEXT_SIZE + 1];
    
    // ACT: Attempt to generate a secure proof with oversized context
    let result = make_secure_proof(&keypair, &oversized_context);
    
    // ASSERT: Should fail with ContextTooLarge error
    assert!(matches!(result, Err(ProofError::ContextTooLarge { .. })));
    
    // ASSERT: Error message should contain size information
    if let Err(ProofError::ContextTooLarge { max, actual }) = result {
        assert_eq!(max, MAX_CONTEXT_SIZE);
        assert_eq!(actual, MAX_CONTEXT_SIZE + 1);
    }
}

#[test]
fn test_secure_proof_verification() {
    // ARRANGE: Create keypair, context, and signature
    let keypair = generate_secure_keypair_with_seed(111);
    let context = b"verification test message";
    let signature = make_secure_proof(&keypair, context).unwrap();
    let public_key = keypair.public_key();
    
    // ACT: Verify the proof securely
    let result = verify_proof_secure(&public_key, context, &signature);
    
    // ASSERT: Verification should succeed
    assert!(result.is_ok());
}

#[test]
fn test_secure_proof_verification_strict() {
    // ARRANGE: Create keypair, non-empty context, and signature
    let keypair = generate_secure_keypair_with_seed(222);
    let context = b"strict verification test";
    let signature = make_secure_proof_strict(&keypair, context).unwrap();
    let public_key = keypair.public_key();
    
    // ACT: Verify the proof with strict validation
    let result = verify_proof_strict(&public_key, context, &signature);
    
    // ASSERT: Verification should succeed
    assert!(result.is_ok());
}

#[test]
fn test_secure_verification_rejects_empty_context() {
    // ARRANGE: Create a valid signature for non-empty context
    let keypair = generate_secure_keypair_with_seed(333);
    let context = b"non-empty";
    let signature = make_secure_proof(&keypair, context).unwrap();
    let public_key = keypair.public_key();
    
    // ACT: Attempt strict verification with empty context
    let empty_context = b"";
    let result = verify_proof_strict(&public_key, empty_context, &signature);
    
    // ASSERT: Should fail with EmptyContext error
    assert!(matches!(result, Err(ProofError::EmptyContext)));
}

#[test]
fn test_secure_verification_rejects_oversized_context() {
    // ARRANGE: Create a valid signature
    let keypair = generate_secure_keypair_with_seed(444);
    let context = b"normal context";
    let signature = make_secure_proof(&keypair, context).unwrap();
    let public_key = keypair.public_key();
    
    // ACT: Attempt verification with oversized context
    let oversized_context = vec![0u8; MAX_CONTEXT_SIZE + 1];
    let result = verify_proof_secure(&public_key, &oversized_context, &signature);
    
    // ASSERT: Should fail with ContextTooLarge error
    assert!(matches!(result, Err(ProofError::ContextTooLarge { .. })));
}

#[test]
fn test_secure_proof_with_tampered_context() {
    // ARRANGE: Create keypair and original context
    let keypair = generate_secure_keypair_with_seed(555);
    let original_context = b"original message";
    let tampered_context = b"tampered message";
    
    // ACT: Create proof with original context
    let signature = make_secure_proof(&keypair, original_context).unwrap();
    let public_key = keypair.public_key();
    
    // ACT: Attempt to verify with tampered context
    let result = verify_proof_secure(&public_key, tampered_context, &signature);
    
    // ASSERT: Should fail with verification error
    assert!(matches!(result, Err(ProofError::VerificationFailed(_))));
}

#[test]
fn test_secure_proof_with_wrong_public_key() {
    // ARRANGE: Create two different keypairs
    let keypair1 = generate_secure_keypair_with_seed(666);
    let keypair2 = generate_secure_keypair_with_seed(777);
    let context = b"test message";
    
    // ACT: Create proof with first keypair
    let signature = make_secure_proof(&keypair1, context).unwrap();
    
    // ACT: Attempt to verify with second keypair's public key
    let wrong_public_key = keypair2.public_key();
    let result = verify_proof_secure(&wrong_public_key, context, &signature);
    
    // ASSERT: Should fail with verification error
    assert!(matches!(result, Err(ProofError::VerificationFailed(_))));
}

#[test]
fn test_invite_with_validation() {
    // ARRANGE: Create valid data
    let valid_data = b"valid invite data".to_vec();
    
    // ACT: Create invite with validation
    let result = Invite::new_with_data(valid_data.clone());
    
    // ASSERT: Should succeed
    assert!(result.is_ok());
    let invite = result.unwrap();
    assert_eq!(invite.get_data(), valid_data.as_slice());
}

#[test]
fn test_invite_validation_rejects_oversized_data() {
    // ARRANGE: Create oversized data
    let oversized_data = vec![0u8; MAX_CONTEXT_SIZE + 1];
    
    // ACT: Attempt to create invite with oversized data
    let result = Invite::new_with_data(oversized_data);
    
    // ASSERT: Should fail with ContextTooLarge error
    assert!(matches!(result, Err(ProofError::ContextTooLarge { .. })));
}

#[test]
fn test_secure_proof_deterministic() {
    // ARRANGE: Create same keypair and context twice
    let keypair1 = generate_secure_keypair_with_seed(888);
    let keypair2 = generate_secure_keypair_with_seed(888);
    let context = b"deterministic test";
    
    // ACT: Generate proofs with both keypairs
    let signature1 = make_secure_proof(&keypair1, context).unwrap();
    let signature2 = make_secure_proof(&keypair2, context).unwrap();
    
    // ASSERT: Signatures should be identical (deterministic)
    assert_eq!(signature1.to_bytes(), signature2.to_bytes());
}

#[test]
fn test_secure_proof_different_seeds_produce_different_signatures() {
    // ARRANGE: Create different keypairs
    let keypair1 = generate_secure_keypair_with_seed(100);
    let keypair2 = generate_secure_keypair_with_seed(200);
    let context = b"uniqueness test";
    
    // ACT: Generate proofs with different keypairs
    let signature1 = make_secure_proof(&keypair1, context).unwrap();
    let signature2 = make_secure_proof(&keypair2, context).unwrap();
    
    // ASSERT: Signatures should be different
    assert_ne!(signature1.to_bytes(), signature2.to_bytes());
}

#[test]
fn test_error_message_formatting() {
    // Test that error messages are informative and well-formatted
    
    // Test ContextTooLarge error
    let oversized_context = vec![0u8; MAX_CONTEXT_SIZE + 100];
    let keypair = generate_secure_keypair_with_seed(999);
    let result = make_secure_proof(&keypair, &oversized_context);
    
    if let Err(error) = result {
        let error_string = format!("{}", error);
        assert!(error_string.contains("exceeds maximum allowed size"));
        assert!(error_string.contains(&format!("{}", MAX_CONTEXT_SIZE)));
        assert!(error_string.contains(&format!("{}", MAX_CONTEXT_SIZE + 100)));
    } else {
        panic!("Expected ContextTooLarge error");
    }
    
    // Test EmptyContext error
    let empty_context = b"";
    let result = make_secure_proof_strict(&keypair, empty_context);
    
    if let Err(error) = result {
        let error_string = format!("{}", error);
        assert!(error_string.contains("cannot be empty"));
    } else {
        panic!("Expected EmptyContext error");
    }
}