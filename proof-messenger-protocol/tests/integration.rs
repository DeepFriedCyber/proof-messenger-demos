use proof_messenger_protocol::key::{generate_keypair, generate_keypair_with_seed};
use proof_messenger_protocol::proof::{make_proof, verify_proof, Invite};

#[test]
fn test_readme_example() {
    // This is the exact example from the README
    let keypair = generate_keypair();
    let invite = Invite::new_with_seed(42);
    let sig = make_proof(&keypair, &invite);
    assert!(verify_proof(&sig, &keypair.public, &invite));
}

#[test]
fn test_deterministic_keypairs() {
    // Same seed should produce same keypair
    let keypair1 = generate_keypair_with_seed(12345);
    let keypair2 = generate_keypair_with_seed(12345);
    
    assert_eq!(keypair1.public.to_bytes(), keypair2.public.to_bytes());
    assert_eq!(keypair1.secret.to_bytes(), keypair2.secret.to_bytes());
}

#[test]
fn test_different_seeds_produce_different_keypairs() {
    let keypair1 = generate_keypair_with_seed(1);
    let keypair2 = generate_keypair_with_seed(2);
    
    assert_ne!(keypair1.public.to_bytes(), keypair2.public.to_bytes());
    assert_ne!(keypair1.secret.to_bytes(), keypair2.secret.to_bytes());
}

#[test]
fn test_invite_data_format() {
    let invite = Invite::new_with_seed(0x123456789ABCDEF0);
    
    // Should be 8 bytes (u64 in big-endian format)
    assert_eq!(invite.data.len(), 8);
    assert_eq!(invite.data, 0x123456789ABCDEF0u64.to_be_bytes().to_vec());
}

#[test]
fn test_cross_verification() {
    // Alice and Bob each have their own keypairs
    let alice_keypair = generate_keypair_with_seed(1);
    let bob_keypair = generate_keypair_with_seed(2);
    
    // Alice creates an invite
    let invite = Invite::new_with_seed(42);
    
    // Alice signs the invite
    let alice_sig = make_proof(&alice_keypair, &invite);
    
    // Alice's signature should verify with Alice's public key
    assert!(verify_proof(&alice_sig, &alice_keypair.public, &invite));
    
    // Alice's signature should NOT verify with Bob's public key
    assert!(!verify_proof(&alice_sig, &bob_keypair.public, &invite));
    
    // Bob signs the same invite
    let bob_sig = make_proof(&bob_keypair, &invite);
    
    // Bob's signature should verify with Bob's public key
    assert!(verify_proof(&bob_sig, &bob_keypair.public, &invite));
    
    // Bob's signature should NOT verify with Alice's public key
    assert!(!verify_proof(&bob_sig, &alice_keypair.public, &invite));
}

#[test]
fn test_multiple_invites() {
    let keypair = generate_keypair_with_seed(42);
    
    // Create multiple invites with different seeds
    let invite1 = Invite::new_with_seed(1);
    let invite2 = Invite::new_with_seed(2);
    let invite3 = Invite::new_with_seed(3);
    
    // Sign each invite
    let sig1 = make_proof(&keypair, &invite1);
    let sig2 = make_proof(&keypair, &invite2);
    let sig3 = make_proof(&keypair, &invite3);
    
    // Each signature should verify with its corresponding invite
    assert!(verify_proof(&sig1, &keypair.public, &invite1));
    assert!(verify_proof(&sig2, &keypair.public, &invite2));
    assert!(verify_proof(&sig3, &keypair.public, &invite3));
    
    // Signatures should NOT verify with wrong invites
    assert!(!verify_proof(&sig1, &keypair.public, &invite2));
    assert!(!verify_proof(&sig2, &keypair.public, &invite3));
    assert!(!verify_proof(&sig3, &keypair.public, &invite1));
}