# proof-messenger-relay

Minimal relay server for proof-messenger protocol MVP.
- Written in Rust (axum or warp)
- Verifies and relays onboarding, messages
- Logs all actions for demo/audit
- Stateless: no sensitive user data stored

## Running

```bash
cargo run
```