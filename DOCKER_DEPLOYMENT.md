# 🐳 Docker Deployment Guide for Proof Messenger

## 📋 Overview

This guide provides comprehensive instructions for containerizing and deploying the Proof Messenger Protocol using Docker. The containerization follows TDD principles with built-in testing, security hardening, and production-ready configurations.

## 🏗️ Architecture

### Container Structure
```
┌─────────────────────────────────────────────────────────────┐
│                    Docker Deployment                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Web App       │  │  Relay Server   │  │   Traefik   │ │
│  │   (Nginx)       │  │    (Rust)       │  │ (Proxy/SSL) │ │
│  │   Port: 80      │  │   Port: 8080    │  │ Port: 80/443│ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│           │                     │                    │      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │            proof-messenger-network                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Multi-Stage Build Process
1. **Build Stage**: Compile Rust binaries and build WASM modules
2. **Test Stage**: Run comprehensive test suites (TDD validation)
3. **Runtime Stage**: Minimal, secure production containers

## 🚀 Quick Start

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ available RAM
- 10GB+ available disk space

### 1. Clone and Setup
```bash
git clone <repository-url>
cd proof-messenger
cp .env.example .env
```

### 2. Build and Test
```bash
# Make scripts executable (Linux/macOS)
chmod +x docker-build.sh docker-test.sh

# Build all containers
./docker-build.sh

# Run comprehensive tests
./docker-test.sh
```

### 3. Start Application
```bash
# Development mode
docker-compose --profile dev up -d

# Production mode
docker-compose -f docker-compose.prod.yml up -d
```

### 4. Verify Deployment
```bash
# Check service health
curl http://localhost/health
curl http://localhost:8080/health

# Access application
open http://localhost
```

## 📦 Container Details

### Relay Server Container
- **Base Image**: `debian:bullseye-slim`
- **Runtime User**: Non-root (`proofmessenger`)
- **Security**: Read-only filesystem, no new privileges
- **Health Check**: `/health` endpoint
- **Resource Limits**: 512MB RAM, 1 CPU core

#### Build Process
```dockerfile
# Multi-stage build with testing
FROM rust:1.75-bullseye as builder
# ... build dependencies and compile
FROM builder as tester
# ... run all tests (TDD validation)
FROM debian:bullseye-slim as runtime
# ... minimal runtime environment
```

### Web Application Container
- **Base Image**: `nginx:alpine`
- **Content**: Static files + WASM modules
- **Security**: Custom nginx config with security headers
- **Health Check**: `/health` endpoint
- **Resource Limits**: 256MB RAM, 0.5 CPU core

#### Build Process
```dockerfile
# WASM build stage
FROM rust:1.75-bullseye as wasm-builder
# ... build WASM modules

# Node.js build stage
FROM node:18-bullseye as node-builder
# ... install dependencies

# Test stage
FROM node-builder as tester
# ... run E2E tests

# Production stage
FROM nginx:alpine as production
# ... serve static content
```

## 🔧 Configuration

### Environment Variables

#### Core Configuration
```bash
# Relay Server
RELAY_PORT=8080
RUST_LOG=info
RUST_BACKTRACE=1

# Web Application
WEB_PORT=80

# Domains (for SSL)
WEB_DOMAIN=localhost
RELAY_DOMAIN=relay.localhost
ACME_EMAIL=admin@localhost
```

#### Security Configuration
```bash
# Resource limits
RELAY_MEMORY_LIMIT=512m
WEB_MEMORY_LIMIT=256m

# Health check settings
HEALTH_CHECK_INTERVAL=30s
HEALTH_CHECK_TIMEOUT=10s
HEALTH_CHECK_RETRIES=3
```

### Docker Compose Profiles

#### Development Profile
```bash
docker-compose --profile dev up
```
- Python development server
- Debug logging enabled
- Hot reload capabilities
- Development domains

#### Production Profile
```bash
docker-compose -f docker-compose.prod.yml up
```
- Nginx production server
- SSL termination with Let's Encrypt
- Resource limits enforced
- Security hardening enabled

#### Testing Profile
```bash
docker-compose --profile test up e2e-tests
```
- Automated E2E test execution
- Test result artifacts
- CI/CD integration ready

#### Monitoring Profile
```bash
docker-compose --profile monitoring up
```
- Prometheus metrics collection
- Grafana dashboards
- Application monitoring

## 🧪 Testing Strategy (TDD-Aligned)

### 1. Build-Time Testing
```bash
# Tests run during Docker build
RUN cargo test --release --workspace
RUN npm test
RUN npx playwright test
```

### 2. Container Testing
```bash
# Health check validation
HEALTHCHECK --interval=30s --timeout=10s \
    CMD curl -f http://localhost:8080/health || exit 1
```

### 3. Integration Testing
```bash
# Full stack testing
./docker-test.sh
```

### Test Coverage
- ✅ **Unit Tests**: Rust crypto functions
- ✅ **Integration Tests**: API endpoints
- ✅ **E2E Tests**: Complete user workflows
- ✅ **Container Tests**: Health checks and connectivity
- ✅ **Security Tests**: Vulnerability scanning

## 🔒 Security Features

### Container Security
- **Non-root execution**: All containers run as non-root users
- **Read-only filesystems**: Prevents runtime modifications
- **No new privileges**: Prevents privilege escalation
- **Resource limits**: Prevents resource exhaustion
- **Security scanning**: Automated vulnerability detection

### Network Security
- **Isolated networks**: Containers communicate on private networks
- **SSL termination**: HTTPS with automatic certificate management
- **Security headers**: HSTS, CSP, X-Frame-Options, etc.
- **CORS configuration**: Controlled cross-origin access

### Application Security
- **WASM sandboxing**: Cryptographic operations in secure environment
- **Input validation**: All inputs validated and sanitized
- **Error handling**: No sensitive information in error messages
- **Audit logging**: All operations logged for security monitoring

## 📊 Monitoring and Observability

### Health Checks
```bash
# Application health
curl http://localhost/health
curl http://localhost:8080/health

# Container health
docker-compose ps
```

### Metrics Collection
- **Prometheus**: Application and system metrics
- **Grafana**: Visual dashboards and alerting
- **Docker stats**: Resource usage monitoring
- **Log aggregation**: Centralized logging

### Performance Monitoring
- **Response times**: API endpoint performance
- **Resource usage**: CPU, memory, network
- **Error rates**: Application error tracking
- **Throughput**: Request processing rates

## 🚀 Deployment Scenarios

### Local Development
```bash
# Quick development setup
docker-compose --profile dev up -d
```
- Fast iteration cycles
- Debug logging enabled
- Hot reload capabilities

### Staging Environment
```bash
# Production-like testing
docker-compose -f docker-compose.prod.yml up -d
```
- Production configurations
- SSL certificates (staging)
- Performance testing

### Production Deployment
```bash
# Full production setup
docker-compose -f docker-compose.prod.yml up -d
```
- SSL certificates (production)
- Resource limits enforced
- Monitoring enabled
- Backup strategies

### Cloud Deployment

#### AWS ECS
```bash
# Deploy to AWS ECS
aws ecs create-service --cli-input-json file://ecs-service.json
```

#### Kubernetes
```bash
# Deploy to Kubernetes
kubectl apply -f k8s/
```

#### Google Cloud Run
```bash
# Deploy to Cloud Run
gcloud run deploy proof-messenger --source .
```

## 🔄 CI/CD Integration

### GitHub Actions
```yaml
name: Build and Deploy
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build and Test
        run: |
          ./docker-build.sh
          ./docker-test.sh
```

### GitLab CI
```yaml
stages:
  - build
  - test
  - deploy

build:
  script:
    - ./docker-build.sh

test:
  script:
    - ./docker-test.sh

deploy:
  script:
    - docker-compose -f docker-compose.prod.yml up -d
```

## 🛠️ Maintenance and Operations

### Updates and Upgrades
```bash
# Update containers
docker-compose pull
docker-compose up -d

# Rebuild after code changes
./docker-build.sh
docker-compose up -d --force-recreate
```

### Backup and Recovery
```bash
# Backup volumes
docker run --rm -v proof-messenger_data:/data \
  -v $(pwd):/backup alpine tar czf /backup/backup.tar.gz /data

# Restore volumes
docker run --rm -v proof-messenger_data:/data \
  -v $(pwd):/backup alpine tar xzf /backup/backup.tar.gz -C /
```

### Log Management
```bash
# View logs
docker-compose logs -f

# Log rotation
docker-compose logs --tail=1000 > logs/app.log
```

### Performance Tuning
```bash
# Monitor resource usage
docker stats

# Adjust resource limits
# Edit docker-compose.yml deploy.resources section
```

## 🚨 Troubleshooting

### Common Issues

#### Container Won't Start
```bash
# Check logs
docker-compose logs service-name

# Check health
docker-compose ps

# Rebuild if needed
docker-compose build --no-cache service-name
```

#### Network Connectivity Issues
```bash
# Check networks
docker network ls

# Test connectivity
docker exec container-name curl http://other-container:port/health
```

#### Performance Issues
```bash
# Check resource usage
docker stats

# Check system resources
free -h
df -h
```

#### SSL Certificate Issues
```bash
# Check Traefik logs
docker-compose logs traefik

# Verify domain configuration
# Check DNS settings
# Verify firewall rules
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
export TRAEFIK_LOG_LEVEL=DEBUG

# Restart with debug
docker-compose up -d
```

## 📈 Scaling and High Availability

### Horizontal Scaling
```bash
# Scale relay servers
docker-compose up -d --scale relay-server=3

# Load balancer configuration
# Update Traefik configuration for multiple backends
```

### Database Integration
```bash
# Add PostgreSQL for persistence
# Add Redis for caching
# Configure connection pooling
```

### Multi-Region Deployment
```bash
# Deploy to multiple regions
# Configure DNS-based load balancing
# Implement data replication
```

## 🎯 Success Criteria

### Deployment Validation
- ✅ All containers start successfully
- ✅ Health checks pass
- ✅ E2E tests complete successfully
- ✅ SSL certificates are valid
- ✅ Monitoring is operational

### Performance Benchmarks
- ✅ Response time < 100ms (95th percentile)
- ✅ Memory usage < 512MB per service
- ✅ CPU usage < 50% under normal load
- ✅ Zero downtime deployments

### Security Validation
- ✅ No high/critical vulnerabilities
- ✅ All security headers present
- ✅ Non-root container execution
- ✅ Network isolation functional

---

## 🎉 Conclusion

Your Proof Messenger Protocol is now fully containerized and ready for deployment anywhere! The Docker setup provides:

- **🔒 Security**: Hardened containers with minimal attack surface
- **📊 Observability**: Comprehensive monitoring and logging
- **🚀 Scalability**: Ready for horizontal scaling
- **🧪 Testability**: TDD-aligned testing at every stage
- **🔄 Maintainability**: Easy updates and operations

**Ready for Production**: Your containerized application can now run reliably on any Docker-compatible platform! 🐳