use clap::{Parser, Subcommand};
use proof_messenger_protocol::key::{generate_keypair, generate_keypair_with_seed};
use proof_messenger_protocol::proof::{make_proof, Invite, verify_proof};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Invite {
        #[arg(long)]
        seed: Option<u64>,
    },
    Onboard {
        invite_seed: u64,
    },
    Send {
        #[arg(long)]
        to_pubkey: String,
        #[arg(long)]
        msg: String,
    },
    Verify {
        proof: String,
        invite_seed: u64,
    }
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Invite { seed } => {
            let seed = seed.unwrap_or(42);
            let keypair = generate_keypair_with_seed(seed);
            let invite = Invite::new_with_seed(seed + 1);
            println!("Invite: {:?}, PublicKey: {:?}", invite.data, keypair.public);
        }
        Commands::Onboard { invite_seed } => {
            let keypair = generate_keypair();
            let invite = Invite::new_with_seed(*invite_seed);
            let proof = make_proof(&keypair, &invite);
            println!("Onboard proof: {:?}", proof.to_bytes());
        }
        Commands::Send { to_pubkey, msg } => {
            println!("Sending message to {}: '{}'", to_pubkey, msg);
            // For demo, just print, in a real app would connect to relay server
        }
        Commands::Verify { proof, invite_seed } => {
            let keypair = generate_keypair_with_seed(*invite_seed);
            let invite = Invite::new_with_seed(*invite_seed);
            // For demo, just show the verification process
            println!(
                "Verification for proof '{}' with invite_seed {}: [DEMO - would verify real proof]",
                proof, invite_seed
            );
            println!("Generated keypair public key: {:?}", keypair.public.to_bytes());
            println!("Invite data: {:?}", invite.data);
        }
    }
}