# SecureAI MVP - Deployment Guide

**Version**: 1.0  
**Last Updated**: 2026-08-14

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development Setup](#local-development-setup)
3. [Docker Deployment](#docker-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Production Checklist](#production-checklist)
6. [Scaling & High Availability](#scaling--high-availability)
7. [Backup & Disaster Recovery](#backup--disaster-recovery)
8. [Troubleshooting Deployment](#troubleshooting-deployment)

---

## Prerequisites

### System Requirements

**Minimum**:
- CPU: 2 cores (4+ recommended)
- Memory: 4 GB (8+ GB recommended)
- Disk: 20 GB (SSD recommended)
- OS: Linux (Ubuntu 22.04 LTS preferred), macOS, or Docker

**For Firecracker VM Support**:
- KVM support (Intel/AMD CPU with virtualization extensions)
- Linux kernel 5.6+ with KVM enabled
- Firecracker binary installed

### Required Software

```bash
# Rust toolchain (for building from source)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.70+

# Docker (for containerized deployment)
docker --version  # 20.10+

# kubectl (for Kubernetes)
kubectl version  # 1.25+

# NATS CLI (optional, for queue debugging)
brew install nats-io/nats-tools/nats

# grpcurl (for API testing)
brew install grpcurl
```

### External Services

- **OIDC Provider**: Okta, Auth0, Azure AD, or similar (for authentication)
- **NATS JetStream Cluster**: For distributed task queue
- **OpenTelemetry Collector**: For distributed tracing (optional)
- **Firecracker**: For microVM execution

---

## Local Development Setup

### Step 1: Clone Repository

```bash
git clone https://github.com/secureai/mvp.git
cd secureai
```

### Step 2: Create Configuration

```bash
cat > secureai.toml << 'EOF'
# Core policy
allowed_paths = ["/data", "/tmp"]
network_access = false
max_memory_mb = 512
allowed_models = ["llama3", "mistral"]

# Isolation
[isolation]
enable_landlock = true
enable_seccomp = true
enable_cgroups = true
memory_limit_mb = 512
cpu_quota = 1.0
max_processes = 100

# Audit (optional for dev)
[audit]
enabled = true
persistence_enabled = false

# Guardrails (optional for dev)
[guardrails]
enabled = false

# Other subsystems disabled for minimal dev setup
# [telemetry]
# [queue]
# [cache]
# [evals]
# [auth]
EOF
```

### Step 3: Build

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, faster runtime)
cargo build --release

# Verify build
./target/release/secureai --version
```

### Step 4: Initialize

```bash
./target/release/secureai init
# Output:
# ✅ Identity initialized: did:secureai:abc123...
# ✅ TPM keys verified.
```

### Step 5: Run a Test Task

```bash
./target/release/secureai run \
  "What is 2+2?" \
  --model llama3

# Output:
# 🤖 Agent Processing: "What is 2+2?"
# --- Result ---
# 4
# ---------
# ✅ Task complete. Session shredded.
```

### Step 6: Run Tests

```bash
# All tests
cargo test --all

# With logging
RUST_LOG=debug cargo test

# Specific feature tests
cargo test --test jwt_rbac_test
cargo test --test evals_integration_test
cargo test --test cache_test
```

---

## Docker Deployment

### Build Image

```dockerfile
# Dockerfile
FROM rust:1.70 AS builder

WORKDIR /build
COPY . .

# Build with optimizations
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    firecracker \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /build/target/release/secureai /usr/local/bin/

# Copy default config
COPY secureai.toml /etc/secureai/

# Create working directory
WORKDIR /secureai

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD grpcurl -plaintext localhost:50051 list || exit 1

# Default command
ENTRYPOINT ["secureai"]
CMD ["run", "--help"]
```

### Build and Run

```bash
# Build image
docker build -t secureai:latest .

# Run container
docker run -it \
  -v /data:/data:ro \
  -v /etc/secureai:/etc/secureai:ro \
  -p 50051:50051 \
  secureai:latest \
  run "Your prompt here"

# Run with config override
docker run -it \
  -e SECUREAI_ALLOWED_MODELS=llama3,mistral \
  -e SECUREAI_AUDIT_ENABLED=true \
  secureai:latest \
  run "Your prompt"
```

### Docker Compose

```yaml
version: '3.8'

services:
  secureai:
    build: .
    image: secureai:latest
    ports:
      - "50051:50051"  # gRPC
    volumes:
      - ./secureai.toml:/etc/secureai/secureai.toml:ro
      - ./data:/data:ro
      - audit_logs:/var/log/secureai
    environment:
      RUST_LOG: info
      SECUREAI_AUDIT_ENABLED: "true"
      SECUREAI_QUEUE_NATS_URL: "nats://nats:4222"
    depends_on:
      - nats
    healthcheck:
      test: ["CMD", "grpcurl", "-plaintext", "localhost:50051", "list"]
      interval: 30s
      timeout: 10s
      retries: 3

  nats:
    image: nats:latest
    ports:
      - "4222:4222"  # NATS
    command: -js
    volumes:
      - nats_data:/data

  # Optional: OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector:latest
    ports:
      - "4318:4318"  # OTLP HTTP
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml:ro
    command: --config=/etc/otel-collector-config.yaml

volumes:
  nats_data:
  audit_logs:
```

### Run Compose Stack

```bash
docker compose up -d

# Check logs
docker compose logs -f secureai

# Test gRPC service
grpcurl -plaintext localhost:50051 list
```

---

## Kubernetes Deployment

### ConfigMap with Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: secureai-config
  namespace: secureai
data:
  secureai.toml: |
    allowed_paths = ["/data", "/tmp"]
    network_access = false
    max_memory_mb = 512
    allowed_models = ["llama3", "mistral"]

    [isolation]
    enable_landlock = true
    enable_seccomp = true
    enable_cgroups = true

    [audit]
    enabled = true
    persistence_enabled = true
    key_path = "/etc/secureai/audit_keys"
    ledger_path = "/var/log/secureai/audit.log"

    [queue]
    enabled = true
    nats_url = "nats://nats:4222"
    max_workers = 10

    [cache]
    enabled = true
    tier1_enabled = true
    tier2_enabled = true

    [telemetry]
    enabled = true
    otlp_exporter_endpoint = "http://otel-collector:4318/v1/traces"

    [auth]
    enabled = true
    oidc_discovery_url = "https://auth.example.com"
    audience = "api.example.com"
    issuer = "https://auth.example.com"
```

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secureai
  namespace: secureai
  labels:
    app: secureai
    version: v1
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: secureai
  template:
    metadata:
      labels:
        app: secureai
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
    spec:
      serviceAccountName: secureai
      securityContext:
        runAsNonRoot: false  # Requires root for Landlock/seccomp
        runAsUser: 0
        fsGroup: 0
      containers:
      - name: secureai
        image: secureai:latest
        imagePullPolicy: Always
        ports:
        - name: grpc
          containerPort: 50051
          protocol: TCP
        - name: metrics
          containerPort: 9090
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: SECUREAI_QUEUE_NATS_URL
          value: "nats://nats:4222"
        - name: SECUREAI_TELEMETRY_OTLP_EXPORTER_ENDPOINT
          value: "http://otel-collector:4318/v1/traces"
        - name: SECUREAI_AUTH_OIDC_DISCOVERY_URL
          value: "https://auth.example.com"
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: config
          mountPath: /etc/secureai
          readOnly: true
        - name: audit-logs
          mountPath: /var/log/secureai
        - name: data
          mountPath: /data
          readOnly: true
        resources:
          requests:
            cpu: 1000m
            memory: 2Gi
          limits:
            cpu: 2000m
            memory: 4Gi
        livenessProbe:
          exec:
            command:
            - grpcurl
            - -plaintext
            - localhost:50051
            - list
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          exec:
            command:
            - grpcurl
            - -plaintext
            - localhost:50051
            - list
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 5
          failureThreshold: 3
        securityContext:
          allowPrivilegeEscalation: true
          capabilities:
            add:
            - SYS_PTRACE
            - SYS_ADMIN
            - NET_ADMIN
      volumes:
      - name: config
        configMap:
          name: secureai-config
      - name: audit-logs
        emptyDir: {}
      - name: data
        nfs:
          server: nfs-server.example.com
          path: "/data"
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - secureai
              topologyKey: kubernetes.io/hostname
```

### Service Manifest

```yaml
apiVersion: v1
kind: Service
metadata:
  name: secureai
  namespace: secureai
spec:
  type: ClusterIP
  selector:
    app: secureai
  ports:
  - name: grpc
    port: 50051
    targetPort: grpc
    protocol: TCP
```

### Deploy to Kubernetes

```bash
# Create namespace
kubectl create namespace secureai

# Apply ConfigMap
kubectl apply -f configmap.yaml

# Apply Deployment
kubectl apply -f deployment.yaml

# Apply Service
kubectl apply -f service.yaml

# Check status
kubectl get pods -n secureai
kubectl get svc -n secureai

# View logs
kubectl logs -n secureai -f deployment/secureai

# Port forward for testing
kubectl port-forward -n secureai svc/secureai 50051:50051

# Test gRPC
grpcurl -plaintext localhost:50051 list
```

### Kubernetes Secret Management

```yaml
# Create secret for OIDC credentials
apiVersion: v1
kind: Secret
metadata:
  name: secureai-oidc
  namespace: secureai
type: Opaque
stringData:
  client-id: "your-client-id"
  client-secret: "your-client-secret"

---
# Reference in deployment
env:
- name: SECUREAI_AUTH_CLIENT_ID
  valueFrom:
    secretKeyRef:
      name: secureai-oidc
      key: client-id
- name: SECUREAI_AUTH_CLIENT_SECRET
  valueFrom:
    secretKeyRef:
      name: secureai-oidc
      key: client-secret
```

---

## Production Checklist

### Pre-Deployment

- [ ] Security audit completed
- [ ] Configuration reviewed by team
- [ ] OIDC provider configured and tested
- [ ] NATS JetStream cluster deployed and tested
- [ ] OpenTelemetry collector deployed
- [ ] SSL/TLS certificates generated
- [ ] Backup strategy planned
- [ ] Disaster recovery plan documented
- [ ] Monitoring and alerting configured
- [ ] Capacity planning completed

### Deployment

- [ ] Container image built and scanned for vulnerabilities
- [ ] Kubernetes manifests reviewed by team
- [ ] Network policies configured for tenant isolation
- [ ] Pod security policies enforced
- [ ] Resource quotas set for each namespace
- [ ] RBAC rules configured
- [ ] Health checks configured (liveness, readiness)
- [ ] Graceful shutdown tested

### Post-Deployment

- [ ] All services passing health checks
- [ ] gRPC endpoint responding
- [ ] Audit logging working
- [ ] Tracing data flowing to collector
- [ ] Cache working (Tier 1 and 2)
- [ ] Queue jobs processing
- [ ] Alerts configured and tested
- [ ] Dashboards set up in monitoring tool
- [ ] On-call runbooks created
- [ ] Team trained on deployment

---

## Scaling & High Availability

### Horizontal Scaling

```bash
# Scale replicas
kubectl scale deployment secureai -n secureai --replicas=5

# Set up auto-scaling
kubectl autoscale deployment secureai -n secureai \
  --min=3 --max=10 --cpu-percent=80
```

### Load Balancing

**Option 1: Service LoadBalancer**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: secureai-lb
spec:
  type: LoadBalancer
  selector:
    app: secureai
  ports:
  - port: 50051
    targetPort: 50051
```

**Option 2: Ingress Controller**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: secureai
spec:
  ingressClassName: nginx
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: secureai
            port:
              number: 50051
```

### Multi-Region Deployment

```yaml
# Deploy to multiple regions
# Region 1: us-east-1
kubectl config use-context us-east-1
kubectl apply -f deployment.yaml

# Region 2: us-west-2
kubectl config use-context us-west-2
kubectl apply -f deployment.yaml

# Global load balancing
# Use DNS-based failover or multi-region service mesh
```

---

## Backup & Disaster Recovery

### Backup Strategy

```bash
# Daily backup of audit ledger
* 2 * * * * /scripts/backup-audit-ledger.sh

# Weekly backup of entire state
0 3 * * 0 /scripts/backup-secureai-state.sh
```

### Backup Script

```bash
#!/bin/bash
BACKUP_DIR="/backups/secureai"
DATE=$(date +%Y%m%d-%H%M%S)

# Backup audit ledger
kubectl exec -n secureai deployment/secureai -- \
  tar czf - /var/log/secureai | \
  gpg --encrypt --recipient backup@example.com | \
  aws s3 cp - s3://backups/audit-$DATE.tar.gz.gpg

# Backup configuration
kubectl get configmap -n secureai -o yaml | \
  gpg --encrypt --recipient backup@example.com | \
  aws s3 cp - s3://backups/config-$DATE.yaml.gpg

# Backup database/persistent state
kubectl exec -n secureai deployment/secureai -- \
  pg_dump audit_db | \
  gzip | \
  gpg --encrypt --recipient backup@example.com | \
  aws s3 cp - s3://backups/database-$DATE.sql.gz.gpg

echo "Backup completed at $(date)"
```

### Disaster Recovery Plan

1. **Data Loss**: Restore from encrypted S3 backup
2. **Service Outage**: Failover to secondary region
3. **Compromised Cluster**: Redeploy from clean template
4. **RTO (Recovery Time Objective)**: < 30 minutes
5. **RPO (Recovery Point Objective)**: < 1 day

---

## Troubleshooting Deployment

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod <pod-name> -n secureai

# Check logs
kubectl logs <pod-name> -n secureai -p  # Previous logs if crashed

# Check resource availability
kubectl top nodes
kubectl top pods -n secureai
```

### gRPC Not Responding

```bash
# Test from pod
kubectl exec -n secureai <pod-name> -- \
  grpcurl -plaintext localhost:50051 list

# Test from external
grpcurl -plaintext <service-ip>:50051 list

# Check network policy
kubectl get networkpolicy -n secureai
```

### Configuration Not Applied

```bash
# Verify ConfigMap loaded
kubectl get configmap -n secureai secureai-config -o yaml

# Check pod environment
kubectl exec -n secureai <pod-name> -- env | grep SECUREAI

# Restart deployment
kubectl rollout restart deployment/secureai -n secureai
```

### Performance Issues

```bash
# Check CPU/memory usage
kubectl top pod <pod-name> -n secureai --containers

# Check query latency
grpcurl -plaintext localhost:50051 \
  -H "Authorization: Bearer $JWT_TOKEN" \
  secureai.policy.PolicyService/EvaluatePolicy

# Profile with pprof
# (Requires debug build with pprof enabled)
curl http://localhost:6060/debug/pprof/profile?seconds=30 > profile.pb.gz
go tool pprof profile.pb.gz
```

---

**End of Deployment Guide**
