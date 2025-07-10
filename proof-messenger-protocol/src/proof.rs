use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

#[derive(Clone)]
pub struct Invite {
    pub data: Vec<u8>, // e.g., group ID, timestamp, etc.
}

impl Invite {
    pub fn new_with_seed(seed: u64) -> Self {
        let data = seed.to_be_bytes().to_vec();
        Invite { data }
    }
}

pub fn make_proof(keypair: &Keypair, invite: &Invite) -> Signature {
    keypair.sign(&invite.data)
}

pub fn verify_proof(sig: &Signature, public: &PublicKey, invite: &Invite) -> bool {
    public.verify(&invite.data, sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::generate_keypair_with_seed;

    #[test]
    fn test_proof_roundtrip() {
        let keypair = generate_keypair_with_seed(42);
        let invite = Invite::new_with_seed(123);
        
        let sig = make_proof(&keypair, &invite);
        assert!(verify_proof(&sig, &keypair.public, &invite));
    }

    #[test]
    fn test_proof_fails_with_wrong_key() {
        let keypair1 = generate_keypair_with_seed(42);
        let keypair2 = generate_keypair_with_seed(43);
        let invite = Invite::new_with_seed(123);
        
        let sig = make_proof(&keypair1, &invite);
        assert!(!verify_proof(&sig, &keypair2.public, &invite));
    }

    #[test]
    fn test_proof_fails_with_tampered_invite() {
        let keypair = generate_keypair_with_seed(42);
        let invite = Invite::new_with_seed(123);
        let mut tampered_invite = Invite::new_with_seed(123);
        tampered_invite.data.push(0xFF); // Tamper with the data
        
        let sig = make_proof(&keypair, &invite);
        assert!(!verify_proof(&sig, &keypair.public, &tampered_invite));
    }

    #[test]
    fn test_invite_from_seed() {
        let invite1 = Invite::new_with_seed(42);
        let invite2 = Invite::new_with_seed(42);
        let invite3 = Invite::new_with_seed(43);
        
        // Same seed should produce same data
        assert_eq!(invite1.data, invite2.data);
        // Different seed should produce different data
        assert_ne!(invite1.data, invite3.data);
    }
}