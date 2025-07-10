# 🖥️ CLI Demo - Simulated Output

## Command 1: Generate Invite (Default Seed)
```bash
$ cargo run -- invite
```
**Output:**
```
Invite: [0, 0, 0, 0, 0, 0, 0, 43], PublicKey: PublicKey([45, 123, 67, 89, 12, 34, 56, 78, 90, 11, 22, 33, 44, 55, 66, 77, 88, 99, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140])
```

## Command 2: Generate Invite (Custom Seed)
```bash
$ cargo run -- invite --seed 12345
```
**Output:**
```
Invite: [0, 0, 0, 0, 0, 0, 48, 57], PublicKey: PublicKey([78, 156, 234, 12, 90, 168, 246, 24, 102, 180, 258, 36, 114, 192, 270, 48, 126, 204, 282, 60, 138, 216, 294, 72, 150, 228, 306, 84, 162, 240, 318, 96])
```

## Command 3: Onboard with Invite
```bash
$ cargo run -- onboard 43
```
**Output:**
```
Onboard proof: [234, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56, 78, 90, 123, 45, 67, 89, 12, 34, 56]
```

## Command 4: Send Message
```bash
$ cargo run -- send --to-pubkey "bob_public_key_123" --msg "Hello Bob!"
```
**Output:**
```
Sending message to bob_public_key_123: 'Hello Bob!'
```

## Command 5: Verify Proof
```bash
$ cargo run -- verify "proof_signature_data" 12345
```
**Output:**
```
Verification for proof 'proof_signature_data' with invite_seed 12345: [DEMO - would verify real proof]
Generated keypair public key: [78, 156, 234, 12, 90, 168, 246, 24, 102, 180, 258, 36, 114, 192, 270, 48, 126, 204, 282, 60, 138, 216, 294, 72, 150, 228, 306, 84, 162, 240, 318, 96]
Invite data: [0, 0, 0, 0, 0, 0, 48, 57]
```

## Help Command
```bash
$ cargo run -- --help
```
**Output:**
```
A simple CLI for proof-messenger protocol

Usage: proof-messenger-cli <COMMAND>

Commands:
  invite   Generate an invite
  onboard  Onboard with an invite seed
  send     Send a message
  verify   Verify a proof
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Invite Help
```bash
$ cargo run -- invite --help
```
**Output:**
```
Generate an invite

Usage: proof-messenger-cli invite [OPTIONS]

Options:
      --seed <SEED>  
  -h, --help         Print help
```

## Send Help
```bash
$ cargo run -- send --help
```
**Output:**
```
Send a message

Usage: proof-messenger-cli send --to-pubkey <TO_PUBKEY> --msg <MSG>

Options:
      --to-pubkey <TO_PUBKEY>  
      --msg <MSG>              
  -h, --help                   Print help
```