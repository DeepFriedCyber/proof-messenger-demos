use wasm_bindgen::prelude::*;
use ed25519_dalek::Keypair;
use proof_messenger_protocol::key::generate_keypair;

#[wasm_bindgen]
pub fn generate_keypair_js() -> Vec<u8> {
    let kp = generate_keypair();
    kp.public.to_bytes().to_vec()
}

#[wasm_bindgen]
pub fn make_proof_js(pubkey: &[u8], privkey: &[u8], invite_data: &[u8]) -> Vec<u8> {
    // implement a real version using ed25519_dalek, for demo purpose only
    vec![1,2,3]
}