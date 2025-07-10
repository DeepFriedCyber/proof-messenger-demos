use wasm_bindgen::prelude::*;
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier, SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Set up panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// Console logging functions
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

/// Convert bytes to hex string
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
#[wasm_bindgen]
pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex).unwrap_or_default()
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
pub fn verify_proof_wasm(pubkey_bytes: &[u8], context: &[u8], proof_bytes: &[u8]) -> bool {
    match (
        PublicKey::from_bytes(pubkey_bytes),
        Signature::from_bytes(proof_bytes)
    ) {
        (Ok(pubkey), Ok(sig)) => pubkey.verify(context, &sig).is_ok(),
        _ => false
    }
}

/// Validate invitation code format
#[wasm_bindgen]
pub fn validate_invite_code(code: &str) -> bool {
    code.len() == 8 && code.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Generate a random invitation code
#[wasm_bindgen]
pub fn generate_invite_code() -> String {
    use rand::Rng;
    let mut rng = OsRng;
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    (0..8).map(|_| chars[rng.gen_range(0, chars.len())]).collect()
}

// Message structure for WASM
#[derive(Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct WasmMessage {
    id: String,
    sender_hex: String,
    recipient_hex: String,
    content: String,
    timestamp: String,
    signature: Option<Vec<u8>>,
    is_signed: bool,
}

#[wasm_bindgen]
impl WasmMessage {
    #[wasm_bindgen(constructor)]
    pub fn new(sender_bytes: &[u8], recipient_bytes: &[u8], content: &str) -> WasmMessage {
        let id = format!("msg_{}", js_sys::Date::now() as u64);
        let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
        
        WasmMessage {
            id,
            sender_hex: hex::encode(sender_bytes),
            recipient_hex: hex::encode(recipient_bytes),
            content: content.to_string(),
            timestamp,
            signature: None,
            is_signed: false,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn sender_hex(&self) -> String {
        self.sender_hex.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn recipient_hex(&self) -> String {
        self.recipient_hex.clone()
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
        self.is_signed
    }
    
    pub fn sign(&mut self, keypair_bytes: &[u8]) {
        if keypair_bytes.len() != SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH {
            return;
        }
        
        let privkey_bytes = &keypair_bytes[..SECRET_KEY_LENGTH];
        let message_data = format!("{}:{}:{}", self.sender_hex, self.recipient_hex, self.content);
        
        match ed25519_dalek::SecretKey::from_bytes(privkey_bytes) {
            Ok(secret) => {
                let public = ed25519_dalek::PublicKey::from(&secret);
                let keypair = Keypair { secret, public };
                let sig = keypair.sign(message_data.as_bytes());
                self.signature = Some(sig.to_bytes().to_vec());
                self.is_signed = true;
            }
            Err(_) => {}
        }
    }
    
    pub fn verify(&self, pubkey_bytes: &[u8]) -> bool {
        if let Some(ref sig_bytes) = self.signature {
            let message_data = format!("{}:{}:{}", self.sender_hex, self.recipient_hex, self.content);
            verify_proof_wasm(pubkey_bytes, message_data.as_bytes(), sig_bytes)
        } else {
            false
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    
    pub fn from_json(json: &str) -> Option<WasmMessage> {
        serde_json::from_str(json).ok()
    }
}

// KeyPair wrapper for WASM
#[wasm_bindgen]
pub struct WasmKeyPair {
    keypair_bytes: Vec<u8>,
}

#[wasm_bindgen]
impl WasmKeyPair {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmKeyPair {
        WasmKeyPair {
            keypair_bytes: generate_keypair_wasm(),
        }
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Option<WasmKeyPair> {
        if bytes.len() == SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH {
            Some(WasmKeyPair {
                keypair_bytes: bytes.to_vec(),
            })
        } else {
            None
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        get_public_key_from_keypair(&self.keypair_bytes)
    }
    
    #[wasm_bindgen(getter)]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        get_private_key_from_keypair(&self.keypair_bytes)
    }
    
    #[wasm_bindgen(getter)]
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key_bytes())
    }
    
    #[wasm_bindgen(getter)]
    pub fn keypair_bytes(&self) -> Vec<u8> {
        self.keypair_bytes.clone()
    }
    
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        make_proof_wasm(&self.private_key_bytes(), data)
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
    
    pub fn sign(&mut self, keypair: &WasmKeyPair) {
        let signature = keypair.sign(&self.context);
        self.signature = Some(signature);
        self.public_key = Some(keypair.public_key_bytes());
    }
    
    pub fn verify(&self) -> bool {
        if let (Some(ref sig), Some(ref pubkey)) = (&self.signature, &self.public_key) {
            verify_proof_wasm(pubkey, &self.context, sig)
        } else {
            false
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
    
    pub fn generate_invite_code() -> String {
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