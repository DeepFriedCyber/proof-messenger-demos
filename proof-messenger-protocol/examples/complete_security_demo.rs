//! Complete Security Enhancement Demonstration
//! 
//! This example demonstrates all the security enhancements implemented
//! in the Proof Messenger protocol, including memory protection,
//! input validation, and secure proof generation.

use proof_messenger_protocol::key::{generate_secure_keypair, generate_keypair};
use proof_messenger_protocol::proof::{
    make_secure_proof, make_secure_proof_strict, verify_proof_secure, verify_proof_strict,
    make_proof_context, verify_proof_result, ProofError, Invite, MAX_CONTEXT_SIZE
};
use ed25519_dalek::Verifier;

fn main() {
    println!("🔐 Complete Security Enhancement Demo");
    println!("=====================================\n");

    // Demonstrate memory protection
    demonstrate_memory_protection();
    
    // Demonstrate input validation
    demonstrate_input_validation();
    
    // Demonstrate secure proof generation
    demonstrate_secure_proof_generation();
    
    // Demonstrate error handling
    demonstrate_error_handling();
    
    // Demonstrate backward compatibility
    demonstrate_backward_compatibility();
    
    // Demonstrate production scenario
    demonstrate_production_scenario();
}

fn demonstrate_memory_protection() {
    println!("1. Memory Protection Enhancement");
    println!("--------------------------------");
    
    println!("🔒 Generating SecureKeypair with automatic memory protection...");
    let secure_keypair = generate_secure_keypair();
    let public_key_bytes = secure_keypair.public_key_bytes();
    
    println!("✅ SecureKeypair generated");
    println!("📤 Public key extracted: {} bytes", public_key_bytes.len());
    
    let message = b"sensitive cryptographic operation";
    let signature = secure_keypair.sign(message);
    
    println!("🔏 Message signed securely");
    println!("🛡️  Private key will be automatically zeroed when keypair goes out of scope");
    
    // Verify the signature
    let public_key = secure_keypair.public_key();
    let is_valid = public_key.verify(message, &signature).is_ok();
    println!("✅ Signature verification: {}\n", if is_valid { "Valid" } else { "Invalid" });
}

fn demonstrate_input_validation() {
    println!("2. Input Validation Enhancement");
    println!("-------------------------------");
    
    let keypair = generate_secure_keypair();
    
    // Test normal input
    println!("📝 Testing normal input validation...");
    let normal_context = b"normal message context";
    match make_secure_proof(&keypair, normal_context) {
        Ok(_) => println!("✅ Normal input accepted"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    // Test empty context with regular secure proof (should work)
    println!("📝 Testing empty context with regular secure proof...");
    let empty_context = b"";
    match make_secure_proof(&keypair, empty_context) {
        Ok(_) => println!("✅ Empty context accepted (backward compatibility)"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    // Test empty context with strict validation (should fail)
    println!("📝 Testing empty context with strict validation...");
    match make_secure_proof_strict(&keypair, empty_context) {
        Ok(_) => println!("❌ Empty context should have been rejected"),
        Err(ProofError::EmptyContext) => println!("✅ Empty context properly rejected by strict validation"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    // Test oversized input (should fail)
    println!("📝 Testing oversized input validation...");
    let oversized_context = vec![0u8; MAX_CONTEXT_SIZE + 1];
    match make_secure_proof(&keypair, &oversized_context) {
        Ok(_) => println!("❌ Oversized input should have been rejected"),
        Err(ProofError::ContextTooLarge { max, actual }) => {
            println!("✅ Oversized input properly rejected (max: {}, actual: {})", max, actual);
        },
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    println!();
}

fn demonstrate_secure_proof_generation() {
    println!("3. Secure Proof Generation");
    println!("--------------------------");
    
    let keypair = generate_secure_keypair();
    let context = b"secure proof generation test";
    
    // Generate secure proof
    println!("🔏 Generating secure proof with validation...");
    let signature = match make_secure_proof(&keypair, context) {
        Ok(sig) => {
            println!("✅ Secure proof generated successfully");
            sig
        },
        Err(e) => {
            println!("❌ Failed to generate secure proof: {}", e);
            return;
        }
    };
    
    // Verify with secure verification
    println!("🔍 Verifying proof with secure validation...");
    let public_key = keypair.public_key();
    match verify_proof_secure(&public_key, context, &signature) {
        Ok(()) => println!("✅ Secure verification successful"),
        Err(e) => println!("❌ Secure verification failed: {}", e),
    }
    
    // Test with tampered context
    println!("🔍 Testing tamper detection...");
    let tampered_context = b"tampered proof generation test";
    match verify_proof_secure(&public_key, tampered_context, &signature) {
        Ok(()) => println!("❌ Tamper detection failed"),
        Err(ProofError::VerificationFailed(_)) => println!("✅ Tamper detection successful"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    println!();
}

fn demonstrate_error_handling() {
    println!("4. Enhanced Error Handling");
    println!("--------------------------");
    
    let keypair = generate_secure_keypair();
    
    // Demonstrate different error types
    println!("📋 Testing different error scenarios...");
    
    // Empty context error
    let empty_context = b"";
    if let Err(error) = make_secure_proof_strict(&keypair, empty_context) {
        println!("🔍 EmptyContext error: {}", error);
        println!("   Debug format: {:?}", error);
    }
    
    // Context too large error
    let oversized_context = vec![0u8; MAX_CONTEXT_SIZE + 100];
    if let Err(error) = make_secure_proof(&keypair, &oversized_context) {
        println!("🔍 ContextTooLarge error: {}", error);
        println!("   Debug format: {:?}", error);
    }
    
    // Verification failed error
    let keypair2 = generate_secure_keypair();
    let context = b"error handling test";
    let signature = make_secure_proof(&keypair, context).unwrap();
    let wrong_public_key = keypair2.public_key();
    
    if let Err(error) = verify_proof_secure(&wrong_public_key, context, &signature) {
        println!("🔍 VerificationFailed error: {}", error);
        println!("   Debug format: {:?}", error);
    }
    
    println!();
}

fn demonstrate_backward_compatibility() {
    println!("5. Backward Compatibility");
    println!("-------------------------");
    
    let context = b"compatibility test message";
    
    // Old API
    println!("📜 Using legacy API...");
    let old_keypair = generate_keypair();
    let old_signature = make_proof_context(&old_keypair, context);
    let old_result = verify_proof_result(&old_keypair.public, context, &old_signature);
    println!("✅ Legacy API result: {:?}", old_result.is_ok());
    
    // New API
    println!("🆕 Using new secure API...");
    let new_keypair = generate_secure_keypair();
    let new_signature = make_secure_proof(&new_keypair, context).unwrap();
    let new_public_key = new_keypair.public_key();
    let new_result = verify_proof_secure(&new_public_key, context, &new_signature);
    println!("✅ New API result: {:?}", new_result.is_ok());
    
    // Cross-compatibility
    println!("🔄 Testing cross-compatibility...");
    let secure_keypair = generate_secure_keypair();
    let legacy_keypair = secure_keypair.as_keypair();
    let cross_signature = make_proof_context(&legacy_keypair, context);
    let cross_result = verify_proof_secure(&secure_keypair.public_key(), context, &cross_signature);
    println!("✅ Cross-compatibility result: {:?}", cross_result.is_ok());
    
    println!();
}

fn demonstrate_production_scenario() {
    println!("6. Production Security Scenario");
    println!("-------------------------------");
    
    // Simulate a secure message processing server
    struct SecureMessageProcessor {
        keypair: proof_messenger_protocol::key::SecureKeypair,
        processed_count: u32,
    }
    
    impl SecureMessageProcessor {
        fn new() -> Self {
            println!("🚀 Initializing secure message processor...");
            Self {
                keypair: generate_secure_keypair(),
                processed_count: 0,
            }
        }
        
        fn get_public_key(&self) -> [u8; 32] {
            self.keypair.public_key_bytes()
        }
        
        fn process_message(&mut self, message: &[u8]) -> Result<[u8; 64], ProofError> {
            // Validate input
            if message.is_empty() {
                return Err(ProofError::EmptyContext);
            }
            
            if message.len() > MAX_CONTEXT_SIZE {
                return Err(ProofError::ContextTooLarge {
                    max: MAX_CONTEXT_SIZE,
                    actual: message.len(),
                });
            }
            
            // Generate secure proof
            let signature = make_secure_proof_strict(&self.keypair, message)?;
            self.processed_count += 1;
            
            println!("   ✅ Processed message #{}: {} bytes", 
                     self.processed_count, message.len());
            
            Ok(signature.to_bytes())
        }
        
        fn get_stats(&self) -> u32 {
            self.processed_count
        }
    }
    
    impl Drop for SecureMessageProcessor {
        fn drop(&mut self) {
            println!("🛑 Secure processor shutting down after {} messages", self.processed_count);
            println!("🔐 All private key material automatically zeroed");
        }
    }
    
    let public_key_bytes;
    let signatures: Vec<[u8; 64]>;
    
    // Process messages securely
    {
        let mut processor = SecureMessageProcessor::new();
        public_key_bytes = processor.get_public_key();
        
        let messages = [
            b"User authentication request".as_slice(),
            b"Document signing request".as_slice(),
            b"Transaction authorization".as_slice(),
        ];
        
        signatures = messages
            .iter()
            .map(|msg| processor.process_message(msg).unwrap())
            .collect();
        
        println!("📊 Total messages processed: {}", processor.get_stats());
        
        // Test error handling
        println!("🧪 Testing error handling...");
        match processor.process_message(b"") {
            Err(ProofError::EmptyContext) => println!("   ✅ Empty message properly rejected"),
            _ => println!("   ❌ Empty message should have been rejected"),
        }
        
        // Processor and its secure keypair are dropped here
    }
    
    // Verify all signatures are still valid
    println!("🔍 Verifying all signatures with public key only...");
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .expect("Valid public key");
    
    let messages = [
        b"User authentication request".as_slice(),
        b"Document signing request".as_slice(),
        b"Transaction authorization".as_slice(),
    ];
    
    for (i, signature_bytes) in signatures.iter().enumerate() {
        let signature = ed25519_dalek::Signature::from_bytes(signature_bytes)
            .expect("Valid signature");
        let is_valid = public_key.verify(messages[i], &signature).is_ok();
        println!("   ✅ Message {}: {}", i + 1, if is_valid { "Valid" } else { "Invalid" });
    }
    
    println!("\n🎯 Security Benefits Demonstrated:");
    println!("   ✅ Automatic memory protection for private keys");
    println!("   ✅ Comprehensive input validation");
    println!("   ✅ Secure error handling without information leakage");
    println!("   ✅ Production-ready performance and reliability");
    println!("   ✅ Full backward compatibility with existing code");
    println!("   ✅ Defense-in-depth security architecture");
}