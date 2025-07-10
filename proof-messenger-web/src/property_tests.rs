/**
 * Property-Based Tests for Protocol Invariants
 * 
 * This module contains comprehensive property-based tests using proptest
 * to verify all cryptographic and protocol invariants hold under random inputs.
 */

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
    use rand::SeedableRng;
    use crate::{WasmKeyPair, WasmMessage, validate_invite_code, validate_public_key, validate_signature, verify_signature};

    // Strategy generators for test data
    prop_compose! {
        fn arb_keypair()(seed in any::<u64>()) -> Keypair {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            Keypair::generate(&mut rng)
        }
    }

    prop_compose! {
        fn arb_message_content()(content in ".*") -> String {
            content
        }
    }

    prop_compose! {
        fn arb_bytes(size: usize)(bytes in prop::collection::vec(any::<u8>(), size)) -> Vec<u8> {
            bytes
        }
    }

    proptest! {
        // INVARIANT 1: Keypair Generation Consistency
        #[test]
        fn keypair_generation_is_deterministic_with_same_seed(seed in any::<u64>()) {
            let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed);
            let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed);
            
            let kp1 = Keypair::generate(&mut rng1);
            let kp2 = Keypair::generate(&mut rng2);
            
            prop_assert_eq!(kp1.public.to_bytes(), kp2.public.to_bytes());
            prop_assert_eq!(kp1.secret.to_bytes(), kp2.secret.to_bytes());
        }

        // INVARIANT 2: Public Key Derivation Consistency
        #[test]
        fn public_key_always_derives_from_private_key(seed in any::<u64>()) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let derived_public = PublicKey::from(&kp.secret);
            prop_assert_eq!(kp.public.to_bytes(), derived_public.to_bytes());
        }

        // INVARIANT 3: Signature Verification Consistency
        #[test]
        fn signature_always_verifies_with_correct_key_and_data(
            seed in any::<u64>(),
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let signature = kp.sign(&data);
            prop_assert!(kp.public.verify(&data, &signature).is_ok());
        }

        // INVARIANT 4: Signature Fails with Wrong Key
        #[test]
        fn signature_fails_with_wrong_key(
            seed1 in any::<u64>(),
            seed2 in any::<u64>(),
            data in prop::collection::vec(any::<u8>(), 1..1000)
        ) {
            prop_assume!(seed1 != seed2); // Ensure different keys
            
            let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed1);
            let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed2);
            
            let kp1 = Keypair::generate(&mut rng1);
            let kp2 = Keypair::generate(&mut rng2);
            
            let signature = kp1.sign(&data);
            prop_assert!(kp2.public.verify(&data, &signature).is_err());
        }

        // INVARIANT 5: Signature Fails with Tampered Data
        #[test]
        fn signature_fails_with_tampered_data(
            seed in any::<u64>(),
            mut data in prop::collection::vec(any::<u8>(), 1..1000),
            tamper_byte in any::<u8>()
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let signature = kp.sign(&data);
            
            // Tamper with the data
            let original_byte = data[0];
            data[0] = tamper_byte;
            
            // Only assert failure if data actually changed
            if original_byte != tamper_byte {
                prop_assert!(kp.public.verify(&data, &signature).is_err());
            }
        }

        // INVARIANT 6: Message ID Uniqueness
        #[test]
        fn message_ids_are_unique(
            seed1 in any::<u64>(),
            seed2 in any::<u64>(),
            content1 in ".*",
            content2 in ".*"
        ) {
            let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed1);
            let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed2);
            
            let kp1 = Keypair::generate(&mut rng1);
            let kp2 = Keypair::generate(&mut rng2);
            
            let msg1 = WasmMessage {
                sender: kp1.public.to_bytes().to_vec(),
                recipient: kp2.public.to_bytes().to_vec(),
                content: content1,
                proof: None,
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            
            let msg2 = WasmMessage {
                sender: kp1.public.to_bytes().to_vec(),
                recipient: kp2.public.to_bytes().to_vec(),
                content: content2,
                proof: None,
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            
            prop_assert_ne!(msg1.id, msg2.id);
        }

        // INVARIANT 7: Message Signing Determinism
        #[test]
        fn message_signing_is_deterministic(
            seed in any::<u64>(),
            content in ".*"
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let sender = kp.public.to_bytes().to_vec();
            let recipient = kp.public.to_bytes().to_vec();
            
            let mut msg1 = WasmMessage {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content: content.clone(),
                proof: None,
                id: "test-id".to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            
            let mut msg2 = WasmMessage {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content: content.clone(),
                proof: None,
                id: "test-id".to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            
            let keypair_bytes = {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&kp.secret.to_bytes());
                bytes.extend_from_slice(&kp.public.to_bytes());
                bytes
            };
            
            msg1.sign(&keypair_bytes).unwrap();
            msg2.sign(&keypair_bytes).unwrap();
            
            prop_assert_eq!(msg1.proof, msg2.proof);
        }

        // INVARIANT 8: Invite Code Format Consistency
        #[test]
        fn invite_codes_always_have_correct_format(seed in any::<u64>()) {
            use rand::RngCore;
            use rand::SeedableRng;
            
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut buf = [0u8; 10];
            rng.fill_bytes(&mut buf);
            
            let code = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &buf);
            let code16 = &code[..16];
            
            prop_assert_eq!(code16.len(), 16);
            prop_assert!(code16.chars().all(|c| c.is_ascii_alphanumeric()));
            prop_assert!(validate_invite_code(code16));
        }

        // INVARIANT 9: Hex Conversion Roundtrip
        #[test]
        fn hex_conversion_is_reversible(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
            let hex = hex::encode(&bytes);
            let decoded = hex::decode(&hex).unwrap();
            prop_assert_eq!(bytes, decoded);
        }

        // INVARIANT 10: Key Length Invariants
        #[test]
        fn key_lengths_are_always_correct(seed in any::<u64>()) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            prop_assert_eq!(kp.public.to_bytes().len(), PUBLIC_KEY_LENGTH);
            prop_assert_eq!(kp.secret.to_bytes().len(), SECRET_KEY_LENGTH);
        }

        // INVARIANT 11: WASM Keypair Consistency
        #[test]
        fn wasm_keypair_maintains_consistency(seed in any::<u64>()) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let native_kp = Keypair::generate(&mut rng);
            
            // Create WASM keypair from same bytes
            let mut keypair_bytes = Vec::new();
            keypair_bytes.extend_from_slice(&native_kp.secret.to_bytes());
            keypair_bytes.extend_from_slice(&native_kp.public.to_bytes());
            
            let wasm_kp = WasmKeyPair::from_bytes(&keypair_bytes).unwrap();
            
            prop_assert_eq!(wasm_kp.public_key_bytes(), native_kp.public.to_bytes().to_vec());
            prop_assert_eq!(wasm_kp.private_key_bytes(), native_kp.secret.to_bytes().to_vec());
        }

        // INVARIANT 12: Message Verification Consistency
        #[test]
        fn message_verification_matches_native_crypto(
            seed in any::<u64>(),
            content in ".*"
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let sender = kp.public.to_bytes().to_vec();
            let recipient = kp.public.to_bytes().to_vec();
            
            let mut msg = WasmMessage {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content: content.clone(),
                proof: None,
                id: "test-id".to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            
            let keypair_bytes = {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&kp.secret.to_bytes());
                bytes.extend_from_slice(&kp.public.to_bytes());
                bytes
            };
            
            msg.sign(&keypair_bytes).unwrap();
            
            // Verify using WASM method
            let wasm_verified = msg.verify(&kp.public.to_bytes()).unwrap();
            
            // Verify using native crypto
            let mut to_sign = sender;
            to_sign.extend(&recipient);
            to_sign.extend(content.as_bytes());
            
            let signature = Signature::from_bytes(msg.proof.as_ref().unwrap()).unwrap();
            let native_verified = kp.public.verify(&to_sign, &signature).is_ok();
            
            prop_assert_eq!(wasm_verified, native_verified);
        }

        // INVARIANT 13: Error Handling Consistency
        #[test]
        fn invalid_keys_always_fail_consistently(
            invalid_len in 1usize..100,
            data in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            prop_assume!(invalid_len != PUBLIC_KEY_LENGTH && invalid_len != SECRET_KEY_LENGTH);
            
            let invalid_key = vec![0u8; invalid_len];
            
            // All these should fail consistently
            prop_assert!(PublicKey::from_bytes(&invalid_key).is_err());
            prop_assert!(SecretKey::from_bytes(&invalid_key).is_err());
            prop_assert!(!validate_public_key(&invalid_key));
        }

        // INVARIANT 14: Signature Format Consistency
        #[test]
        fn signatures_always_have_correct_format(
            seed in any::<u64>(),
            data in prop::collection::vec(any::<u8>(), 1..1000)
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let signature = kp.sign(&data);
            let sig_bytes = signature.to_bytes();
            
            prop_assert_eq!(sig_bytes.len(), 64);
            prop_assert!(validate_signature(&sig_bytes));
            prop_assert!(Signature::from_bytes(&sig_bytes).is_ok());
        }

        // INVARIANT 15: Cross-Platform Consistency
        #[test]
        fn wasm_and_native_produce_same_results(
            seed in any::<u64>(),
            data in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let native_kp = Keypair::generate(&mut rng);
            
            // Create WASM keypair from same seed
            let mut keypair_bytes = Vec::new();
            keypair_bytes.extend_from_slice(&native_kp.secret.to_bytes());
            keypair_bytes.extend_from_slice(&native_kp.public.to_bytes());
            
            let wasm_kp = WasmKeyPair::from_bytes(&keypair_bytes).unwrap();
            
            // Sign with both
            let native_sig = native_kp.sign(&data);
            let wasm_sig = wasm_kp.sign(&data).unwrap();
            
            // Both should produce same signature
            prop_assert_eq!(native_sig.to_bytes().to_vec(), wasm_sig.clone());
            
            // Both should verify correctly
            prop_assert!(native_kp.public.verify(&data, &native_sig).is_ok());
            prop_assert!(verify_signature(&native_kp.public.to_bytes(), &data, &wasm_sig).unwrap());
        }
    }

    // Additional unit tests for edge cases
    #[test]
    fn test_empty_data_signing() {
        let kp = WasmKeyPair::new();
        let empty_data = Vec::new();
        
        let signature = kp.sign(&empty_data).unwrap();
        assert_eq!(signature.len(), 64);
        
        let verified = verify_signature(&kp.public_key_bytes(), &empty_data, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_maximum_data_size() {
        let kp = WasmKeyPair::new();
        let large_data = vec![0u8; 1_000_000]; // 1MB
        
        let signature = kp.sign(&large_data).unwrap();
        let verified = verify_signature(&kp.public_key_bytes(), &large_data, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_unicode_content_handling() {
        let kp = WasmKeyPair::new();
        let unicode_content = "Hello 世界 🌍 Здравствуй мир";
        
        let mut msg = WasmMessage {
            sender: kp.public_key_bytes(),
            recipient: kp.public_key_bytes(),
            content: unicode_content.to_string(),
            proof: None,
            id: "test-unicode".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        };
        
        msg.sign(&kp.keypair_bytes()).unwrap();
        let verified = msg.verify(&kp.public_key_bytes()).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;
        
        let kp = Arc::new(WasmKeyPair::new());
        let mut handles = vec![];
        
        for i in 0..10 {
            let kp_clone = Arc::clone(&kp);
            let handle = thread::spawn(move || {
                let data = format!("test data {}", i).into_bytes();
                let signature = kp_clone.sign(&data).unwrap();
                verify_signature(&kp_clone.public_key_bytes(), &data, &signature).unwrap()
            });
            handles.push(handle);
        }
        
        for handle in handles {
            assert!(handle.join().unwrap());
        }
    }
}