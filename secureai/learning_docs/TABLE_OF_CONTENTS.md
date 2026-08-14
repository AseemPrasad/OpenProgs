# Learning Curriculum - Table of Contents

Complete breakdown of all 20 modules in the SecureAI MVP learning curriculum.

---

## Module Descriptions

### ✅ COMPLETED MODULES

#### [Module 00: Executive Summary](00_EXECUTIVE_SUMMARY.md)
**What you'll learn:**
- What this curriculum teaches
- System overview in 2 paragraphs
- Key statistics about the codebase
- Architectural layers diagram
- Ten core features matrix
- Learning outcomes by level
- Who should read this
- Navigation tips
- Core principles demonstrated

**Key Topics:**
- Problem statement
- Design philosophy (security-first, non-breaking evolution, fail-secure, observable, multi-tenant)
- Architectural layers
- Feature matrix
- Learning paths by role
- Time estimates

**Prerequisites:** None

**Time to Read:** 20 minutes

---

#### [Module 01: Repository Map](01_REPOSITORY_MAP.md)
**What you'll learn:**
- Complete filesystem structure
- What each of 12 core modules does
- Key files in each module
- Dependencies between modules
- File locations and responsibilities
- Configuration files
- Test modules
- Entry points

**Core Modules Covered:**
1. Sandbox (MicroVM execution)
2. Audit Ledger (cryptographic trail)
3. Authentication (OAuth2/OIDC + RBAC)
4. Semantic Guardrails (threat detection)
5. Distributed Queue (NATS JetStream)
6. Semantic Cache (LRU + vector)
7. Real-Time Evals (drift detection)
8. Token-Budgeted Proxy (streaming)
9. gRPC Control Plane (policy management)
10. OpenTelemetry (distributed tracing)
11. Policy Engine (orchestrator)
12. Identity Management (TPM keys)

**Key Questions Answered:**
- Where is authentication implemented?
- How does caching work?
- What's in the audit module?
- How are modules connected?
- Where are tests?

**Prerequisites:** None

**Time to Read:** 30 minutes

---

### 🔄 IN PROGRESS

#### [Module 02: System Mental Model](02_SYSTEM_MENTAL_MODEL.md)
**What you'll learn:**
- Problem statement and why it matters
- Major actors (users, clients, providers)
- Major components and their roles
- Data stores (caches, queues, ledger)
- External systems (OIDC, NATS, ONNX)
- Component boundaries
- Data flows (request → response)
- Control flows (initialization → shutdown)
- System as a newcomer would see it

**Sections:**
- The Problem SecureAI Solves
- Actors in the System
- Major Components Overview
- Data Stores & Persistence
- External Dependencies
- System Boundaries
- Major Request Flow
- Major Control Flow
- Mental Model Summary
- Real-World Analogy

**Prerequisites:** Modules 00-01

**Time to Read:** 40 minutes

---

### 📋 PLANNED MODULES

#### [Module 03: Architecture Overview](03_ARCHITECTURE_OVERVIEW.md)
**Topics:**
- Layered architecture pattern
- Layers: API, Security, Threats, Execution, Persistence
- Module boundaries
- Dependency graph
- Isolation patterns
- Entry/exit points
- Request paths through layers
- Feature toggling architecture
- Configuration-driven design

#### [Module 04: Component Architecture](04_COMPONENT_ARCHITECTURE.md)
**Topics (12 modules):**
- Sandbox: VM execution, isolation, resource limits
- Audit: append-only ledger, Ed25519 signing, verification
- Auth: JWKS caching, JWT validation, RBAC mapping
- Guardrails: ONNX embeddings, cosine similarity, threat vectors
- Queue: NATS JetStream, job state machine, worker pool
- Cache: Tier 1 (exact), Tier 2 (semantic), LRU eviction
- Evals: sampling, metrics, drift detection (3-sigma)
- Proxy: token budgeting, SSE parsing, rate limiting
- Control Plane: gRPC service, hot-reload, arc-swap
- Telemetry: OTLP export, spans, batching
- Policy: configuration loading, feature initialization
- Identity: DID generation, session tokens, TPM

**For Each Component:**
- Responsibility
- Key files and functions
- Data structures
- Dependencies (internal + external)
- Integration points
- Design patterns used
- Performance characteristics
- Error handling
- Test examples

#### [Module 05: Data Architecture](05_DATA_ARCHITECTURE.md)
**Topics:**
- Data stores (cache, queue, audit)
- Data models (Config, Claims, Entry, Job)
- Serialization (TOML, JSON, gRPC protobuf)
- State management (arc-swap, RwLock)
- Consistency patterns (eventually consistent)
- Persistence (append-only, file-backed)
- Hot-reload (mtime polling, atomic swap)
- Multitenant isolation (tenant_id propagation)

#### [Module 06: Runtime Flows](06_RUNTIME_FLOWS.md)
**Topics:**
- Request lifecycle (entry → response)
- Happy path trace (all layers)
- Error paths (invalid JWT, denied permission)
- Initialization flow (startup sequence)
- Shutdown flow (graceful termination)
- Task execution flow (sandbox spawn)
- Async evaluation flow (fire-and-forget)
- Cache lookup flow (Tier 1 → Tier 2)
- Job processing flow (enqueue → completion)

**For Each Flow:**
- Sequence diagram
- File/function references
- State transitions
- Error handling
- Performance characteristics

#### [Module 07: Important Features](07_IMPORTANT_FEATURES.md)
**Topics:**
- Feature 1: MicroVM Sandboxing (problem + solution)
- Feature 2: Audit Ledger (compliance + non-repudiation)
- Feature 3: OAuth2 Authentication (enterprise + multi-tenant)
- Feature 4: Semantic Guardrails (threat detection)
- Feature 5: Task Queue (async + fault-tolerance)
- Feature 6: Semantic Cache (2-tier + performance)
- Feature 7: Evals & Drift (monitoring + QA)
- Feature 8: SSE Proxy (streaming + budgeting)
- Feature 9: gRPC Control (hot-reload + policy)
- Feature 10: Distributed Tracing (observability)

**For Each Feature:**
- Problem it solves
- Solution approach
- Key algorithms
- Configuration options
- Metrics/KPIs
- Example usage
- Test cases

#### [Module 08: Design Patterns](08_DESIGN_PATTERNS.md)
**Topics:**
- Dependency injection (PolicyEngine initializes features)
- Repository pattern (PolicyStore abstraction)
- Service layer (RPC handlers call services)
- Factory pattern (JwtValidator, CacheManager creation)
- Adapter pattern (ONNX wrapper, NATS wrapper)
- State machine (Job states: Pending → Running → Completed)
- Observer pattern (file watcher for hot-reload)
- Strategy pattern (Sampling strategies in evals)
- Chain of responsibility (guardrail → cache → compute)
- Async/await pattern (non-blocking I/O)
- Lock-free patterns (arc-swap for policy)

**For Each Pattern:**
- What it does
- Where it's used
- Why it's appropriate
- Alternative approaches
- Tradeoffs

#### [Module 09: Architecture Decisions](09_ARCHITECTURE_DECISIONS.md)
**20+ Decisions Covered:**
1. OAuth2/OIDC over custom auth
2. RBAC static mapping over config
3. Two-tier cache over single
4. Append-only audit over relational DB
5. NATS JetStream over Redis
6. Async/await over threads
7. Monolithic over microservices
8. TOML config over env vars
9. Opt-in features over always-on
10. Fail-secure over fail-open
11. gRPC over REST
12. Lock-free reads (arc-swap)
13. Fire-and-forget evals
14. Semantic guardrails via ONNX
15. Edge case handling
... and more

**For Each Decision:**
- What is being done?
- Where is it implemented?
- Why this design makes sense
- Evidence supporting it
- Alternatives that existed
- Tradeoffs made
- When this stops being appropriate

#### [Module 10: Alternatives & Tradeoffs](10_ALTERNATIVES_TRADEOFFS.md)
**Topics:**
- Speed vs. Security (added layers have cost)
- Consistency vs. Performance (eventual consistency in cache)
- Flexibility vs. Simplicity (many feature options)
- Correctness vs. Performance (semantic search is O(n))
- Scalability vs. Complexity (monolithic vs. distributed)
- Observability vs. Overhead (spans have CPU cost)
- Completeness vs. Timeliness (3-sigma may lag)

**For Each Tradeoff:**
- What's being chosen?
- What's being sacrificed?
- When is this choice optimal?
- When would you choose differently?
- Examples of mismatched choices

#### [Module 11: Security](11_SECURITY.md)
**Topics:**
- Threat model
- Defense in depth (5 layers)
- Authentication (JWT, JWKS, signatures)
- Authorization (RBAC, tenant isolation)
- Threat detection (semantic guardrails)
- Audit & non-repudiation (Ed25519)
- Encryption (TLS/mTLS assumed)
- Secret management (TPM, env vars)
- Attack scenarios
- Vulnerability analysis
- Security assumptions
- Audit trail completeness

#### [Module 12: Reliability](12_RELIABILITY.md)
**Topics:**
- Fault tolerance (job retry, crash recovery)
- Failure modes (what can break?)
- Recovery mechanisms (auto-requeue, restart)
- Health checks (gRPC liveness/readiness)
- Graceful degradation (features disable independently)
- Idempotency (job deduplication)
- Timeouts (lease-based recovery)
- Bulkheads (worker pool limits)
- Circuit breakers (if implemented)
- Retry strategies (exponential backoff patterns)

#### [Module 13: Performance](13_PERFORMANCE.md)
**Topics:**
- Latency breakdown (per-request timing)
- Throughput (concurrent requests)
- Bottlenecks (what's slowest?)
- Optimization strategies
- Cache hit rate targets
- Async benefits (concurrency)
- Lock contention (minimal)
- Memory usage (efficient)
- Database query performance (N/A, append-only)
- Monitoring & profiling
- Load testing scenarios
- Capacity planning

#### [Module 14: Testing](14_TESTING.md)
**Topics:**
- Test strategy (unit + integration)
- Test pyramid (150+ tests for 3000 LOC)
- Test coverage (critical paths)
- Test organization (what/why/how)
- Mock vs. real (NATS, ONNX, OIDC)
- Test data (fixtures)
- Determinism (reproducible results)
- Performance testing
- Chaos testing (fault injection)
- Benchmarking
- CI/CD integration

#### [Module 15: Infrastructure](15_INFRASTRUCTURE.md)
**Topics:**
- Deployment targets (local, Docker, K8s)
- Configuration management
- Scaling (horizontal/vertical)
- Multi-region (geo-distribution)
- High availability (replicas, failover)
- Monitoring (observability)
- Logging (structured logs)
- Alerting (drift alerts, performance)
- Backup & recovery
- Incident response
- Cost optimization

#### [Module 16: Weaknesses & Technical Debt](16_WEAKNESSES_TECHNICAL_DEBT.md)
**Topics:**
- Architectural smells
- Unnecessary abstractions
- Hidden coupling
- Scalability risks
- Security risks
- Reliability risks
- Maintainability problems
- Testing gaps
- Performance risks
- Confusing code patterns

**For Each Issue:**
- What's the problem?
- Evidence it's a problem
- Impact if not addressed
- Difficulty to fix
- Recommended priority

#### [Module 17: Learning Curriculum](17_LEARNING_CURRICULUM.md)
**Topics:**
- Structured learning paths
- Progression (beginner → advanced)
- Hands-on exercises
- Code reading exercises
- Architecture exercises
- Design exercises
- Testing exercises
- Deployment exercises
- Troubleshooting scenarios
- Project ideas

**Learning Paths:**
- Beginner path (understand basics)
- Intermediate path (understand code)
- Advanced path (understand design)
- Architect path (think strategically)
- Senior path (teach others)

#### [Module 18: Knowledge Gaps](18_KNOWLEDGE_GAPS.md)
**Topics:**
- What we can't determine from code
- Assumptions made by design
- Missing documentation
- Implicit requirements
- Performance targets (if any)
- Scalability limits (if any)
- User requirements (inferred)
- Business context (inferred)
- Historical decisions (lost context)
- Future roadmap (unknowable)

**For Each Gap:**
- What don't we know?
- Why does it matter?
- How could we find out?
- What's the impact if wrong?

#### [Module 19: Recommended Deep-Dive Order](19_DEEP_DIVE_ORDER.md)
**Topics:**
- Linear deep-dive path
- Domain-specific paths
- Role-specific paths
- Quick-start path
- Comprehensive path
- Reference usage
- Recommended first modification
- Recommended first test write
- Recommended first feature addition

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total modules | 20 |
| Total estimated read time | 25+ hours |
| Total estimated exercises | 50+ |
| Code references | 200+ |
| Diagrams | 30+ |
| Questions | 150+ |
| Design decisions analyzed | 20+ |
| Patterns identified | 15+ |
| Security issues covered | 20+ |
| Potential improvements identified | 30+ |

---

## How Modules Connect

```
Module 00 (Executive Summary)
    ↓
Module 01 (Repository Map)
    ↓
Module 02 (System Mental Model) ← FOUNDATIONAL
    ↓
Module 03 (Architecture Overview) ← ARCHITECTURE LAYER
    ↓
Module 04 (Component Architecture)
    ├─→ Module 05 (Data Architecture)
    ├─→ Module 06 (Runtime Flows)
    └─→ Module 07 (Important Features)
    ↓
Module 08 (Design Patterns) ← PATTERNS LAYER
    ↓
Module 09 (Architecture Decisions) ← REASONING LAYER
    ├─→ Module 10 (Alternatives & Tradeoffs)
    └─→ Module 11 (Security)
    ↓
Module 12 (Reliability) ← QUALITY LAYER
├─→ Module 13 (Performance)
└─→ Module 14 (Testing)
    ↓
Module 15 (Infrastructure) ← OPERATIONS LAYER
    ↓
Module 16 (Weaknesses & Technical Debt) ← IMPROVEMENTS LAYER
    ↓
Module 17 (Learning Curriculum) ← APPLICATION LAYER
    ↓
Module 18 (Knowledge Gaps) ← REFLECTION LAYER
    ↓
Module 19 (Deep-Dive Order) ← NAVIGATION LAYER
```

---

## Reading Strategies

### Sequential (Complete Understanding)
Start at 00 → Read sequentially through 19. Best for deep learning.

### Breadth-First (Big Picture)
Read: 00 → 01 → 02 → 03 → 19. Then choose deep-dive modules.

### Role-Based (Just What I Need)
Skip to modules for your role. Use README.md learning paths.

### Problem-Focused (Solve a Problem)
Find your problem in Module 16 → Read related modules.

### Design-Focused (Why Decisions)
Read: 00 → 03 → 09 → 10. Skip implementation details.

### Code-Focused (Understand Code)
Read: 01 → 04 → 06 → 14. Focus on how code works.

---

## Modules by Difficulty Level

### Easy (Start Here)
- Module 00: Executive Summary
- Module 01: Repository Map
- Module 02: System Mental Model

### Medium (Intermediate Understanding)
- Module 03: Architecture Overview
- Module 07: Important Features
- Module 08: Design Patterns

### Hard (Deep Knowledge)
- Module 04: Component Architecture
- Module 06: Runtime Flows
- Module 09: Architecture Decisions

### Very Hard (Mastery)
- Module 05: Data Architecture
- Module 10: Alternatives & Tradeoffs
- Module 17: Learning Curriculum

### Context-Dependent (Depends on Background)
- Module 11: Security (hard if new to crypto)
- Module 13: Performance (hard if new to systems)
- Module 15: Infrastructure (hard if new to DevOps)

---

## Cross-References

### By Topic

**Authentication & Authorization**
- Modules: 01 (repo map), 04 (component), 09 (decisions), 11 (security)

**Performance**
- Modules: 04 (cache/queue), 06 (traces), 09 (tradeoffs), 13 (performance)

**Security**
- Modules: 04 (auth/audit), 09 (decisions), 11 (security), 16 (weaknesses)

**Scalability**
- Modules: 03 (architecture), 04 (queue), 13 (performance), 15 (infrastructure)

**Testing**
- Modules: 04 (components), 14 (testing), 17 (exercises)

**Reliability**
- Modules: 04 (queue), 09 (decisions), 12 (reliability), 16 (weaknesses)

---

## Next Steps

1. **Begin Learning**: Start with [Module 00](00_EXECUTIVE_SUMMARY.md)
2. **Choose Path**: Use learning paths in [README.md](README.md)
3. **Deep Dive**: Read relevant modules for your role
4. **Practice**: Do exercises in [Module 17](17_LEARNING_CURRICULUM.md)
5. **Mastery**: Teach others what you've learned

---

[← Back to README](README.md)
