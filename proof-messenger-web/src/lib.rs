use wasm_bindgen::prelude::*;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier, SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Set up panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// A. Logging Macros (for Dev Experience)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[wasm_bindgen]
pub fn console_log(s: &str) {
    log(s);
}

#[wasm_bindgen]
pub fn console_warn(s: &str) {
    warn(s);
}

#[wasm_bindgen]
pub fn console_error(s: &str) {
    error(s);
}

/// Generate a random keypair; returns [privkey_bytes, pubkey_bytes]
#[wasm_bindgen]
pub fn generate_keypair_wasm() -> Vec<u8> {
    let keypair = Keypair::generate(&mut OsRng);
    let mut out = Vec::with_capacity(SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH);
    out.extend_from_slice(&keypair.secret.to_bytes());
    out.extend_from_slice(&keypair.public.to_bytes());
    out
}

/// Extract public key from keypair bytes
#[wasm_bindgen]
pub fn get_public_key_from_keypair(keypair_bytes: &[u8]) -> Vec<u8> {
    if keypair_bytes.len() != SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH {
        panic!("Invalid keypair length");
    }
    keypair_bytes[SECRET_KEY_LENGTH..].to_vec()
}

/// Extract private key from keypair bytes
#[wasm_bindgen]
pub fn get_private_key_from_keypair(keypair_bytes: &[u8]) -> Vec<u8> {
    if keypair_bytes.len() != SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH {
        panic!("Invalid keypair length");
    }
    keypair_bytes[..SECRET_KEY_LENGTH].to_vec()
}

// B. Hex/Bytes Helpers
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[wasm_bindgen]
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsValue> {
    hex::decode(hex).map_err(|e| JsValue::from_str(&format!("Hex decode error: {e}")))
}

/// Sign some context data with the secret key
#[wasm_bindgen]
pub fn make_proof_wasm(privkey_bytes: &[u8], context: &[u8]) -> Vec<u8> {
    let secret = ed25519_dalek::SecretKey::from_bytes(privkey_bytes).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    let keypair = Keypair { secret, public };
    let sig = keypair.sign(context);
    sig.to_bytes().to_vec()
}

/// Verify a proof given pubkey, context, and proof (signature)
#[wasm_bindgen]
pub fn verify_proof_wasm(pubkey_bytes: &[u8], context: &[u8], proof_bytes: &[u8]) -> Result<bool, JsValue> {
    let pubkey = PublicKey::from_bytes(pubkey_bytes)
        .map_err(|e| JsValue::from_str(&format!("PublicKey error: {e}")))?;
    let signature = Signature::from_bytes(proof_bytes)
        .map_err(|e| JsValue::from_str(&format!("Signature error: {e}")))?;
    Ok(pubkey.verify(context, &signature).is_ok())
}

/// Validate invitation code format (16-character base32)
#[wasm_bindgen]
pub fn validate_invite_code(code: &str) -> bool {
    code.len() == 16 && code.chars().all(|c| c.is_ascii_alphanumeric())
}

// D. Invite/Proof Functions
/// Generate a cryptographically secure 16-character base32 invite code
#[wasm_bindgen]
pub fn generate_invite_code() -> Result<String, JsValue> {
    use rand::RngCore;
    let mut buf = [0u8; 10];
    OsRng.fill_bytes(&mut buf);
    
    match base32::encode(base32::Alphabet::RFC4648 { padding: false }, &buf).get(..16) {
        Some(code) => Ok(code.to_string()),
        None => Err(JsValue::from_str("Failed to generate invite code"))
    }
}

/// Verify a signature with separate public key
#[wasm_bindgen]
pub fn verify_signature(public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<bool, JsValue> {
    verify_proof_wasm(public_key_bytes, message, signature_bytes)
}

/// Validate public key format
#[wasm_bindgen]
pub fn validate_public_key(public_key_bytes: &[u8]) -> bool {
    public_key_bytes.len() == PUBLIC_KEY_LENGTH && 
    PublicKey::from_bytes(public_key_bytes).is_ok()
}

/// Validate signature format
#[wasm_bindgen]
pub fn validate_signature(signature_bytes: &[u8]) -> bool {
    signature_bytes.len() == 64 && 
    Signature::from_bytes(signature_bytes).is_ok()
}

// E. Message and Proof Classes
#[derive(Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct WasmMessage {
    sender: Vec<u8>,
    recipient: Vec<u8>,
    content: String,
    proof: Option<Vec<u8>>,
    id: String,
    timestamp: String,
}

#[wasm_bindgen]
impl WasmMessage {
    #[wasm_bindgen(constructor)]
    pub fn new(sender: &[u8], recipient: &[u8], content: &str) -> WasmMessage {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
        
        WasmMessage {
            sender: sender.to_vec(),
            recipient: recipient.to_vec(),
            content: content.to_string(),
            proof: None,
            id,
            timestamp,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn sender_hex(&self) -> String {
        hex::encode(&self.sender)
    }
    
    #[wasm_bindgen(getter)]
    pub fn recipient_hex(&self) -> String {
        hex::encode(&self.recipient)
    }
    
    #[wasm_bindgen(getter)]
    pub fn sender_bytes(&self) -> Vec<u8> {
        self.sender.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn recipient_bytes(&self) -> Vec<u8> {
        self.recipient.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> String {
        self.timestamp.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn is_signed(&self) -> bool {
        self.proof.is_some()
    }
    
    pub fn sign(&mut self, keypair_bytes: &[u8]) -> Result<(), JsValue> {
        let secret = SecretKey::from_bytes(&keypair_bytes[0..SECRET_KEY_LENGTH])
            .map_err(|e| JsValue::from_str(&format!("SecretKey error: {e}")))?;
        let public = PublicKey::from_bytes(&keypair_bytes[SECRET_KEY_LENGTH..])
            .map_err(|e| JsValue::from_str(&format!("PublicKey error: {e}")))?;
        let keypair = Keypair { secret, public };
        
        // Create message to sign: sender + recipient + content
        let mut to_sign = self.sender.clone();
        to_sign.extend(&self.recipient);
        to_sign.extend(self.content.as_bytes());
        
        self.proof = Some(keypair.sign(&to_sign).to_bytes().to_vec());
        Ok(())
    }
    
    pub fn verify(&self, pubkey_bytes: &[u8]) -> Result<bool, JsValue> {
        if let Some(ref sig) = self.proof {
            let public = PublicKey::from_bytes(pubkey_bytes)
                .map_err(|e| JsValue::from_str(&format!("PublicKey error: {e}")))?;
            
            // Reconstruct message to verify: sender + recipient + content
            let mut to_sign = self.sender.clone();
            to_sign.extend(&self.recipient);
            to_sign.extend(self.content.as_bytes());
            
            let signature = Signature::from_bytes(sig)
                .map_err(|e| JsValue::from_str(&format!("Signature error: {e}")))?;
            
            Ok(public.verify(&to_sign, &signature).is_ok())
        } else {
            Ok(false)
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    
    pub fn from_json(json: &str) -> Option<WasmMessage> {
        serde_json::from_str(json).ok()
    }
}

// C. Keypair Struct/Class
#[wasm_bindgen]
pub struct WasmKeyPair {
    secret: Vec<u8>,
    public: Vec<u8>,
}

#[wasm_bindgen]
impl WasmKeyPair {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmKeyPair {
        let kp = Keypair::generate(&mut OsRng);
        WasmKeyPair {
            secret: kp.secret.to_bytes().to_vec(),
            public: kp.public.to_bytes().to_vec(),
        }
    }
    
    #[wasm_bindgen(js_name = from_bytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<WasmKeyPair, JsValue> {
        if bytes.len() != SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH {
            return Err(JsValue::from_str("Keypair bytes wrong length"));
        }
        let secret = SecretKey::from_bytes(&bytes[0..SECRET_KEY_LENGTH])
            .map_err(|e| JsValue::from_str(&format!("SecretKey error: {e}")))?;
        let public = PublicKey::from_bytes(&bytes[SECRET_KEY_LENGTH..])
            .map_err(|e| JsValue::from_str(&format!("PublicKey error: {e}")))?;
        Ok(WasmKeyPair {
            secret: secret.to_bytes().to_vec(),
            public: public.to_bytes().to_vec(),
        })
    }
    
    #[wasm_bindgen(getter, js_name = public_key_hex)]
    pub fn public_key_hex(&self) -> String {
        hex::encode(&self.public)
    }
    
    #[wasm_bindgen(getter, js_name = public_key_bytes)]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public.clone()
    }
    
    #[wasm_bindgen(getter, js_name = private_key_bytes)]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.secret.clone()
    }
    
    #[wasm_bindgen(getter, js_name = keypair_bytes)]
    pub fn keypair_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH);
        bytes.extend_from_slice(&self.secret);
        bytes.extend_from_slice(&self.public);
        bytes
    }
    
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, JsValue> {
        let secret = SecretKey::from_bytes(&self.secret)
            .map_err(|e| JsValue::from_str(&format!("SecretKey error: {e}")))?;
        let public = PublicKey::from_bytes(&self.public)
            .map_err(|e| JsValue::from_str(&format!("PublicKey error: {e}")))?;
        let keypair = Keypair { secret, public };
        Ok(keypair.sign(data).to_bytes().to_vec())
    }
}

// Proof wrapper for WASM
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmProof {
    id: String,
    proof_type: String,
    context: Vec<u8>,
    signature: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
    timestamp: String,
}

#[wasm_bindgen]
impl WasmProof {
    #[wasm_bindgen(constructor)]
    pub fn new(proof_type: &str, context: &[u8]) -> WasmProof {
        let id = format!("proof_{}", js_sys::Date::now() as u64);
        let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
        
        WasmProof {
            id,
            proof_type: proof_type.to_string(),
            context: context.to_vec(),
            signature: None,
            public_key: None,
            timestamp,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    pub fn sign(&mut self, keypair: &WasmKeyPair) -> Result<(), JsValue> {
        let signature = keypair.sign(&self.context)?;
        self.signature = Some(signature);
        self.public_key = Some(keypair.public_key_bytes());
        Ok(())
    }
    
    pub fn verify(&self) -> Result<bool, JsValue> {
        if let (Some(ref sig), Some(ref pubkey)) = (&self.signature, &self.public_key) {
            verify_proof_wasm(pubkey, &self.context, sig)
        } else {
            Ok(false)
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// Local Storage wrapper
#[wasm_bindgen]
pub struct LocalStorage;

#[wasm_bindgen]
impl LocalStorage {
    pub fn save(key: &str, value: &str) -> bool {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            storage.set_item(key, value).is_ok()
        } else {
            false
        }
    }
    
    pub fn load(key: &str) -> Option<String> {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            storage.get_item(key).ok().flatten()
        } else {
            None
        }
    }
    
    pub fn remove(key: &str) -> bool {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            storage.remove_item(key).is_ok()
        } else {
            false
        }
    }
}

// Utility functions
#[wasm_bindgen]
pub struct Utils;

#[wasm_bindgen]
impl Utils {
    pub fn validate_invite_code(code: &str) -> bool {
        validate_invite_code(code)
    }
    
    pub fn generate_invite_code() -> Result<String, JsValue> {
        generate_invite_code()
    }
    
    pub fn current_timestamp() -> String {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap()
    }
    
    pub fn format_timestamp(timestamp: &str) -> String {
        // Simple formatting - in a real app you'd use a proper date library
        timestamp.split('T').next().unwrap_or(timestamp).to_string()
    }
}

// Event dispatcher for handling async events
#[wasm_bindgen]
pub struct EventDispatcher {
    callbacks: HashMap<String, js_sys::Function>,
}

#[wasm_bindgen]
impl EventDispatcher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EventDispatcher {
        EventDispatcher {
            callbacks: HashMap::new(),
        }
    }
    
    pub fn on(&mut self, event: &str, callback: &js_sys::Function) {
        self.callbacks.insert(event.to_string(), callback.clone());
    }
    
    pub fn emit(&self, event: &str, data: &JsValue) {
        if let Some(callback) = self.callbacks.get(event) {
            let _ = callback.call1(&JsValue::NULL, data);
        }
    }
}

// WebSocket connection wrapper (placeholder for relay connection)
#[wasm_bindgen]
pub struct RelayConnection {
    url: String,
    websocket: Option<web_sys::WebSocket>,
}

#[wasm_bindgen]
impl RelayConnection {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> RelayConnection {
        RelayConnection {
            url: url.to_string(),
            websocket: None,
        }
    }
    
    pub fn connect(&mut self) -> Result<(), JsValue> {
        let ws = web_sys::WebSocket::new(&self.url)?;
        self.websocket = Some(ws);
        Ok(())
    }
    
    pub fn send(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ref ws) = self.websocket {
            ws.send_with_str(message)?;
        }
        Ok(())
    }
    
    pub fn ready_state(&self) -> u16 {
        if let Some(ref ws) = self.websocket {
            ws.ready_state()
        } else {
            web_sys::WebSocket::CLOSED
        }
    }
}

// F. TDD/Property-Based Testing Example (Rust, not WASM)
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn proof_verifies_for_random_content(seed in any::<u64>()) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            let data = format!("context-{seed}").into_bytes();
            let sig = kp.sign(&data);
            prop_assert!(kp.public.verify(&data, &sig).is_ok());
        }
        
        #[test]
        fn proof_fails_for_modified_content(seed in any::<u64>()) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            let mut data = format!("context-{seed}").into_bytes();
            let sig = kp.sign(&data);
            data.push(0xFF); // tamper
            prop_assert!(!kp.public.verify(&data, &sig).is_ok());
        }
        
        #[test]
        fn keypair_roundtrip_preserves_keys(seed in any::<u64>()) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp1 = Keypair::generate(&mut rng);
            
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&kp1.secret.to_bytes());
            bytes.extend_from_slice(&kp1.public.to_bytes());
            
            let secret = SecretKey::from_bytes(&bytes[0..SECRET_KEY_LENGTH]).unwrap();
            let public = PublicKey::from_bytes(&bytes[SECRET_KEY_LENGTH..]).unwrap();
            let kp2 = Keypair { secret, public };
            
            prop_assert_eq!(kp1.public.to_bytes(), kp2.public.to_bytes());
            prop_assert_eq!(kp1.secret.to_bytes(), kp2.secret.to_bytes());
        }
        
        #[test]
        fn message_signing_is_deterministic(
            content in ".*",
            seed in any::<u64>()
        ) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let kp = Keypair::generate(&mut rng);
            
            let sender = kp.public.to_bytes().to_vec();
            let recipient = kp.public.to_bytes().to_vec(); // Self-message for test
            
            // Create messages manually for testing (avoiding js-sys::Date)
            let mut msg1 = WasmMessage {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content: content.clone(),
                proof: None,
                id: "test-id-1".to_string(),
                timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            };
            let mut msg2 = WasmMessage {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content: content.clone(),
                proof: None,
                id: "test-id-2".to_string(),
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
            
            // Same message should produce same signature
            prop_assert_eq!(msg1.proof, msg2.proof);
        }
        
        #[test]
        fn hex_conversion_roundtrip(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
            let hex = hex::encode(&bytes);
            let decoded = hex::decode(&hex).unwrap();
            prop_assert_eq!(bytes, decoded);
        }
        
        #[test]
        fn invite_code_validation_works(seed in any::<u64>()) {
            use rand::RngCore;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut buf = [0u8; 10];
            rng.fill_bytes(&mut buf);
            
            let code = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &buf);
            let code16 = &code[..16];
            
            prop_assert!(validate_invite_code(code16));
            prop_assert!(!validate_invite_code("invalid"));
            prop_assert!(!validate_invite_code(&code[..15])); // Too short
        }
    }
    
    #[test]
    fn test_basic_keypair_operations() {
        let kp = WasmKeyPair::new();
        assert_eq!(kp.public_key_bytes().len(), PUBLIC_KEY_LENGTH);
        assert_eq!(kp.private_key_bytes().len(), SECRET_KEY_LENGTH);
        assert_eq!(kp.keypair_bytes().len(), SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH);
        
        // Test signing
        let data = b"test message";
        let signature = kp.sign(data).unwrap();
        assert_eq!(signature.len(), 64);
        
        // Test verification
        let valid = verify_signature(&kp.public_key_bytes(), data, &signature).unwrap();
        assert!(valid);
    }
    
    #[test]
    fn test_message_operations() {
        let alice = WasmKeyPair::new();
        let bob = WasmKeyPair::new();
        
        // Create message manually for testing (avoiding js-sys::Date)
        let mut message = WasmMessage {
            sender: alice.public_key_bytes(),
            recipient: bob.public_key_bytes(),
            content: "Hello Bob!".to_string(),
            proof: None,
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        };
        
        assert!(!message.is_signed());
        
        message.sign(&alice.keypair_bytes()).unwrap();
        assert!(message.is_signed());
        
        let valid = message.verify(&alice.public_key_bytes()).unwrap();
        assert!(valid);
        
        // Wrong key should fail
        let invalid = message.verify(&bob.public_key_bytes()).unwrap();
        assert!(!invalid);
    }
}