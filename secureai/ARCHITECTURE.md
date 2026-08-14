# SecureAI MVP - Architecture & Design Document

**Version**: 1.0  
**Last Updated**: 2026-08-14  
**Audience**: Architects, Senior Engineers, Contributors

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [System Architecture](#system-architecture)
3. [Component Design Decisions](#component-design-decisions)
4. [Data Flow Architecture](#data-flow-architecture)
5. [Concurrency Model](#concurrency-model)
6. [Error Handling Strategy](#error-handling-strategy)
7. [Security Architecture](#security-architecture)
8. [Performance Architecture](#performance-architecture)
9. [Extensibility & Modularity](#extensibility--modularity)
10. [Future Evolution](#future-evolution)

---

## Design Philosophy

### Principles

1. **Security by Default**: All subsystems assume "zero-trust" model
   - No request is trusted until authenticated + authorized
   - All state changes logged to append-only ledger
   - All long-running operations recoverable from failure

2. **Non-Breaking Evolution**: New features added without affecting existing functionality
   - All features opt-in via configuration (disabled by default)
   - Backward compatible API design
   - Subsystem isolation (each feature in separate module)

3. **Fail-Secure, Not Fail-Open**: Errors default to denying access
   - Invalid JWT → 401 Unauthenticated (not "skip auth")
   - Missing permission → 403 PermissionDenied (not "grant access")
   - Guardrail threat → Deny (not "let through")

4. **Observable by Default**: All operations traceable end-to-end
   - Distributed tracing (OpenTelemetry spans)
   - Audit logging (Ed25519 signed ledger)
   - Metrics (cache hits, queue throughput, drift alerts)

5. **Multi-Tenant from Day One**: Data isolation enforced at every layer
   - tenant_id in JWT claims
   - Enforced in request context
   - Validated in policy decisions
   - Admin bypass with audit trail

---

## System Architecture

### Layered Architecture

```
┌─────────────────────────────────────┐
│      API Layer (gRPC)               │
│  EvaluatePolicy, WatchPolicyUpdates │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│    Security Layer (Auth + RBAC)     │
│  JWT validation, permission checks  │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   Policy & Guardrails Layer         │
│  Threat detection, policy eval      │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   Execution Layer (Compute)         │
│  Sandbox, queue, cache, evals       │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   Persistence Layer (Audit)         │
│  Ledger, file storage, telemetry    │
└─────────────────────────────────────┘
```

### Microservices Mindset (Monolithic Implementation)

**Architecture Choice**: Monolithic binary with isolated subsystems

**Why Not Microservices**:
- Lower operational complexity for MVP
- Single binary to deploy
- Easier debugging (all logs in one place)
- No network latency between components

**Future Migration Path**:
- Each subsystem (queue, cache, evals, etc.) could become independent microservice
- gRPC API already designed for external calls
- Pub/sub patterns ready for decoupling

---

## Component Design Decisions

### 1. Authentication: OAuth2/OIDC over Custom

**Decision**: Use industry-standard OAuth2/OIDC instead of custom auth.

**Rationale**:
- ✅ Proven security (used by Google, Microsoft, Amazon)
- ✅ Multi-tenant support built-in (tenant_id claim)
- ✅ No passwords to manage (delegated to provider)
- ✅ Easy integration with enterprise SSO
- ✅ JWKS caching reduces provider load

**Alternative Considered**: Custom JWT + TPM keys
- Would give full control but add security burden
- Harder to integrate with existing corporate systems

**Implementation Details**:
- JWKS cache: LRU with 1-hour TTL + capacity 10
- RS256 signature verification (asymmetric, provider's public key)
- Claim validation: aud (audience), iss (issuer), exp (expiration)

### 2. RBAC: Static Mapping over Dynamic

**Decision**: Hard-coded Role → Permission mapping in code, not in config.

**Rationale**:
- ✅ Immutable (can't accidentally grant wrong permission)
- ✅ Type-safe (enum-based, compile-time checked)
- ✅ Auditable (changes reviewed in code)
- ❌ Requires code change + redeploy for new role

**Alternative Considered**: Config-based RBAC (roles.toml)
- Would allow hot-reload but risky
- Admin could accidentally break access control
- Harder to audit permission changes

**Migration Path**: If dynamic RBAC needed later, can add config layer on top

### 3. Cache: Two-Tier (Exact + Semantic) over Single Tier

**Decision**: Tier 1 (exact-match) + Tier 2 (vector similarity).

**Why Two Tiers**:
- **Tier 1**: 60-80% hit rate on repeated requests (O(1) fast)
- **Tier 2**: Additional 20-30% hit rate on similar requests
- **Combined**: Can provide 2x-10x speedup vs. no cache

**Why Not Just One**:
- Exact-match alone: Misses 20-30% of similar requests
- Vector-based alone: Every request needs embedding compute

**Trade-offs**:
- More memory (keeping two copies)
- More complex eviction policy
- Benefit: Dramatically faster response times

### 4. Audit: Append-Only Ledger over Relational DB

**Decision**: Append-only file + Ed25519 signatures instead of traditional database.

**Rationale**:
- ✅ Simple (just append to file)
- ✅ Tamper-proof (hash chain + crypto signatures)
- ✅ Forensics-friendly (immutable history)
- ✅ Compliance-friendly (SOC2, HIPAA, GDPR ready)
- ❌ No queries (can't search easily)

**Alternative Considered**: PostgreSQL + triggers
- More queryable but requires database
- Tampering risk (admin could delete records)

**Design Details**:
```
Entry N:
  ├─ Hash = SHA256(Hash[N-1] + Data[N])
  └─ Signature = Ed25519Sign(PrivateKey, Hash)

Verification:
  Ed25519Verify(PublicKey, Hash, Signature)
  SHA256(Hash[N-1] + Data[N]) == Hash[N]
```

### 5. Queue: NATS JetStream over Redis/RabbitMQ

**Decision**: NATS JetStream for distributed task queue.

**Rationale**:
- ✅ JetStream = Kafka-like durability in NATS
- ✅ Exactly-once delivery semantics
- ✅ Scales to 1000+ jobs/sec
- ✅ Pull-based consumer (worker controls pace)
- ✅ Automatic requeue on lease timeout

**Alternative Considered**: Celery (Redis backend)
- More Python-centric
- Would need separate service

**Why Pull Not Push**:
- **Pull**: Consumer pulls jobs at own pace (backpressure)
- **Push**: Broker pushes jobs (risk of overwhelming consumer)

### 6. Semantic Cache: LRU + Cosine Similarity over Vector DB

**Decision**: In-memory LRU + custom similarity search instead of dedicated vector DB.

**Rationale**:
- ✅ No external dependency (simpler deployment)
- ✅ Fast (in-process, no network)
- ✅ Sufficient for 5K+ embeddings
- ❌ Doesn't scale to millions of vectors

**Alternative Considered**: Pinecone / Weaviate
- Would scale better but adds operational complexity
- Good future migration path

**Tier 2 Search Algorithm**:
```rust
for entry in tier2_cache {
    similarity = cosine_distance(current_embedding, entry.embedding)
    if similarity > threshold {
        return entry  // Semantic cache hit
    }
}
```

### 7. Concurrency: Async/Await (Tokio) over Threads

**Decision**: Async/await throughout using Tokio runtime.

**Rationale**:
- ✅ 1000s of concurrent tasks on single thread
- ✅ Lower memory (async stack ~ 100 bytes vs thread stack ~ 2 MB)
- ✅ Simpler synchronization (fewer mutexes needed)
- ❌ Requires async-compatible libraries

**Sync Operations (Minimal)**:
- Audit logging: <1ms (acceptable to block)
- RBAC checks: <1ms (acceptable to block)
- Cache lookups: <1ms (acceptable to block)

**Async Operations (Must-Have)**:
- OIDC discovery: ~50-100ms (network I/O)
- JWKS fetch: ~50-100ms (network I/O)
- MicroVM spawn: ~500-1000ms (I/O)
- Task execution: Variable (I/O)

**Synchronization Primitives**:
- `parking_lot::RwLock`: Faster than std RwLock
- `Arc<Mutex<T>>`: For shared mutable state
- `tokio::sync::mpsc`: For async channels (evals queue)
- `atomic` operations: For counters (sampling stats)

### 8. Error Handling: anyhow::Result<T> over thiserror

**Decision**: Use `anyhow::Result` for error propagation, custom error enums only when needed.

**Rationale**:
- ✅ Simple (just Result<T>)
- ✅ Flexible (can wrap any error type)
- ✅ Good error context (can add `.context()` messages)
- ✅ Less boilerplate

**When to Use thiserror**:
- API error types (gRPC Status codes)
- Public library errors

**Example**:
```rust
// Good: anyhow
fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    toml::from_str(&content)
        .context("Failed to parse TOML")?
}

// Error propagates with full context
```

### 9. Configuration: TOML File + Env Vars

**Decision**: secureai.toml as primary config, env vars for overrides.

**Rationale**:
- ✅ TOML = human-readable
- ✅ File-based = version control friendly
- ✅ Env vars = container-friendly (Docker, K8s)
- ✅ Hierarchical = reflects feature structure

**Priority**:
1. Environment variables (highest)
2. secureai.toml (middle)
3. Defaults in code (lowest)

### 10. Observability: OpenTelemetry OTLP

**Decision**: OpenTelemetry for distributed tracing, not application-specific logging.

**Rationale**:
- ✅ Vendor-agnostic (works with Datadog, Jaeger, Honeycomb, etc.)
- ✅ Structured spans (not just logs)
- ✅ Correlation across services (trace_id)
- ✅ Batch export (doesn't block requests)

**Span Hierarchy**:
```
trace_id: "abc123"
├─ span_id: "1": secureai.request (root)
│  ├─ span_id: "1.1": secureai.auth.validate
│  │  ├─ span_id: "1.1.1": secureai.auth.jwt_parse
│  │  └─ span_id: "1.1.2": secureai.auth.jwt_verify
│  ├─ span_id: "1.2": secureai.guardrail.check
│  └─ span_id: "1.3": secureai.sandbox.execute
```

---

## Data Flow Architecture

### Happy Path Flow

```
Client Request
    ↓
┌─────────────────────────────────────┐
│ Extract Authorization Header       │
│ Authorization: Bearer eyJ...        │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ JWT Validation (with JWKS caching) │
│ - Decode header (extract kid)       │
│ - Fetch JWKS (if not cached)        │
│ - Verify signature (RS256)          │
│ - Check exp, aud, iss claims       │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ RBAC Permission Check              │
│ - Parse roles from JWT             │
│ - Map roles → permissions          │
│ - Check if has tools:execute       │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Semantic Guardrail Check           │
│ - Vectorize prompt (ONNX model)    │
│ - Cosine similarity vs. threats    │
│ - Deny if similarity > threshold   │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Policy Evaluation                  │
│ - Check model allowed              │
│ - Check paths allowed              │
│ - Log to audit ledger (Ed25519)    │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Cache Lookup                       │
│ - Tier 1: Exact match (O(1))       │
│ - Tier 2: Semantic search (O(n))   │
│ - Miss: Proceed to execution       │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Sandbox Execution                  │
│ - Spawn Firecracker VM             │
│ - Apply LSM + seccomp + cgroups    │
│ - Execute task                     │
│ - Teardown VM                      │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Async Evaluation (Fire-and-Forget) │
│ - Enqueue to tokio::sync::mpsc     │
│ - Compute metrics (no blocking)    │
│ - Detect drift (3-sigma)           │
│ - Alert if anomaly                 │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Response to Client                 │
│ - Include drift alerts in metadata │
│ - Return gRPC response             │
│ - Export spans to OTLP collector   │
└─────────────────────────────────────┘
```

### Error Path

```
Invalid JWT
    ↓
gRPC Status: 401 Unauthenticated
    ↓
Error Response Sent
    ↓
Audit Log: auth_failure (unsigned, no identity to sign with)
    ↓
Metrics: auth_failure_count++
```

---

## Concurrency Model

### Threading Model

```
Main Thread (Request Handler Loop)
├─ gRPC Service (Tonic)
│  └─ Per-request async task (green thread)
│     ├─ Auth validation (async)
│     ├─ Guardrails check (async)
│     ├─ Policy evaluation (sync, <10ms)
│     ├─ Cache lookup (sync, <1ms)
│     ├─ Sandbox execute (blocking task, Tokio block_in_place)
│     └─ Response serialization (sync, <5ms)
│
├─ Audit Logger (sync thread)
│  └─ Every policy decision logged
│
├─ Policy Watcher (async task)
│  └─ File mtime polling every 1s
│  └─ Atomic policy swap via arc-swap
│
├─ Queue Consumer (async task)
│  └─ Pull jobs from NATS JetStream
│  └─ Dispatch to Worker Pool
│  └─ Heartbeat renewal (every 10s)
│
├─ Evals Worker (async task)
│  └─ Consume from tokio::sync::mpsc
│  └─ Compute metrics
│  └─ Update drift detectors (RwLock)
│
└─ Telemetry Exporter (async task)
   └─ Batch spans every 10s or 512 spans
   └─ HTTP POST to OTLP collector
```

### Lock-Free Patterns

**Policy Store (arc-swap)**:
```rust
// Readers never block
let policy = policy_store.load();  // Atomic load
policy.check_model()               // No lock needed

// Writer updates atomically
let new_policy = PolicyConfig::load("secureai.toml");
policy_store.swap(Arc::new(new_policy));  // Atomic swap
// All future readers get new policy
```

**JWKS Cache (LRU)**:
```rust
// Read path (most common)
{
    let cache = jwks_cache.read();  // RwLock read (shared)
    if let Some(jwks) = cache.get(issuer) {
        return jwks;  // Fast path, no exclusive lock
    }
}

// Write path (rare, only on cache miss)
{
    let mut cache = jwks_cache.write();  // RwLock write (exclusive)
    cache.put(issuer, jwks);
}
```

---

## Error Handling Strategy

### Fail-Secure Defaults

```rust
// Authentication: Fail-secure
match authenticate_request(&request).await {
    Ok(auth_context) => { /* proceed */ }
    Err(status) => return Err(status),  // 401, block request
}

// Authorization: Fail-secure
match check_permission(&auth_context, Permission::ToolsExecute) {
    Ok(()) => { /* proceed */ }
    Err(status) => return Err(status),  // 403, block request
}

// Guardrails: Fail-secure
match guardrail.check_prompt(&prompt).await {
    Ok(GuardrailDecision::Permit) => { /* proceed */ }
    Ok(GuardrailDecision::Deny { reason }) => {
        return Err(format!("Guardrail violation: {}", reason));
    }
    Err(e) => return Err(e),  // Fail-secure, deny on error
}
```

### Error Context Propagation

```rust
// Errors include full context for debugging
fn load_policy(path: &str) -> anyhow::Result<PolicyConfig> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read policy file at {}", path))?;
    
    toml::from_str(&content)
        .context("Failed to parse secureai.toml - check TOML syntax")?
}

// Error stack on failure:
// Error: Failed to parse secureai.toml - check TOML syntax
// Caused by: eof while parsing a table key at line 42 column 5
```

### Non-Fatal Errors (Logged, No Block)

```rust
// Examples where we log but don't block:
// - OpenTelemetry export failure (not critical)
// - Cache eviction (graceful degradation)
// - Queue requeue timeout (auto-retried)
// - Webhook alert failure (non-blocking eval)

if let Err(e) = telemetry::export_spans().await {
    tracing::warn!("Failed to export spans: {}", e);
    // Service continues, telemetry degraded
}
```

---

## Security Architecture

### Defense in Depth

```
Layer 1: Authentication (JWT + OIDC)
    Validates: User identity, expiration, issuer
    Blocks: Unauthenticated or expired tokens

Layer 2: Authorization (RBAC)
    Validates: User has required role/permission
    Blocks: Insufficient permissions (403)

Layer 3: Semantic Threat Detection (ONNX)
    Validates: Prompt doesn't match threat patterns
    Blocks: Threats detected (semantic guardrails)

Layer 4: Isolation (Firecracker + LSM)
    Isolates: Each task in separate VM
    Limits: CPU, memory, processes, files, network

Layer 5: Audit & Non-Repudiation (Ed25519)
    Records: All actions in append-only ledger
    Verifies: Signatures to detect tampering
    Stores: Hash chain to detect modifications
```

### Zero-Trust Principles

**Trust Nothing**: Every request starts unauthenticated
- JWT required, no implicit user assumption
- No "guest" mode (guest role exists but has no permissions)

**Verify Everything**: All claims validated
- JWT signature verified against provider JWKS
- Policy re-evaluated for every request (no caching decisions)
- Tenant_id validated in request context

**Minimal Privilege**: Admin bypass with audit trail
- Admin users marked in logs when accessing other tenants
- Can't hide admin actions (audit trail immutable)

**Audit & Monitor**: Complete observability
- Every action logged (audit ledger)
- Every request traced (OpenTelemetry spans)
- Anomalies detected (3-sigma drift detection)

---

## Performance Architecture

### Latency Optimization

```
Critical Path (must be <100ms):
├─ JWT validation: 1-5ms (JWKS cached)
├─ RBAC check: <1ms (in-memory, no I/O)
├─ Cache lookup: <1ms (Moka LRU O(1))
└─ Policy eval: <10ms (local check)
   TOTAL: ~15-20ms

Non-Critical Path (can be async):
├─ ONNX guardrails: 20-60ms (can pre-check, parallelizable)
├─ Audit logging: 2-3ms (batched, fire-and-forget)
├─ Evals: 0ms (async, fire-and-forget)
└─ Telemetry export: 0ms (batched, async)
   TOTAL: 0ms (non-blocking)
```

### Throughput Optimization

```
Concurrency Model:
├─ Async/await: 1000s of concurrent requests
├─ Tokio runtime: Single thread per core (work-stealing scheduler)
├─ Non-blocking I/O: All I/O async (no thread blocking)
└─ Lock-free reads: Arc-swap, RwLock shared reads

Cache Optimization:
├─ Tier 1: O(1) exact-match (Moka hash table)
├─ Tier 2: O(n) similarity search (LRU with sampling)
├─ Hit rate: 60-80% Tier 1 + 20-30% Tier 2
└─ Result: 2x-10x faster on cache hit

Batch Export:
├─ Evals: Async batch processing (no per-request I/O)
├─ Audit: Append-only file (sequential, no seek)
├─ Telemetry: Batch processor (flush every 512 spans or 10s)
└─ Result: Minimal I/O overhead
```

### Memory Optimization

```
Per-Request Memory:
├─ gRPC request buffer: ~10 KB
├─ Response buffer: ~10 KB
├─ Auth context: ~1 KB
├─ Span context: ~1 KB
├─ Local variables: ~5 KB
└─ TOTAL: ~30 KB per request

Cached Memory (one-time):
├─ JWKS keys: 10 issuers × 5 KB = 50 KB
├─ Semantic cache Tier 2: 5K embeddings × 20 B = 100 KB
├─ Policy store: 50 KB
├─ ONNX model: 100-200 MB (loaded once)
├─ Audit entries: Append-only, unbounded (file-backed)
└─ TOTAL: ~100 MB + model

Scalability:
├─ 1000 concurrent requests: 1000 × 30 KB = 30 MB (green threads)
├─ + 100 MB shared state
└─ TOTAL: ~130 MB (very efficient)
```

---

## Extensibility & Modularity

### Subsystem Isolation

**Each feature is independent**:
```
src/
├── auth/           # Independent module
│   ├── jwt.rs      #   - No dependencies on cache, queue, evals
│   ├── rbac.rs     #   - Pure logic, easy to test
│   └── mod.rs      #   - Minimal integration points
│
├── cache/          # Independent module
│   ├── tier1.rs    #   - No dependencies on auth, queue, evals
│   ├── tier2.rs    #   - Can be enabled/disabled via config
│   └── mod.rs      #
│
└── evals/          # Independent module
    ├── metrics.rs  #   - No dependencies on auth, cache
    ├── sampling.rs #   - Fire-and-forget async
    └── mod.rs      #
```

**Integration is Explicit**:
```rust
// In PolicyEngine only:
if let Some(auth_cfg) = self.get_auth_config() {
    // Initialize auth if configured
}

if let Some(cache_cfg) = self.get_cache_config() {
    // Initialize cache if configured
}

// Not: auth knows about cache knows about evals
```

### Adding a New Feature (10-Step Template)

1. Create module: `src/newfeature/mod.rs`
2. Add types: `src/newfeature/types.rs`
3. Implement logic: `src/newfeature/logic.rs`
4. Extend PolicyConfig: Add `newfeature: Option<Config>`
5. Add to PolicyEngine: Add `get_newfeature_config()` getter
6. Initialize in main.rs: Step 1X init + shutdown
7. Integrate in API: Optional checks in gRPC handlers
8. Add tests: `tests/newfeature_test.rs` (25+ tests)
9. Commit: Following pattern (1 commit per step)
10. Document: Update TECHNICAL_DOCUMENTATION.md

---

## Future Evolution

### Phase 2 (Post-MVP)

**Modular Services**:
- Extract Queue → NATS wrapper microservice
- Extract Cache → Dedicated cache service
- Extract Evals → Evaluation service cluster

**Advanced Features**:
- Ray integration (distributed ML training)
- Model serving (vLLM / ONNX-Runtime cluster)
- Advanced guardrails (few-shot learning with examples)
- Policy versioning (A/B testing policies)

**Scaling**:
- Multi-region deployment (active-active)
- Geo-replication of audit logs
- Global rate limiting (shared rate limit service)

### Phase 3 (Long-term)

**AI-Driven**:
- Automatic threat detection training (learn from evals)
- Policy recommendations (ML-generated policies)
- Anomaly root cause analysis (ML on logs)

**Compliance Automation**:
- Auto-generate compliance reports (SOC2, HIPAA, GDPR)
- Audit trail visualization (interactive timeline)
- Risk scoring (ML-based compliance risk)

### Backward Compatibility Guarantee

- All APIs versioned (v1, v2, etc.)
- Deprecated features supported for 2 major versions
- Configuration format stable (TOML maintained)
- gRPC proto backward compatible (no breaking changes)

---

**End of Architecture Document**

*For implementation details, refer to TECHNICAL_DOCUMENTATION.md*  
*For deployment specifics, refer to DEPLOYMENT_GUIDE.md*
