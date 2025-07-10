# proof-messenger-cli

Terminal app for proof-driven secure messaging demos.
- Uses the proof-messenger-protocol crate
- Command-line onboarding, invites, messaging, proof verification
- Designed for live demos, technical users, and CI

## Usage
```bash
cargo run -- invite
cargo run -- onboard <invite>
cargo run -- send --to <pubkey> --msg "hello"
cargo run -- verify <proof> <context>
```

See --help for full commands.