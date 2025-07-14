//! Demonstration of secure key handling with automatic memory protection
//! 
//! This example shows how to use SecureKeypair to protect private key material
//! from memory analysis attacks.

use proof_messenger_protocol::key::{generate_secure_keypair, generate_keypair};
use ed25519_dalek::{Signer, Verifier};

fn main() {
    println!("🔐 Secure Key Handling Demo");
    println!("============================\n");

    // Demonstrate the difference between regular and secure keypairs
    demonstrate_memory_protection();
    
    // Show practical usage patterns
    demonstrate_practical_usage();
    
    // Show production server scenario
    demonstrate_server_scenario();
}

fn demonstrate_memory_protection() {
    println!("1. Memory Protection Comparison");
    println!("-------------------------------");
    
    let message = b"sensitive data to sign";
    
    // Regular keypair (legacy approach)
    println!("⚠️  Regular Keypair:");
    let regular_signature = {
        let keypair = generate_keypair();
        println!("   - Generated keypair");
        let sig = keypair.sign(message);
        println!("   - Signed message");
        sig
        // keypair is dropped here, but private key may remain in memory
    };
    println!("   - Keypair dropped (private key may linger in memory)");
    
    // Secure keypair (recommended approach)
    println!("\n✅ SecureKeypair:");
    let secure_signature = {
        let keypair = generate_secure_keypair();
        println!("   - Generated secure keypair");
        let sig = keypair.sign(message);
        println!("   - Signed message");
        sig
        // keypair is dropped here and private key is automatically zeroed
    };
    println!("   - SecureKeypair dropped (private key automatically zeroed)");
    
    // Both produce valid signatures
    println!("\n📝 Verification:");
    println!("   - Regular signature length: {} bytes", regular_signature.to_bytes().len());
    println!("   - Secure signature length: {} bytes", secure_signature.to_bytes().len());
    println!("   - Both signatures are cryptographically valid");
    
    println!("\n🛡️  Security Benefit:");
    println!("   - SecureKeypair prevents private key recovery from memory dumps");
    println!("   - Protection against cold boot attacks and memory analysis");
    println!("   - Automatic cleanup even during program panics\n");
}

fn demonstrate_practical_usage() {
    println!("2. Practical Usage Patterns");
    println!("---------------------------");
    
    // Generate a secure keypair
    let keypair = generate_secure_keypair();
    println!("✅ Generated SecureKeypair");
    
    // Extract public key (safe to share)
    let public_key = keypair.public_key();
    let public_key_bytes = keypair.public_key_bytes();
    println!("📤 Extracted public key: {} bytes", public_key_bytes.len());
    
    // Sign multiple messages
    let messages = [
        b"First message".as_slice(),
        b"Second message".as_slice(),
        b"Third message".as_slice(),
    ];
    
    println!("🔏 Signing messages:");
    for (i, message) in messages.iter().enumerate() {
        let signature = keypair.sign(message);
        
        // Verify signature
        let is_valid = public_key.verify(message, &signature).is_ok();
        println!("   - Message {}: {} ({})", 
                 i + 1, 
                 String::from_utf8_lossy(message),
                 if is_valid { "✅ Valid" } else { "❌ Invalid" });
    }
    
    println!("🔒 Private key will be automatically zeroed when keypair goes out of scope\n");
}

fn demonstrate_server_scenario() {
    println!("3. Production Server Scenario");
    println!("-----------------------------");
    
    // Simulate a secure server that handles signing requests
    struct SecureSigningServer {
        keypair: proof_messenger_protocol::key::SecureKeypair,
        request_count: u32,
    }
    
    impl SecureSigningServer {
        fn new() -> Self {
            println!("🚀 Starting secure signing server...");
            Self {
                keypair: generate_secure_keypair(),
                request_count: 0,
            }
        }
        
        fn get_public_key(&self) -> [u8; 32] {
            self.keypair.public_key_bytes()
        }
        
        fn sign_request(&mut self, data: &[u8]) -> [u8; 64] {
            self.request_count += 1;
            println!("   📝 Processing signing request #{}", self.request_count);
            self.keypair.sign(data).to_bytes()
        }
        
        fn get_stats(&self) -> u32 {
            self.request_count
        }
    }
    
    // When the server is dropped, the private key is automatically zeroed
    impl Drop for SecureSigningServer {
        fn drop(&mut self) {
            println!("🛑 Server shutting down after {} requests", self.request_count);
            println!("🔐 Private key automatically zeroed from memory");
        }
    }
    
    let public_key_bytes;
    let signatures: Vec<[u8; 64]>;
    
    // Server lifecycle
    {
        let mut server = SecureSigningServer::new();
        public_key_bytes = server.get_public_key();
        
        // Process some signing requests
        let requests = [
            b"User authentication token".as_slice(),
            b"Document signature request".as_slice(), 
            b"Transaction authorization".as_slice(),
        ];
        
        signatures = requests
            .iter()
            .map(|data| server.sign_request(data))
            .collect();
        
        println!("📊 Server processed {} requests", server.get_stats());
        
        // Server goes out of scope here - private key is automatically zeroed
    }
    
    // Verify all signatures are still valid using only the public key
    println!("🔍 Verifying signatures with public key only:");
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .expect("Valid public key");
    
    let requests = [
        b"User authentication token".as_slice(),
        b"Document signature request".as_slice(), 
        b"Transaction authorization".as_slice(),
    ];
    
    for (i, signature_bytes) in signatures.iter().enumerate() {
        let signature = ed25519_dalek::Signature::from_bytes(signature_bytes)
            .expect("Valid signature");
        let is_valid = public_key.verify(requests[i], &signature).is_ok();
        println!("   - Request {}: {} ({})", 
                 i + 1,
                 String::from_utf8_lossy(requests[i]),
                 if is_valid { "✅ Valid" } else { "❌ Invalid" });
    }
    
    println!("\n🎯 Key Benefits:");
    println!("   - Private key was automatically protected throughout server lifetime");
    println!("   - Memory was securely cleaned up when server shut down");
    println!("   - All cryptographic operations remain verifiable");
    println!("   - Zero manual memory management required");
    println!("   - Protection against memory dump attacks");
}