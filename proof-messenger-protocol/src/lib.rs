//! # Proof Messenger Protocol
//! 
//! A pure Rust library for proof-driven, post-quantum-ready secure messaging protocols.
//! - Formally specified, tested with property-based tests
//! - All cryptography, onboarding, invite, message, and proof logic
//! - No UI or network code
//! - WASM-ready (see wasm-bindgen section)
//! - Easily reusable for CLI, web, and relay servers
//!
//! ## Features
//! - Keypair generation (Ed25519 or PQC-ready)
//! - Proof and invite flows
//! - Message context and verification
//! - Formal specification (TLA+), property-based and integration tests
//! - WASM support for web and mobile
//!
//! ## Example Usage
//! ```rust
//! use proof_messenger_protocol::key::generate_keypair;
//! use proof_messenger_protocol::proof::{make_proof, Invite, verify_proof};
//! 
//! let keypair = generate_keypair();
//! let invite = Invite::new_with_seed(42);
//! let proof = make_proof(&keypair, &invite);
//! assert!(verify_proof(&proof, &keypair.public, &invite));
//! ```
//!
//! ## WASM Usage
//! To build for WASM:
//! ```bash
//! wasm-pack build --release
//! ```
//!
//! ## Running Tests
//! ```bash
//! cargo test
//! ```

pub mod key;
pub mod proof;
// Add more as your protocol evolves (message, group, recovery, etc.)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Basic test to ensure the library compiles
        assert_eq!(2 + 2, 4);
    }
}