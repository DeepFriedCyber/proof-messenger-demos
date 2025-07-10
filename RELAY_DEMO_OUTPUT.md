# 🔗 Relay Server Demo - Simulated Output

## Starting the Relay Server
```bash
$ cd proof-messenger-relay
$ cargo run
```
**Output:**
```
   Compiling proof-messenger-relay v0.1.0 (/path/to/proof-messenger-relay)
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
     Running `target/debug/proof-messenger-relay`

🚀 Relay server starting...
📡 Listening on 0.0.0.0:8080
✅ Server ready to accept connections
```

## Testing with curl - Message 1
```bash
$ curl -X POST http://localhost:8080/relay \
  -H "Content-Type: application/json" \
  -d '{
    "msg_type": "message",
    "context": "demo",
    "proof": "proof_data_123",
    "pubkey": "alice_public_key",
    "body": "Hello World!"
  }'
```

**Server Log Output:**
```
Received: Message {
    msg_type: "message",
    context: "demo",
    proof: "proof_data_123",
    pubkey: "alice_public_key",
    body: "Hello World!"
}
```

**curl Response:**
```
Message relayed
```

## Testing with curl - Message 2
```bash
$ curl -X POST http://localhost:8080/relay \
  -H "Content-Type: application/json" \
  -d '{
    "msg_type": "onboard",
    "context": "new_user",
    "proof": "onboard_signature_456",
    "pubkey": "bob_public_key",
    "body": "Bob joining the network"
  }'
```

**Server Log Output:**
```
Received: Message {
    msg_type: "onboard",
    context: "new_user",
    proof: "onboard_signature_456",
    pubkey: "bob_public_key",
    body: "Bob joining the network"
}
```

**curl Response:**
```
Message relayed
```

## Testing with curl - Message 3
```bash
$ curl -X POST http://localhost:8080/relay \
  -H "Content-Type: application/json" \
  -d '{
    "msg_type": "send",
    "context": "alice_to_bob",
    "proof": "message_signature_789",
    "pubkey": "alice_public_key",
    "body": "Hey Bob, welcome to ProofPipe!"
  }'
```

**Server Log Output:**
```
Received: Message {
    msg_type: "send",
    context: "alice_to_bob",
    proof: "message_signature_789",
    pubkey: "alice_public_key",
    body: "Hey Bob, welcome to ProofPipe!"
}
```

**curl Response:**
```
Message relayed
```

## Testing Invalid JSON
```bash
$ curl -X POST http://localhost:8080/relay \
  -H "Content-Type: application/json" \
  -d '{"invalid": "json"}'
```

**Server Log Output:**
```
Error: Failed to deserialize JSON: missing field `msg_type`
```

**curl Response:**
```
HTTP/1.1 400 Bad Request
Content-Type: text/plain

Failed to deserialize the JSON body into the target type: missing field `msg_type` at line 1 column 18
```

## Server Running Continuously
```
🚀 Relay server starting...
📡 Listening on 0.0.0.0:8080
✅ Server ready to accept connections

Received: Message { msg_type: "message", context: "demo", proof: "proof_data_123", pubkey: "alice_public_key", body: "Hello World!" }
Received: Message { msg_type: "onboard", context: "new_user", proof: "onboard_signature_456", pubkey: "bob_public_key", body: "Bob joining the network" }
Received: Message { msg_type: "send", context: "alice_to_bob", proof: "message_signature_789", pubkey: "alice_public_key", body: "Hey Bob, welcome to ProofPipe!" }

[Server continues running, logging all incoming messages...]
```

## Testing with Different Tools

### Using PowerShell (Windows)
```powershell
Invoke-RestMethod -Uri "http://localhost:8080/relay" -Method POST -ContentType "application/json" -Body '{"msg_type":"test","context":"powershell","proof":"ps_proof","pubkey":"ps_key","body":"Hello from PowerShell!"}'
```

### Using JavaScript (Browser Console)
```javascript
fetch('http://localhost:8080/relay', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    msg_type: 'web',
    context: 'browser',
    proof: 'js_proof',
    pubkey: 'browser_key',
    body: 'Hello from browser!'
  })
}).then(r => r.text()).then(console.log);
```

### Using Python
```python
import requests
import json

response = requests.post('http://localhost:8080/relay', 
    headers={'Content-Type': 'application/json'},
    json={
        'msg_type': 'python',
        'context': 'script',
        'proof': 'py_proof',
        'pubkey': 'python_key',
        'body': 'Hello from Python!'
    })
print(response.text)  # "Message relayed"
```