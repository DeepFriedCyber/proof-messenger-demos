//! Property-based tests for proof error handling
//!
//! This module contains comprehensive property-based tests using proptest
//! to verify that the new Result-based error handling maintains all invariants
//! under random inputs while providing detailed error information.

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::key::generate_keypair_with_seed;

    proptest! {
        /// Property: Valid proofs always verify successfully
        #[test]
        fn prop_valid_proof_always_succeeds(
            seed in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let keypair = generate_keypair_with_seed(seed);
            let signature = crate::proof::make_proof_context(&keypair, &context);
            
            let result = crate::proof::verify_proof_result(&keypair.public, &context, &signature);
            prop_assert!(result.is_ok());
        }

        /// Property: Proofs with wrong keys always fail with VerificationFailed
        #[test]
        fn prop_wrong_key_always_fails_with_verification_error(
            seed1 in any::<u64>(),
            seed2 in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            prop_assume!(seed1 != seed2); // Ensure different keypairs
            
            let keypair1 = generate_keypair_with_seed(seed1);
            let keypair2 = generate_keypair_with_seed(seed2);
            
            let signature = crate::proof::make_proof_context(&keypair1, &context);
            let result = crate::proof::verify_proof_result(&keypair2.public, &context, &signature);
            
            prop_assert!(matches!(result, Err(crate::proof::ProofError::VerificationFailed(_))));
        }

        /// Property: Tampered context always fails with VerificationFailed
        #[test]
        fn prop_tampered_context_always_fails(
            seed in any::<u64>(),
            original_context in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_index in any::<usize>(),
            tamper_value in any::<u8>()
        ) {
            let keypair = generate_keypair_with_seed(seed);
            let signature = crate::proof::make_proof_context(&keypair, &original_context);
            
            // Create tampered context
            let mut tampered_context = original_context.clone();
            if !tampered_context.is_empty() {
                let index = tamper_index % tampered_context.len();
                // Ensure we actually change the value
                tampered_context[index] = tamper_value;
                
                // Only test if we actually tampered something
                if tampered_context != original_context {
                    let result = crate::proof::verify_proof_result(&keypair.public, &tampered_context, &signature);
                    prop_assert!(matches!(result, Err(crate::proof::ProofError::VerificationFailed(_))));
                }
            }
        }

        /// Property: Error messages are always non-empty and informative
        #[test]
        fn prop_error_messages_are_informative(
            seed1 in any::<u64>(),
            seed2 in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            prop_assume!(seed1 != seed2);
            
            let keypair1 = generate_keypair_with_seed(seed1);
            let keypair2 = generate_keypair_with_seed(seed2);
            
            let signature = crate::proof::make_proof_context(&keypair1, &context);
            let result = crate::proof::verify_proof_result(&keypair2.public, &context, &signature);
            
            match result {
                Err(err) => {
                    let error_string = format!("{}", err);
                    let debug_string = format!("{:?}", err);
                    
                    // Error messages should be non-empty
                    prop_assert!(!error_string.is_empty());
                    prop_assert!(!debug_string.is_empty());
                    
                    // Error messages should contain key information
                    prop_assert!(error_string.contains("verification failed") || 
                               error_string.contains("invalid signature"));
                    prop_assert!(debug_string.contains("VerificationFailed"));
                }
                Ok(_) => prop_assert!(false, "Expected verification to fail with wrong key"),
            }
        }

        /// Property: Result-based API is consistent with boolean API
        #[test]
        fn prop_result_api_consistent_with_boolean_api(
            seed in any::<u64>(),
            invite_seed in any::<u64>()
        ) {
            let keypair = generate_keypair_with_seed(seed);
            let invite = crate::proof::Invite::new_with_seed(invite_seed);
            
            // Test with old API
            let signature = crate::proof::make_proof(&keypair, &invite);
            let bool_result = crate::proof::verify_proof(&signature, &keypair.public, &invite);
            
            // Test with new API using same data
            let result_api = crate::proof::verify_proof_result(&keypair.public, &invite.data, &signature);
            
            // Results should be consistent
            prop_assert_eq!(bool_result, result_api.is_ok());
        }

        /// Property: Signatures are deterministic for same input
        #[test]
        fn prop_signatures_are_deterministic(
            seed in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let keypair = generate_keypair_with_seed(seed);
            
            let sig1 = crate::proof::make_proof_context(&keypair, &context);
            let sig2 = crate::proof::make_proof_context(&keypair, &context);
            
            // Signatures should be identical for same keypair and context
            prop_assert_eq!(sig1.to_bytes(), sig2.to_bytes());
            
            // Both should verify successfully
            let result1 = crate::proof::verify_proof_result(&keypair.public, &context, &sig1);
            let result2 = crate::proof::verify_proof_result(&keypair.public, &context, &sig2);
            
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
        }

        /// Property: Empty context handling
        #[test]
        fn prop_empty_context_handling(seed in any::<u64>()) {
            let keypair = generate_keypair_with_seed(seed);
            let empty_context = &[];
            
            let signature = crate::proof::make_proof_context(&keypair, empty_context);
            let result = crate::proof::verify_proof_result(&keypair.public, empty_context, &signature);
            
            // Empty context should still work
            prop_assert!(result.is_ok());
        }

        /// Property: Large context handling
        #[test]
        fn prop_large_context_handling(
            seed in any::<u64>(),
            large_context in prop::collection::vec(any::<u8>(), 10000..50000)
        ) {
            let keypair = generate_keypair_with_seed(seed);
            
            let signature = crate::proof::make_proof_context(&keypair, &large_context);
            let result = crate::proof::verify_proof_result(&keypair.public, &large_context, &signature);
            
            // Large contexts should still work
            prop_assert!(result.is_ok());
        }

        /// Property: Cross-verification between different keypairs always fails
        #[test]
        fn prop_cross_verification_always_fails(
            seed1 in any::<u64>(),
            seed2 in any::<u64>(),
            seed3 in any::<u64>(),
            context in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            prop_assume!(seed1 != seed2 && seed2 != seed3 && seed1 != seed3);
            
            let keypair1 = generate_keypair_with_seed(seed1);
            let keypair2 = generate_keypair_with_seed(seed2);
            let keypair3 = generate_keypair_with_seed(seed3);
            
            let signature = crate::proof::make_proof_context(&keypair1, &context);
            
            // Verification with different keys should always fail
            let result2 = crate::proof::verify_proof_result(&keypair2.public, &context, &signature);
            let result3 = crate::proof::verify_proof_result(&keypair3.public, &context, &signature);
            
            prop_assert!(matches!(result2, Err(crate::proof::ProofError::VerificationFailed(_))));
            prop_assert!(matches!(result3, Err(crate::proof::ProofError::VerificationFailed(_))));
        }
    }
}