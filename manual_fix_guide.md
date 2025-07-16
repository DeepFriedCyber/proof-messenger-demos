# Manual Fix Guide for Rust Protocol Compilation Issues

## 1. Fix the WasmKeyPair Implementation in lib.rs

Open the file `proof-messenger-web/src/lib.rs` in your code editor and make the following changes:

### First Instance (around line 488-496):

Replace:
```rust
    #[wasm_bindgen(getter, js_name = public_key_bytes)]
    pub fn public_key_bytes(&self) -> Vec<u8> {

    #[wasm_bindgen]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.keypair.secret.to_bytes().to_vec()
    }
        self.secure_keypair.public_key_bytes().to_vec()
    }
```

With:
```rust
    #[wasm_bindgen(getter, js_name = public_key_bytes)]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.public_key_bytes().to_vec()
    }
    
    #[wasm_bindgen]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.keypair.secret.to_bytes().to_vec()
    }
```

### Second Instance (around line 549-557):

Replace:
```rust
    #[wasm_bindgen(getter, js_name = public_key_bytes)]
    pub fn public_key_bytes(&self) -> Vec<u8> {

    #[wasm_bindgen]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.keypair.secret.to_bytes().to_vec()
    }
        self.secure_keypair.public_key_bytes().to_vec()
    }
```

With:
```rust
    #[wasm_bindgen(getter, js_name = public_key_bytes)]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.public_key_bytes().to_vec()
    }
    
    #[wasm_bindgen]
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.secure_keypair.keypair.secret.to_bytes().to_vec()
    }
```

## 2. Fix the Property Tests

The property tests in `proof-messenger-web/src/property_tests.rs` already have the correct `_data` prefix for unused variables, so no changes are needed there.

## 3. Fix the DataPolicy Import

The DataPolicy import in `proof-messenger-protocol/src/compliance/context_builder.rs` is already correct, so no changes are needed there.

## Verification

After making these changes, run:
```bash
cargo check --workspace
```

This should resolve all the compilation errors.