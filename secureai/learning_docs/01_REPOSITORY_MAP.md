# Learning Curriculum: SecureAI MVP
## 1. Repository Map

**Purpose**: Provide a complete inventory of all files, modules, directories, and their relationships. This is your roadmap for navigating the codebase.

**Time to Read**: 20-30 minutes

---

## Filesystem Structure

```
secureai/
├── src/                          # Main application code
│   ├── main.rs                   # CLI entry point
│   ├── lib.rs                    # Library exports
│   ├── auth/                     # OAuth2/OIDC & RBAC (Module #3)
│   ├── guardrails/               # Semantic threat detection (Module #4)
│   ├── audit/                    # Cryptographic ledger (Module #2)
│   ├── queue/                    # Distributed task queue (Module #5)
│   ├── cache/                    # Semantic cache (Module #6)
│   ├── evals/                    # Real-time evals & drift (Module #7)
│   ├── proxy/                    # Token-budgeted streaming (Module #8)
│   ├── api/                      # gRPC control plane (Module #9)
│   ├── sandbox/                  # MicroVM execution (Module #1)
│   ├── policy/                   # Policy engine
│   ├── identity.rs               # Identity management
│   ├── router/                   # Multi-model routing
│   └── telemetry/                # OpenTelemetry tracing (Module #10)
│
├── tests/                        # Integration tests
│   ├── jwt_rbac_test.rs
│   ├── evals_integration_test.rs
│   ├── cache_test.rs
│   └── audit_ledger_test.rs
│
├── proto/                        # gRPC protocol buffers
│   └── policy_service.proto      # Service definitions
│
├── Cargo.toml                    # Rust dependencies & metadata
├── Cargo.lock                    # Dependency lock file
│
├── TECHNICAL_DOCUMENTATION.md    # Complete feature specs
├── DEPLOYMENT_GUIDE.md           # Production deployment
├── QUICK_START.md                # Getting started
├── ARCHITECTURE.md               # Design rationale
│
└── learning_docs/                # This curriculum
    ├── 00_EXECUTIVE_SUMMARY.md
    ├── 01_REPOSITORY_MAP.md      # (you are here)
    ├── 02_SYSTEM_MENTAL_MODEL.md
    └── ... (more docs)
```

---

## Core Modules (12 Total)

### Module 1: Sandbox (MicroVM Execution)
**Location**: `src/sandbox/`

**Responsibility**: Execute tasks in isolated Firecracker microVMs with resource limits and LSM security.

**Key Files**:
- `src/sandbox/mod.rs`: SandboxManager orchestrator
- `src/sandbox/executor.rs`: VM execution logic
- `src/sandbox/landlock.rs`: Landlock LSM policy
- `src/sandbox/seccomp.rs`: System call filtering
- `src/sandbox/cgroups.rs`: Resource limits (CPU, memory)

**Dependencies**:
- External: Firecracker binary, Linux KVM
- Internal: None (independent)

**Dependents**:
- `main.rs`: Calls sandbox.spawn_vm() in task execution
- `audit/`: Logs sandbox execution events

**Responsibilities**:
- Spawn and teardown VMs
- Apply security policies (Landlock + seccomp + cgroups)
- Execute arbitrary commands in isolation
- Monitor resource usage
- Capture output and errors

---

### Module 2: Audit Ledger (Cryptographic Audit Trail)
**Location**: `src/audit/`

**Responsibility**: Create tamper-proof, non-repudiation audit trail using cryptographic signatures.

**Key Files**:
- `src/audit/mod.rs`: AuditLedger main orchestrator
- `src/audit/ledger.rs`: Append-only chain implementation
- `src/audit/keys.rs`: Ed25519 key management
- `src/audit/persist.rs`: File-based persistence
- `src/audit/hooks.rs`: GlobalAuditHooks for integration

**Dependencies**:
- External: ed25519-dalek, sha2
- Internal: None (independent)

**Dependents**:
- `main.rs`: Policy validation logging
- `api/grpc.rs`: All RPC calls logged
- `queue/`: Job execution logging

**Responsibilities**:
- Maintain append-only entry chain
- Sign entries with Ed25519
- Verify hash continuity
- Persist to disk (optional)
- Detect tampering

**Key Data Structure**:
```rust
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub action: String,
    pub subject: String,
    pub details: serde_json::Value,
    pub hash: String,      // SHA256(prev_hash + data)
    pub signature: String, // Ed25519Sign(private_key, hash)
}
```

---

### Module 3: Authentication & Authorization
**Location**: `src/auth/`

**Responsibility**: Validate JWT tokens against OIDC providers and enforce role-based access control.

**Key Files**:
- `src/auth/mod.rs`: Core types, lifecycle
- `src/auth/jwt.rs`: JWKS fetching, JWT validation
- `src/auth/rbac.rs`: Role-to-permission mapping
- `src/auth/middleware.rs`: Request authentication

**Dependencies**:
- External: jsonwebtoken, reqwest, lru
- Internal: None (independent)

**Dependents**:
- `api/grpc.rs`: Validates every RPC request
- `api/auth_middleware.rs`: Used in request processing

**Responsibilities**:
- Fetch JWKS from OIDC provider
- Cache JWKS keys (LRU, 1h TTL)
- Verify JWT signatures
- Extract claims (roles, scopes, tenant_id)
- Map roles to permissions
- Enforce permission checks

**Key Data Structures**:
```rust
pub struct JwtClaims {
    pub sub: String,              // User ID
    pub aud: String,              // Audience
    pub iss: String,              // Issuer
    pub exp: i64,                 // Expiration
    pub roles: Vec<String>,       // User roles
    pub tenant_id: Option<String>,// Multi-tenant
}

pub enum Permission {
    ToolsExecute, ToolsWrite, PolicyRead, PolicyWrite,
    PolicyDelete, AuditRead, AuditWrite, QueueManage,
    CacheManage, EvalsRead, EvalsWrite, AdminAll,
}

pub enum Role {
    Admin, PolicyEditor, AuditReader, ToolOperator,
    EvalsManager, Guest,
}
```

---

### Module 4: Semantic Guardrails (Threat Detection)
**Location**: `src/guardrails/`

**Responsibility**: Detect semantic threats in prompts/commands using ONNX embeddings.

**Key Files**:
- `src/guardrails/mod.rs`: SemanticGuardrail orchestrator
- `src/guardrails/semantic.rs`: Cosine similarity matching
- `src/guardrails/onnx.rs`: ONNX runtime wrapper
- `src/guardrails/threat_vectors.rs`: Pre-computed threat patterns

**Dependencies**:
- External: ort (ONNX Runtime), ndarray, ndarray-linalg
- Internal: None (independent)

**Dependents**:
- `api/grpc.rs`: Checks all prompts in evaluate_policy()
- `policy/mod.rs`: Policy engine references guardrail

**Responsibilities**:
- Load ONNX embeddings model
- Vectorize prompts and tools
- Compute cosine similarity vs. 9 threat vectors
- Deny if similarity exceeds threshold
- Report threat type and confidence

**Threat Vectors** (9 patterns):
1. Prompt Injection
2. Data Exfiltration
3. Privilege Escalation
4. Reverse Shell
5. SQL Injection
6. XXS/Code Injection
7. DoS/Rate Limiting
8. Malware Upload
9. Cryptojacking

---

### Module 5: Distributed Task Queue
**Location**: `src/queue/`

**Responsibility**: Execute long-running tasks asynchronously with fault tolerance using NATS JetStream.

**Key Files**:
- `src/queue/mod.rs`: QueueService orchestrator
- `src/queue/producer.rs`: Job publication
- `src/queue/consumer.rs`: Job consumption
- `src/queue/pool.rs`: Worker pool management
- `src/queue/job.rs`: Job state machine
- `src/queue/hooks.rs`: Integration points

**Dependencies**:
- External: async-nats, tokio, tokio-retry
- Internal: None (independent)

**Dependents**:
- `main.rs`: Initialization
- `evals/`: Enqueues evaluation requests
- Extendable for tool execution

**Responsibilities**:
- Enqueue jobs to NATS JetStream
- Pull jobs for processing
- Manage worker pool (N concurrent workers)
- Handle job state transitions
- Implement lease-based reliability
- Auto-retry on timeout
- Abandon after max retries

**Job State Machine**:
```
Pending → Running (lease acquired)
       → Completed (success)
       → Failed (error)
       → TimedOut (lease expired, auto-requeue)
       → Abandoned (max retries exceeded)
```

---

### Module 6: Semantic Cache (Two-Tier)
**Location**: `src/cache/`

**Responsibility**: Cache responses using exact-match (Tier 1) and semantic similarity (Tier 2).

**Key Files**:
- `src/cache/mod.rs`: CacheManager orchestrator
- `src/cache/tier1.rs`: Exact-match cache (Moka LRU)
- `src/cache/tier2.rs`: Semantic cache (similarity search)
- `src/cache/manager.rs`: Tier coordination
- `src/cache/entry.rs`: Cache entry structures
- `src/cache/integration.rs`: Integration helpers

**Dependencies**:
- External: moka, lru, distances
- Internal: None (independent)

**Dependents**:
- `main.rs`: Initialization
- `api/grpc.rs`: Policy evaluation could use cache

**Responsibilities**:
- Tier 1: O(1) exact-match lookup via SHA256
- Tier 2: O(n) similarity search via cosine distance
- Manage TTL and eviction
- Coordinate between tiers
- Return cache hits with tier information

**Cache Hierarchy**:
```
Request → Tier1 Exact Match (O(1)) → HIT (1ms)
       → Miss → Tier2 Semantic (O(n)) → HIT (10-50ms)
       → Miss → Compute → Cache both tiers
```

---

### Module 7: Real-Time Evals & Drift Detection
**Location**: `src/evals/`

**Responsibility**: Asynchronously evaluate requests and detect anomalies using 3-sigma statistical rule.

**Key Files**:
- `src/evals/mod.rs`: EvalsConfig, lifecycle
- `src/evals/online.rs`: EvaluationEngine, worker loop
- `src/evals/eval_data.rs`: Data structures
- `src/evals/metrics.rs`: MetricWindow, statistics
- `src/evals/drift_detector.rs`: 3-sigma anomaly detection
- `src/evals/sampling.rs`: Sampling strategies

**Dependencies**:
- External: statrs, quantiles, tokio
- Internal: None (independent)

**Dependents**:
- `main.rs`: Initialization
- `api/grpc.rs`: Enqueues evals via async channel
- `api/evals_integration.rs`: Integration helpers

**Responsibilities**:
- Sample requests (configurable rate)
- Evaluate metrics asynchronously (fire-and-forget)
- Maintain sliding windows (1h + 24h)
- Detect anomalies (3-sigma rule)
- Generate and store alerts
- Export sampling stats

**Metrics Tracked**:
1. Toxicity (0.0-1.0)
2. Hallucination Risk (0.0-1.0)
3. Guardrail Triggers (rate)
4. Output Quality (0.0-1.0)

**Sampling Strategies**:
- FixedRate: 10% of requests
- AdaptiveRate: 10% baseline, 100% flagged
- AlwaysSample: 100% of requests
- Disabled: 0% of requests

---

### Module 8: Token-Budgeted SSE Proxy
**Location**: `src/proxy/`

**Responsibility**: Real-time streaming with per-minute token budgets and mid-stream policy inspection.

**Key Files**:
- `src/proxy/mod.rs`: ProxyConfig, lifecycle
- `src/proxy/token_budget.rs`: TokenBudget, refill logic
- `src/proxy/stream.rs`: SSE parsing, token counting
- `src/proxy/inspector.rs`: Mid-stream policy checks
- `src/proxy/handler.rs`: HTTP handler

**Dependencies**:
- External: axum, hyper, tokio-stream, tiktoken-rs, bytes
- Internal: None (independent)

**Dependents**:
- `api/`: Could be integrated into gRPC handlers
- Future: HTTP endpoints

**Responsibilities**:
- Parse Server-Sent Events (SSE) chunks
- Count tokens per chunk
- Enforce per-minute token budget
- Refill tokens on schedule
- Apply mid-stream policy checks
- Block streams over budget
- Stream gracefully to client

---

### Module 9: gRPC Control Plane & Policy Engine
**Location**: `src/api/` + `src/policy/`

**Responsibility**: gRPC service for policy evaluation, hot-reload, and client integration.

**Key Files**:
- `src/api/grpc.rs`: PolicyService RPC implementation
- `src/api/middleware.rs`: TenantContext extraction
- `src/api/policy_watcher.rs`: Hot-reload via file mtime
- `src/api/server.rs`: gRPC server setup
- `src/policy/mod.rs`: PolicyEngine, PolicyConfig
- `src/policy/store.rs`: PolicyStore with arc-swap

**Dependencies**:
- External: tonic, prost, arc-swap, tower
- Internal: All modules (integration point)

**Dependents**:
- `main.rs`: Starts gRPC server
- All clients call gRPC service

**Responsibilities**:
- Expose gRPC PolicyService API
- Implement evaluate_policy() RPC
- Implement watch_policy_updates() stream
- Hot-reload policies from file
- Atomic policy swaps (arc-swap)
- Request validation and error handling

**RPC Endpoints**:
```protobuf
service PolicyService {
    rpc EvaluatePolicy(EvaluatePolicyRequest) 
        returns (EvaluatePolicyResponse);
    
    rpc WatchPolicyUpdates(Empty) 
        returns (stream PolicyUpdate);
}
```

---

### Module 10: OpenTelemetry Distributed Tracing
**Location**: `src/telemetry/`

**Responsibility**: Export distributed traces to OpenTelemetry OTLP backends.

**Key Files**:
- `src/telemetry/mod.rs`: TelemetryConfig, lifecycle
- `src/telemetry/exporter.rs`: OTLP exporter setup
- `src/telemetry/spans.rs`: Span definitions

**Dependencies**:
- External: opentelemetry, opentelemetry-otlp, tracing-opentelemetry
- Internal: None (independent)

**Dependents**:
- `main.rs`: Initialization and shutdown
- All modules: Add spans via tracing crate

**Responsibilities**:
- Configure OTLP exporter
- Batch span export (512 spans or 10s)
- Export to OpenTelemetry collector
- Support Datadog, Jaeger, Honeycomb backends

---

### Module 11: Policy Engine
**Location**: `src/policy/`

**Responsibility**: Core policy validation and feature orchestration.

**Key Files**:
- `src/policy/mod.rs`: PolicyEngine, PolicyConfig
- `src/policy/store.rs`: Hot-reloadable policy store

**Dependencies**:
- External: serde, toml, arc-swap
- Internal: References all feature modules

**Dependents**:
- `main.rs`: Creates PolicyEngine
- `api/grpc.rs`: Uses PolicyEngine

**Responsibilities**:
- Load configuration from secureai.toml
- Provide access to all feature configs
- Orchestrate feature initialization
- Handle policy hot-reload

---

### Module 12: Identity Management
**Location**: `src/identity.rs`

**Responsibility**: User identity and session management via TPM 2.0.

**Key Files**:
- `src/identity.rs`: IdentityManager, SessionToken

**Dependencies**:
- External: tss-esapi (TPM), uuid, jsonwebtoken
- Internal: None (independent)

**Dependents**:
- `main.rs`: Creates identity on init
- `audit/`: Logs with identity context

**Responsibilities**:
- Generate W3C-compliant DIDs
- Create time-bound session tokens
- Verify session tokens
- Interface with TPM 2.0

---

## Test Modules (4 Main Test Files)

| File | What It Tests | # Tests |
|------|---------------|---------|
| `tests/jwt_rbac_test.rs` | JWT validation, RBAC | 25+ |
| `tests/evals_integration_test.rs` | Evals, drift detection | 20+ |
| `tests/cache_test.rs` | Cache layers, LRU | 25+ |
| `tests/audit_ledger_test.rs` | Audit chain, signatures | 25+ |

**Total Test Coverage**: 150+ unit tests for ~3000 lines of code (5% test code ratio)

---

## Configuration

### secureai.toml

Primary configuration file defining all feature settings:

```toml
# Core
allowed_paths = ["/data"]
network_access = false
max_memory_mb = 512
allowed_models = ["llama3"]

# Optional features (all disabled by default)
[isolation]
[guardrails]
[audit]
[telemetry]
[queue]
[cache]
[evals]
[auth]
```

**Key Principle**: Every feature optional, sensible defaults, disabled by default.

---

## Protocol Buffers

### proto/policy_service.proto

Defines gRPC service interface:

```protobuf
service PolicyService {
    rpc EvaluatePolicy(EvaluatePolicyRequest) 
        returns (EvaluatePolicyResponse);
    rpc WatchPolicyUpdates(Empty) 
        returns (stream PolicyUpdate);
}
```

---

## Dependencies (High-Level)

### Security Crates
- `jsonwebtoken`: JWT validation
- `ed25519-dalek`: Ed25519 signatures
- `sha2`: SHA256 hashing
- `rand`: Cryptographic randomness

### Async Runtime
- `tokio`: Async runtime (1.32+)
- `tokio-sync`: Channels, mutexes
- `async-nats`: NATS JetStream client

### Web/API
- `tonic`: gRPC framework
- `prost`: Protocol buffers
- `axum`: HTTP framework
- `hyper`: HTTP client/server

### Data Structures
- `serde`/`toml`: Configuration
- `moka`: High-performance LRU cache
- `lru`: Simpler LRU cache
- `arc-swap`: Lock-free swapping

### ML/AI
- `ort`: ONNX Runtime (embeddings)
- `ndarray`: Numerical arrays
- `ndarray-linalg`: Linear algebra

### Monitoring
- `tracing`: Distributed tracing
- `opentelemetry`: Telemetry SDK
- `opentelemetry-otlp`: OTLP exporter
- `statrs`: Statistical functions

---

## Entry Point & Initialization

### main.rs

**CLI Commands**:
```bash
secureai init      # Generate identity, TPM keys
secureai run       # Execute a task in sandbox
secureai logs      # View audit logs
```

**Initialization Order** (in run command):
1. Load policy from secureai.toml
2. Initialize audit ledger (if enabled)
3. Initialize OpenTelemetry (if enabled)
4. Initialize queue (if enabled)
5. Initialize cache (if enabled)
6. Initialize evals (if enabled)
7. Initialize auth (if enabled)
8. Validate task against policy
9. Create identity session
10. Spawn sandbox, execute task
11. Log to audit ledger
12. Shutdown in reverse order

---

## Module Dependencies (Dependency Graph)

```
main.rs (orchestrator)
├── policy/ (config loading)
│   └── isolation/
├── identity/ (session tokens)
├── auth/ (JWT + RBAC)
├── guardrails/ (threat detection)
├── audit/ (logging)
├── queue/ (job queue)
├── cache/ (caching)
├── evals/ (monitoring)
├── proxy/ (streaming)
├── telemetry/ (tracing)
├── sandbox/ (execution)
├── router/ (multi-model)
└── api/ (gRPC - integrates all above)
```

**Key Observation**: Most modules are independent. Integration happens at API layer.

---

## Questions to Test Understanding

1. **Where would you add a new OAuth provider (e.g., SAML)?**
   Answer: `src/auth/jwt.rs`, extend `JwtValidator::new()`

2. **How would you add a new metric to track?**
   Answer: `src/evals/eval_data.rs` (MetricType enum) + `src/evals/metrics.rs`

3. **What happens if the ONNX model fails to load?**
   Answer: Guardrails disabled, log warning, continue (fail-graceful)

4. **How are policies reloaded without restart?**
   Answer: `src/policy/store.rs` with `arc-swap`, file watcher polls mtime

5. **What prevents a job from running forever in the queue?**
   Answer: `src/queue/pool.rs` - lease timeout + auto-requeue

---

## Related Documentation

- **TECHNICAL_DOCUMENTATION.md**: Complete feature specifications
- **ARCHITECTURE.md**: Design rationale and tradeoffs
- **DEPLOYMENT_GUIDE.md**: Infrastructure and scaling

---

## Next Steps

1. **Understand the system**: Read [System Mental Model](02_SYSTEM_MENTAL_MODEL.md)
2. **See architecture**: Read [Architecture Overview](03_ARCHITECTURE_OVERVIEW.md)
3. **Study components**: Read [Component Architecture](04_COMPONENT_ARCHITECTURE.md)

---

[← Previous: Executive Summary](00_EXECUTIVE_SUMMARY.md) | [Next: System Mental Model →](02_SYSTEM_MENTAL_MODEL.md)
