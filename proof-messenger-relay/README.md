# proof-messenger-relay

The Proof-Messenger Relay Server is the core verification engine that validates cryptographic proofs and provides non-repudiation guarantees for enterprise transactions.

## Deployment Model: Self-Hosted First

**🏢 Enterprise-Controlled Infrastructure**

The Proof-Messenger Relay Server is a **stateless application designed to be run by you, within your own infrastructure**. This is not a SaaS service - it's a binary you deploy and control.

### Key Characteristics:
- **Stateless Design**: No persistent user data or session state
- **Lightweight**: Distributed as a minimal Docker container
- **Scalable**: Designed for horizontal scaling behind load balancers
- **Secure**: Intended for deployment within your private network
- **Auditable**: All verification logs remain in your infrastructure

### Security Deployment Guidelines:
- **🔒 Private Network Only**: Should be deployed within your private network
- **🚫 Not Public-Facing**: Only accessible by your trusted application backends
- **⚖️ Load Balanced**: Deploy multiple instances for high availability
- **📊 Monitored**: Integrate with your existing monitoring and alerting systems

### Quick Start Examples:

**Docker Deployment:**
```bash
# Run the Relay Server via Docker
docker run -p 8000:8000 proof-messenger/relay-server:latest
```

**Kubernetes Deployment:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proof-messenger-relay
spec:
  replicas: 3
  selector:
    matchLabels:
      app: proof-messenger-relay
  template:
    metadata:
      labels:
        app: proof-messenger-relay
    spec:
      containers:
      - name: relay-server
        image: proof-messenger/relay-server:latest
        ports:
        - containerPort: 8000
        resources:
          requests:
            memory: "64Mi"
            cpu: "250m"
          limits:
            memory: "128Mi"
            cpu: "500m"
```

**Behind Load Balancer:**
```bash
# Example with nginx upstream configuration
upstream proof_messenger_relay {
    server 10.0.1.10:8000;
    server 10.0.1.11:8000;
    server 10.0.1.12:8000;
}
```

## Development Running

```bash
cargo run
```