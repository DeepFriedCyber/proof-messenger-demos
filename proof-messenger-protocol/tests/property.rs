use proptest::prelude::*;
use proof_messenger_protocol::key::generate_keypair_with_seed;
use proof_messenger_protocol::proof::{make_proof, verify_proof, Invite};

proptest! {
    #[test]
    fn prop_valid_proof_roundtrip(seed in any::<u64>(), invite_seed in any::<u64>()) {
        let keypair = generate_keypair_with_seed(seed);
        let invite = Invite::new_with_seed(invite_seed);
        let sig = make_proof(&keypair, &invite);
        prop_assert!(verify_proof(&sig, &keypair.public, &invite));
        
        // Tampered invite should fail
        let mut tampered_invite = Invite::new_with_seed(invite_seed);
        tampered_invite.data.push(0xAA);
        prop_assert!(!verify_proof(&sig, &keypair.public, &tampered_invite));
    }
}