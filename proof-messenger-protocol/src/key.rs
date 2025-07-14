use ed25519_dalek::{Keypair, PublicKey, Signature, Signer};
use rand::rngs::OsRng;
use rand::SeedableRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure wrapper around Ed25519 keypair that automatically zeros
/// sensitive key material when dropped from memory.
/// 
/// This prevents private key material from lingering in memory where
/// it could potentially be recovered by an attacker.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureKeypair {
    /// Raw bytes of the keypair (secret key + public key)
    /// This will be automatically zeroed when the struct is dropped
    keypair_bytes: [u8; 64], // 32 bytes secret + 32 bytes public
}

impl SecureKeypair {
    /// Generate a new secure keypair using cryptographically secure randomness
    pub fn generate() -> Self {
        let keypair = Keypair::generate(&mut OsRng);
        Self {
            keypair_bytes: keypair.to_bytes(),
        }
    }

    /// Generate a secure keypair from a deterministic seed (for testing)
    pub fn generate_with_seed(seed: u64) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let keypair = Keypair::generate(&mut rng);
        Self {
            keypair_bytes: keypair.to_bytes(),
        }
    }

    /// Create a secure keypair from raw bytes
    /// 
    /// # Arguments
    /// * `bytes` - 64 bytes containing secret key (first 32) + public key (last 32)
    /// 
    /// # Returns
    /// * `Ok(SecureKeypair)` if bytes are valid
    /// * `Err(())` if bytes are invalid length or format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 64 {
            return Err("Keypair bytes must be exactly 64 bytes");
        }

        // Validate that we can construct a valid keypair from these bytes
        let _keypair = Keypair::from_bytes(bytes)
            .map_err(|_| "Invalid keypair bytes")?;

        let mut keypair_bytes = [0u8; 64];
        keypair_bytes.copy_from_slice(bytes);
        
        Ok(Self { keypair_bytes })
    }

    /// Get the public key (safe to expose)
    pub fn public_key(&self) -> PublicKey {
        // Extract public key from the last 32 bytes
        let mut public_bytes = [0u8; 32];
        public_bytes.copy_from_slice(&self.keypair_bytes[32..]);
        PublicKey::from_bytes(&public_bytes).expect("Valid public key")
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        let mut public_bytes = [0u8; 32];
        public_bytes.copy_from_slice(&self.keypair_bytes[32..]);
        public_bytes
    }

    /// Sign a message with this keypair
    /// 
    /// This method provides access to signing functionality without
    /// exposing the raw keypair or secret key material.
    pub fn sign(&self, message: &[u8]) -> Signature {
        // Temporarily reconstruct the keypair for signing
        let keypair = Keypair::from_bytes(&self.keypair_bytes)
            .expect("Valid keypair bytes");
        keypair.sign(message)
    }

    /// Get a temporary reference to the underlying keypair
    /// 
    /// ⚠️  WARNING: Use this method sparingly and ensure the returned
    /// keypair doesn't outlive this SecureKeypair instance.
    /// 
    /// This method is provided for compatibility with existing APIs
    /// that expect a Keypair reference.
    pub fn as_keypair(&self) -> Keypair {
        Keypair::from_bytes(&self.keypair_bytes)
            .expect("Valid keypair bytes")
    }

    /// Convert to bytes (for serialization)
    /// 
    /// ⚠️  WARNING: The returned bytes contain sensitive key material.
    /// Ensure they are properly handled and zeroed after use.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.keypair_bytes
    }
}

// Implement Clone manually to ensure we don't accidentally expose key material
impl Clone for SecureKeypair {
    fn clone(&self) -> Self {
        Self {
            keypair_bytes: self.keypair_bytes,
        }
    }
}

// Legacy functions for backward compatibility
// These now return regular Keypair instances for compatibility,
// but users should migrate to SecureKeypair for better security

/// Generate a keypair using cryptographically secure randomness
/// 
/// ⚠️  DEPRECATED: Use `SecureKeypair::generate()` for better security.
/// This function is kept for backward compatibility.
pub fn generate_keypair() -> Keypair {
    Keypair::generate(&mut OsRng)
}

/// Generate a keypair from a deterministic seed (for testing)
/// 
/// ⚠️  DEPRECATED: Use `SecureKeypair::generate_with_seed()` for better security.
/// This function is kept for backward compatibility.
pub fn generate_keypair_with_seed(seed: u64) -> Keypair {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    Keypair::generate(&mut rng)
}

/// Generate a secure keypair using cryptographically secure randomness
/// 
/// This is the recommended way to generate keypairs as it provides
/// automatic memory protection for sensitive key material.
pub fn generate_secure_keypair() -> SecureKeypair {
    SecureKeypair::generate()
}

/// Generate a secure keypair from a deterministic seed (for testing)
/// 
/// This is the recommended way to generate test keypairs as it provides
/// automatic memory protection for sensitive key material.
pub fn generate_secure_keypair_with_seed(seed: u64) -> SecureKeypair {
    SecureKeypair::generate_with_seed(seed)
}