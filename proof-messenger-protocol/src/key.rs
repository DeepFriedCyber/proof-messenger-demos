use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::SeedableRng;

pub fn generate_keypair() -> Keypair {
    Keypair::generate(&mut OsRng)
}

pub fn generate_keypair_with_seed(seed: u64) -> Keypair {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    Keypair::generate(&mut rng)
}