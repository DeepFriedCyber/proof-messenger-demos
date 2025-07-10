//! WASM-specific bindings and utilities
//!
//! This module provides WebAssembly bindings for the proof messenger protocol,
//! allowing the library to be used in web browsers and other JavaScript environments.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::prelude::*;

#[cfg(feature = "wasm")]
use js_sys::{Array, Object, Reflect};

#[cfg(feature = "wasm")]
use web_sys::console;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(feature = "wasm")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// WASM wrapper for KeyPair
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmKeyPair {
    inner: KeyPair,
}

/// WASM wrapper for PublicKey
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmPublicKey {
    inner: PublicKey,
}

/// WASM wrapper for Message
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmMessage {
    inner: Message,
}

/// WASM wrapper for Proof
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmProof {
    inner: Proof,
}

/// WASM wrapper for ProofVerifier
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmProofVerifier;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmKeyPair {
    /// Generate a new keypair
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmKeyPair, JsValue> {
        console_log!("Generating new keypair in WASM");
        let keypair = KeyPair::generate()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmKeyPair { inner: keypair })
    }

    /// Get the public key as bytes
    #[wasm_bindgen(getter)]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.inner.public_key().to_bytes().to_vec()
    }

    /// Get the public key as a hex string
    #[wasm_bindgen(getter)]
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.inner.public_key().to_bytes())
    }

    /// Get the private key as bytes (use with caution!)
    #[wasm_bindgen(getter)]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes().to_vec()
    }

    /// Sign a message
    #[wasm_bindgen]
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, JsValue> {
        let signature = self.inner.sign(message)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(signature.to_bytes().to_vec())
    }

    /// Create a keypair from private key bytes
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<WasmKeyPair, JsValue> {
        if bytes.len() != 32 {
            return Err(JsValue::from_str("Private key must be 32 bytes"));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        
        let keypair = KeyPair::from_bytes(&key_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmKeyPair { inner: keypair })
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmPublicKey {
    /// Create a public key from bytes
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<WasmPublicKey, JsValue> {
        if bytes.len() != 32 {
            return Err(JsValue::from_str("Public key must be 32 bytes"));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        
        let public_key = PublicKey::from_bytes(&key_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmPublicKey { inner: public_key })
    }

    /// Get the public key as bytes
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.inner.to_bytes().to_vec()
    }

    /// Get the public key as a hex string
    #[wasm_bindgen(getter)]
    pub fn hex(&self) -> String {
        hex::encode(self.inner.to_bytes())
    }

    /// Verify a signature
    #[wasm_bindgen]
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, JsValue> {
        if signature.len() != 64 {
            return Err(JsValue::from_str("Signature must be 64 bytes"));
        }
        
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        
        let signature = Signature::from_bytes(&sig_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.inner.verify(message, &signature)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmMessage {
    /// Create a new message
    #[wasm_bindgen(constructor)]
    pub fn new(
        sender_bytes: &[u8],
        recipient_bytes: &[u8],
        content: &str,
    ) -> Result<WasmMessage, JsValue> {
        let sender = create_public_key_from_bytes(sender_bytes)?;
        let recipient = create_public_key_from_bytes(recipient_bytes)?;
        
        let message = Message::new(sender, recipient, content.to_string());
        Ok(WasmMessage { inner: message })
    }

    /// Get the message ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Get the message content
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    /// Get the sender's public key as bytes
    #[wasm_bindgen(getter)]
    pub fn sender_bytes(&self) -> Vec<u8> {
        self.inner.sender.to_bytes().to_vec()
    }

    /// Get the recipient's public key as bytes
    #[wasm_bindgen(getter)]
    pub fn recipient_bytes(&self) -> Vec<u8> {
        self.inner.recipient.to_bytes().to_vec()
    }

    /// Get the timestamp as milliseconds since epoch
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> f64 {
        self.inner.timestamp.timestamp_millis() as f64
    }

    /// Check if the message is signed
    #[wasm_bindgen(getter)]
    pub fn is_signed(&self) -> bool {
        self.inner.is_signed()
    }

    /// Sign the message with a keypair
    #[wasm_bindgen]
    pub fn sign(&mut self, keypair: &WasmKeyPair) -> Result<(), JsValue> {
        self.inner.sign(&keypair.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Verify the message signature
    #[wasm_bindgen]
    pub fn verify_signature(&self) -> Result<bool, JsValue> {
        self.inner.verify_signature()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert message to JSON
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create message from JSON
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<WasmMessage, JsValue> {
        let message: Message = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmMessage { inner: message })
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmProof {
    /// Create a new proof
    #[wasm_bindgen(constructor)]
    pub fn new(proof_type: &str, data: &[u8]) -> Result<WasmProof, JsValue> {
        let proof_type = match proof_type {
            "identity" => ProofType::Identity,
            "message" => ProofType::Message,
            "timestamp" => ProofType::Timestamp,
            "group_membership" => ProofType::GroupMembership,
            "zero_knowledge" => ProofType::ZeroKnowledge,
            _ => return Err(JsValue::from_str("Invalid proof type")),
        };
        
        let proof = Proof::new(proof_type, data.to_vec())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmProof { inner: proof })
    }

    /// Get the proof ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Get the proof type as string
    #[wasm_bindgen(getter)]
    pub fn proof_type(&self) -> String {
        match self.inner.proof_type {
            ProofType::Identity => "identity".to_string(),
            ProofType::Message => "message".to_string(),
            ProofType::Timestamp => "timestamp".to_string(),
            ProofType::GroupMembership => "group_membership".to_string(),
            ProofType::ZeroKnowledge => "zero_knowledge".to_string(),
        }
    }

    /// Get the proof data
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.inner.data.clone()
    }

    /// Get the data hash
    #[wasm_bindgen(getter)]
    pub fn data_hash(&self) -> Vec<u8> {
        self.inner.data_hash().to_vec()
    }

    /// Get the timestamp as milliseconds since epoch
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> f64 {
        self.inner.timestamp.timestamp_millis() as f64
    }

    /// Check if the proof is signed
    #[wasm_bindgen(getter)]
    pub fn is_signed(&self) -> bool {
        self.inner.is_signed()
    }

    /// Sign the proof with a keypair
    #[wasm_bindgen]
    pub fn sign(&mut self, keypair: &WasmKeyPair) -> Result<(), JsValue> {
        self.inner.sign(&keypair.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert proof to JSON
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create proof from JSON
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<WasmProof, JsValue> {
        let proof: Proof = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmProof { inner: proof })
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmProofVerifier {
    /// Verify a proof
    #[wasm_bindgen]
    pub fn verify(proof: &WasmProof) -> Result<bool, JsValue> {
        ProofVerifier::verify(&proof.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Utility functions for WASM

#[cfg(feature = "wasm")]
fn create_public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey, JsValue> {
    if bytes.len() != 32 {
        return Err(JsValue::from_str("Public key must be 32 bytes"));
    }
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(bytes);
    
    PublicKey::from_bytes(&key_bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Initialize the WASM module
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log!("Proof Messenger Protocol WASM module initialized");
}

/// Get the library version
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Generate random bytes using the browser's crypto API
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn random_bytes(length: usize) -> Result<Vec<u8>, JsValue> {
    let mut bytes = vec![0u8; length];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to generate random bytes: {}", e)))?;
    Ok(bytes)
}

/// Utility function to convert hex string to bytes
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, JsValue> {
    hex::decode(hex_string)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex string: {}", e)))
}

/// Utility function to convert bytes to hex string
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_wasm_keypair_generation() {
        let keypair = WasmKeyPair::new().expect("Failed to generate keypair");
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.private_key_bytes().len(), 32);
    }

    #[wasm_bindgen_test]
    fn test_wasm_message_creation() {
        let sender_keypair = WasmKeyPair::new().expect("Failed to generate sender keypair");
        let recipient_keypair = WasmKeyPair::new().expect("Failed to generate recipient keypair");
        
        let message = WasmMessage::new(
            &sender_keypair.public_key_bytes(),
            &recipient_keypair.public_key_bytes(),
            "Hello, WASM world!",
        ).expect("Failed to create message");
        
        assert_eq!(message.content(), "Hello, WASM world!");
        assert!(!message.is_signed());
    }

    #[wasm_bindgen_test]
    fn test_wasm_proof_creation() {
        let proof = WasmProof::new("message", b"test data")
            .expect("Failed to create proof");
        
        assert_eq!(proof.proof_type(), "message");
        assert_eq!(proof.data(), b"test data");
        assert!(!proof.is_signed());
    }
}