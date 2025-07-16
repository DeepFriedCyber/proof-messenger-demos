// src/main.rs

use clap::{Parser, Subcommand, ValueEnum};
use proof_messenger_protocol::key::{generate_keypair, generate_keypair_with_seed};
use proof_messenger_protocol::proof::{make_proof, Invite};
use serde::Serialize;
use std::fs;

/// Output format for CLI commands
#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Text,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Output format (text or json)
    #[arg(short, long, global = true, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new keypair and save it
    Keygen,
    /// Generate an invite with optional seed
    Invite {
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Create onboarding proof for an invite
    Onboard {
        invite_seed: u64,
    },
    /// Send a message to a recipient
    Send {
        #[arg(long)]
        to_pubkey: String,
        #[arg(long)]
        msg: String,
    },
    /// Verify a proof against an invite
    Verify {
        proof: String,
        invite_seed: u64,
    }
}

// JSON output structures for each command

#[derive(Serialize)]
struct KeygenOutput {
    status: String,
    #[serde(rename = "publicKeyHex")]
    public_key_hex: String,
    #[serde(rename = "keypairFile")]
    keypair_file: String,
}

#[derive(Serialize)]
struct InviteOutput {
    status: String,
    #[serde(rename = "inviteData")]
    invite_data: String,
    #[serde(rename = "publicKeyHex")]
    public_key_hex: String,
    seed: u64,
}

#[derive(Serialize)]
struct OnboardOutput {
    status: String,
    #[serde(rename = "proofHex")]
    proof_hex: String,
    #[serde(rename = "publicKeyHex")]
    public_key_hex: String,
    #[serde(rename = "inviteSeed")]
    invite_seed: u64,
}

#[derive(Serialize)]
struct SendOutput {
    status: String,
    message: String,
    recipient: String,
}

#[derive(Serialize)]
struct VerifyOutput {
    status: String,
    verified: bool,
    proof: String,
    #[serde(rename = "inviteSeed")]
    invite_seed: u64,
}

fn main() {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Keygen => {
            let keypair = generate_keypair();
            let keypair_bytes = keypair.to_bytes();
            let file_path = "keypair.json";
            
            // Save keypair to file (convert to Vec for serialization)
            let keypair_vec: Vec<u8> = keypair_bytes.to_vec();
            fs::write(file_path, serde_json::to_string(&keypair_vec).unwrap())
                .expect("Failed to write keypair file");
            
            // Output based on format
            match cli.output {
                OutputFormat::Json => {
                    let output_data = KeygenOutput {
                        status: "success".to_string(),
                        public_key_hex: hex::encode(keypair.public.to_bytes()),
                        keypair_file: file_path.to_string(),
                    };
                    println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
                }
                OutputFormat::Text => {
                    println!("✅ Keypair generated successfully!");
                    println!("   Public Key: {}", hex::encode(keypair.public.to_bytes()));
                    println!("   Saved to: {}", file_path);
                }
            }
        }
        
        Commands::Invite { seed } => {
            let seed = seed.unwrap_or(42);
            let keypair = generate_keypair_with_seed(seed);
            let invite = Invite::new_with_seed(seed + 1);
            
            match cli.output {
                OutputFormat::Json => {
                    let output_data = InviteOutput {
                        status: "success".to_string(),
                        invite_data: hex::encode(&invite.data),
                        public_key_hex: hex::encode(keypair.public.to_bytes()),
                        seed,
                    };
                    println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
                }
                OutputFormat::Text => {
                    println!("✅ Invite generated successfully!");
                    println!("   Seed: {}", seed);
                    println!("   Invite Data: {}", hex::encode(&invite.data));
                    println!("   Public Key: {}", hex::encode(keypair.public.to_bytes()));
                }
            }
        }
        
        Commands::Onboard { invite_seed } => {
            let keypair = generate_keypair();
            let invite = Invite::new_with_seed(*invite_seed);
            let proof = make_proof(&keypair, &invite);
            
            match cli.output {
                OutputFormat::Json => {
                    let output_data = OnboardOutput {
                        status: "success".to_string(),
                        proof_hex: hex::encode(proof.to_bytes()),
                        public_key_hex: hex::encode(keypair.public.to_bytes()),
                        invite_seed: *invite_seed,
                    };
                    println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
                }
                OutputFormat::Text => {
                    println!("✅ Onboarding proof generated successfully!");
                    println!("   Invite Seed: {}", invite_seed);
                    println!("   Proof: {}", hex::encode(proof.to_bytes()));
                    println!("   Public Key: {}", hex::encode(keypair.public.to_bytes()));
                }
            }
        }
        
        Commands::Send { to_pubkey, msg } => {
            match cli.output {
                OutputFormat::Json => {
                    let output_data = SendOutput {
                        status: "success".to_string(),
                        message: msg.clone(),
                        recipient: to_pubkey.clone(),
                    };
                    println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
                }
                OutputFormat::Text => {
                    println!("✅ Message prepared for sending!");
                    println!("   To: {}", to_pubkey);
                    println!("   Message: '{}'", msg);
                    println!("   Note: In a real app, this would connect to the relay server");
                }
            }
        }
        
        Commands::Verify { proof, invite_seed } => {
            let keypair = generate_keypair_with_seed(*invite_seed);
            let invite = Invite::new_with_seed(*invite_seed);
            
            // For demo purposes, we'll simulate verification
            // In a real implementation, this would parse and verify the actual proof
            let verified = !proof.is_empty(); // Simple demo logic
            
            match cli.output {
                OutputFormat::Json => {
                    let output_data = VerifyOutput {
                        status: "success".to_string(),
                        verified,
                        proof: proof.clone(),
                        invite_seed: *invite_seed,
                    };
                    println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
                }
                OutputFormat::Text => {
                    println!("✅ Verification completed!");
                    println!("   Proof: {}", proof);
                    println!("   Invite Seed: {}", invite_seed);
                    println!("   Verified: {}", if verified { "✅ Yes" } else { "❌ No" });
                    println!("   Generated Public Key: {}", hex::encode(keypair.public.to_bytes()));
                    println!("   Invite Data: {}", hex::encode(&invite.data));
                    println!("   Note: This is a demo verification");
                }
            }
        }
    }
}