# SecureAI MVP - Quick Start Guide

**Get up and running in 5 minutes!**

---

## Prerequisites

- **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Git**: `git --version`
- **Linux/macOS** (or Docker on Windows)

---

## Option 1: Local Development (Fastest)

### 1. Clone & Build

```bash
git clone https://github.com/secureai/mvp.git
cd secureai
cargo build --release
```

### 2. Create Config

```bash
cat > secureai.toml << 'EOF'
allowed_paths = ["/tmp"]
network_access = false
max_memory_mb = 512
allowed_models = ["llama3"]
EOF
```

### 3. Initialize

```bash
./target/release/secureai init
```

### 4. Run Your First Task

```bash
./target/release/secureai run "What is 2+2?" --model llama3
```

**Output**:
```
🤖 Agent Processing: "What is 2+2?"
--- Result ---
4
--------------
✅ Task complete. Session shredded.
```

---

## Option 2: Docker (5 minutes)

### 1. Build Image

```bash
docker build -t secureai:latest .
```

### 2. Create Config

```bash
cat > secureai.toml << 'EOF'
allowed_paths = ["/data"]
network_access = false
max_memory_mb = 512
allowed_models = ["llama3"]
EOF
```

### 3. Run Container

```bash
docker run -it \
  -v $(pwd)/secureai.toml:/etc/secureai/secureai.toml:ro \
  secureai:latest \
  run "What is 2+2?" \
  --model llama3
```

---

## Enable Features (One at a Time)

### Enable Audit Logging

```toml
[audit]
enabled = true
persistence_enabled = false
```

```bash
cargo build --release
./target/release/secureai run "Your prompt" --model llama3
# ✅ Action logged to audit ledger (Ed25519 signed)
```

### Enable Semantic Guardrails

```toml
[guardrails]
enabled = true
onnx_model_path = "./models/all-MiniLM-L6-v2/model.onnx"
```

**Download ONNX model**:
```bash
mkdir -p models/all-MiniLM-L6-v2
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx \
  -O models/all-MiniLM-L6-v2/model.onnx
```

```bash
cargo build --release
./target/release/secureai run "Your prompt" --model llama3
# ✅ Prompt checked against threat vectors
# ✅ Denied if similarity > threshold
```

### Enable Caching (2x-10x faster!)

```toml
[cache]
enabled = true
tier1_enabled = true
tier2_enabled = true
tier1_capacity = 10000
tier2_capacity = 5000
ttl_secs = 3600
similarity_threshold = 0.95
```

```bash
cargo build --release

# First run: Full execution (500ms)
time ./target/release/secureai run "What is 2+2?" --model llama3
# real    0m0.520s

# Second run (exact match): Cached (10ms)
time ./target/release/secureai run "What is 2+2?" --model llama3
# real    0m0.012s

# Similar prompt: Semantic cache hit (50ms)
time ./target/release/secureai run "What is 2 plus 2?" --model llama3
# real    0m0.045s
```

### Enable Task Queue

```bash
# First, install NATS
docker run -d -p 4222:4222 nats:latest -js
```

```toml
[queue]
enabled = true
nats_url = "nats://localhost:4222"
max_workers = 5
lease_timeout_secs = 30
max_retries = 3
```

```bash
cargo build --release
./target/release/secureai run "Long-running task" --model llama3
# ✅ Job enqueued to NATS JetStream
# ✅ Workers processing in parallel
# ✅ Automatic retry on failure
```

### Enable Real-Time Evals & Drift Detection

```toml
[evals]
enabled = true
sampling_rate = 0.1
boost_flagged_requests = 1.0
anomaly_threshold = 3.0
short_window_hours = 1
long_window_hours = 24
alert_enabled = true
alert_webhook_url = "https://webhook.example.com/alerts"
```

```bash
cargo build --release
./target/release/secureai run "Your prompt" --model llama3
# ✅ 10% of requests evaluated asynchronously
# ✅ Metrics: toxicity, hallucination, quality
# ✅ Drift detection with 3-sigma rule
# ✅ Alerts if anomaly detected
```

### Enable Distributed Tracing

```bash
# Start OpenTelemetry collector
docker run -d \
  -p 4318:4318 \
  -v ./otel-config.yaml:/etc/otel/config.yaml \
  otel/opentelemetry-collector
```

```toml
[telemetry]
enabled = true
otlp_exporter_endpoint = "http://localhost:4318/v1/traces"
batch_size = 512
timeout_secs = 10
```

```bash
cargo build --release
./target/release/secureai run "Your prompt" --model llama3
# ✅ Spans exported to OpenTelemetry
# ✅ View in Jaeger/Datadog/Honeycomb
```

### Enable OAuth2/OIDC Authentication

```toml
[auth]
enabled = true
oidc_discovery_url = "https://auth.example.com"
jwks_cache_ttl_secs = 3600
audience = "api.example.com"
issuer = "https://auth.example.com"
require_tenant_claim = true
```

```bash
cargo build --release

# Test with JWT token from your OIDC provider
export JWT_TOKEN="eyJhbGc..."

grpcurl -H "Authorization: Bearer $JWT_TOKEN" \
  -plaintext localhost:50051 \
  secureai.policy.PolicyService/EvaluatePolicy
# ✅ JWT validated against provider JWKS
# ✅ Roles parsed for RBAC
# ✅ Request allowed/denied based on permissions
```

---

## Running Tests

### All Tests

```bash
cargo test --all
```

### Specific Feature Tests

```bash
# Audit ledger tests
cargo test --test audit_ledger_test

# Cache tests
cargo test --test cache_test

# JWT & RBAC tests
cargo test --test jwt_rbac_test

# Evals tests
cargo test --test evals_integration_test
```

### With Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

---

## Next Steps

### Learn More

- **Full Documentation**: See `TECHNICAL_DOCUMENTATION.md` for complete specs
- **Deployment**: See `DEPLOYMENT_GUIDE.md` for production setup
- **Architecture**: See diagrams in TECHNICAL_DOCUMENTATION.md

### Contribute

1. Pick a feature from `src/` directory
2. Read the module docs
3. Add tests in `tests/`
4. Submit PR

### Performance Tuning

**High Cache Hit Rate**:
```toml
[cache]
tier1_capacity = 50000  # More exact matches
tier2_capacity = 20000  # More semantic cache
similarity_threshold = 0.90  # Lower for more hits
```

**Better Drift Detection**:
```toml
[evals]
sampling_rate = 0.5  # 50% of requests
anomaly_threshold = 2.5  # More sensitive (vs 3.0)
```

**Lower Latency**:
```toml
[guardrails]
# Disable if latency critical
enabled = false
```

---

## Troubleshooting

### "Failed to load ONNX model"

```bash
# Download model
mkdir -p models/all-MiniLM-L6-v2
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx \
  -O models/all-MiniLM-L6-v2/model.onnx

# Update config
[guardrails]
onnx_model_path = "./models/all-MiniLM-L6-v2/model.onnx"
```

### "NATS connection refused"

```bash
# Start NATS
docker run -d -p 4222:4222 nats:latest -js

# Verify
nats server info
```

### "Build failed"

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

---

## Architecture Highlights

```
Request → Auth (JWT) → Guardrails (ONNX) → Cache → Execute → Audit
              ↓            ↓                  ↓       ↓        ↓
          401/403      Deny if threat    2x-10x   MicroVM   Signed
                        detected          faster    sandbox   ledger
```

## Feature Matrix

| Feature | Lines of Code | Test Coverage | Latency Impact |
|---------|--------------|---------------|----------------|
| Auth (OAuth2/OIDC) | 500 | 25+ tests | 1-5ms |
| Guardrails (ONNX) | 400 | 30+ tests | 20-60ms |
| Audit (Ed25519) | 300 | 25+ tests | 2-3ms |
| Cache (LRU+Vector) | 600 | 25+ tests | <1ms (hit) |
| Queue (NATS) | 400 | 30+ tests | 50-100ms (enqueue) |
| Evals (3-sigma) | 500 | 20+ tests | 0ms (async) |
| Tracing (OTLP) | 200 | 10+ tests | <1ms (batch) |
| **Total** | **3000+** | **150+ tests** | **~100-200ms** |

---

## What's Next?

1. **Deploy to Production**: See DEPLOYMENT_GUIDE.md
2. **Integrate with CI/CD**: Add to your ML pipeline
3. **Monitor Performance**: Set up Grafana dashboards
4. **Scale Horizontally**: Deploy to Kubernetes
5. **Customize Policies**: Edit secureai.toml per tenant

---

**Questions?** Check TECHNICAL_DOCUMENTATION.md or open an issue on GitHub.

**Ready to go deeper?** Start with the feature you want to use:
- Security: Audit, Auth, Guardrails
- Performance: Cache, Queue, Evals
- Observability: Tracing, Logging

Happy coding! 🚀
