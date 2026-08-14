# Architecture Diagrams - SecureAI MVP

**Complete set of architecture diagrams derived from repository analysis.**

Time to review: 60-90 minutes

---

## Diagram 1: System Context Diagram

**Shows**: Users, SecureAI system, external systems, and data stores

```mermaid
graph TB
    User["👤 User/Agent"]
    Client["🖥️ Client Application"]
    SecureAI["🔒 SecureAI MVP"]
    
    OIDC["🔐 OIDC Provider<br/>(Okta/Auth0/Azure AD)"]
    Firecracker["🔥 Firecracker VM<br/>Runtime"]
    NATS["📨 NATS JetStream<br/>Queue"]
    ONNX["🧠 ONNX Runtime<br/>(Embeddings)"]
    Collector["📊 OpenTelemetry<br/>Collector"]
    
    User -->|"Submits tasks/prompts"| Client
    Client -->|"gRPC calls"| SecureAI
    
    SecureAI -->|"Validate JWT"| OIDC
    SecureAI -->|"Execute in VM"| Firecracker
    SecureAI -->|"Enqueue jobs"| NATS
    SecureAI -->|"Semantic matching"| ONNX
    SecureAI -->|"Export traces"| Collector
    
    NATS -.->|"Worker pulls jobs"| SecureAI
    OIDC -.->|"JWKS public keys"| SecureAI
    
    style SecureAI fill:#4A90E2,stroke:#2E5C8A,color:#fff
    style User fill:#50C878,stroke:#2E7D4E,color:#fff
    style OIDC fill:#FF6B6B,stroke:#CC5555,color:#fff
    style NATS fill:#FFD93D,stroke:#CC9030,color:#000
```

### How to Read This Diagram

**Solid arrows** show primary flows:
- Users submit work through client applications
- Client talks to SecureAI via gRPC
- SecureAI orchestrates with external systems

**Dotted arrows** show optional/reactive flows:
- NATS pushes jobs back to workers
- OIDC provides keys reactively

**Box colors** indicate system type:
- Blue: Core application
- Green: End users
- Red: Security/Auth
- Yellow: Messaging/Queue

### Important Observations

1. **Asynchronous Integration**: NATS is not synchronous request/response - jobs are pushed and pulled, enabling decoupling
2. **Multi-tenant Design**: OIDC provider is external source of truth for identity/tenancy
3. **No Direct Database**: SecureAI doesn't show a traditional database as primary store (audit is append-only file)
4. **Security at Boundary**: OIDC validation happens at entry (gRPC request)
5. **Optional External Dependencies**: ONNX, NATS, Collector are all optional (can be disabled via config)

### Questions to Test Understanding

1. Why is OIDC external instead of built-in authentication?
   - (Answer: Integrates with enterprise SSO; no passwords to manage; multi-tenant by default)

2. What happens if NATS becomes unavailable?
   - (Answer: Queue feature disabled, evals won't be async; system degrades gracefully)

3. Why does SecureAI connect to Firecracker at all if it's just execution?
   - (Answer: Needs to spawn VMs, monitor, tear down; it orchestrates execution)

4. How would this diagram change if we added a relational database?
   - (Answer: Would need explicit "Database" box; queries/persistence arrows; schema management)

5. Can a user work with SecureAI without going through OIDC?
   - (Answer: No - auth is mandatory for gRPC; can disable in config but not skip in code path)

---

## Diagram 2: Container Diagram

**Shows**: Major deployable units, services, databases, queues

```mermaid
graph TB
    subgraph Clients["🖥️ Clients"]
        CLI["CLI Client"]
        GrpcClient["gRPC Client"]
    end
    
    subgraph SecureAIApp["🔒 SecureAI Application"]
        GrpcServer["gRPC Server<br/>(Tonic)"]
        PolicyEngine["Policy Engine<br/>(Orchestrator)"]
        AuthModule["Auth Module<br/>(JWT + RBAC)"]
        GuardrailsModule["Guardrails<br/>(ONNX)"]
        AuditModule["Audit Ledger<br/>(Ed25519)"]
        CacheModule["Cache Manager<br/>(Tier1 + Tier2)"]
        QueueModule["Queue Service<br/>(NATS)"]
        EvalsModule["Evals Engine<br/>(Drift Detection)"]
        ProxyModule["SSE Proxy<br/>(Streaming)"]
        SandboxModule["Sandbox Manager<br/>(Firecracker)"]
    end
    
    subgraph ExternalServices["🌐 External Services"]
        OIDCProvider["OIDC Provider"]
        NatsCluster["NATS JetStream"]
        OnnxRuntime["ONNX Runtime"]
        FirecrackerBinary["Firecracker"]
        OtelCollector["OpenTelemetry<br/>Collector"]
    end
    
    subgraph DataStores["💾 Data Stores"]
        AuditLedger["Audit Ledger<br/>(Append-only file)"]
        MemoryCache["Memory Cache<br/>(Tier1: Moka)"]
        VectorCache["Vector Cache<br/>(Tier2: LRU)"]
    end
    
    CLI -->|"Tasks"| GrpcServer
    GrpcClient -->|"gRPC"| GrpcServer
    
    GrpcServer -->|"Routes to"| PolicyEngine
    PolicyEngine -->|"Uses"| AuthModule
    PolicyEngine -->|"Uses"| GuardrailsModule
    PolicyEngine -->|"Uses"| CacheModule
    PolicyEngine -->|"Logs to"| AuditModule
    PolicyEngine -->|"Executes"| SandboxModule
    PolicyEngine -->|"Enqueues"| QueueModule
    PolicyEngine -->|"Evaluates async"| EvalsModule
    PolicyEngine -->|"Streams to"| ProxyModule
    
    AuthModule -->|"Validates JWT"| OIDCProvider
    GuardrailsModule -->|"Vectorizes"| OnnxRuntime
    AuditModule -->|"Writes"| AuditLedger
    CacheModule -->|"Reads/Writes"| MemoryCache
    CacheModule -->|"Reads/Writes"| VectorCache
    QueueModule -->|"Connects to"| NatsCluster
    SandboxModule -->|"Spawns"| FirecrackerBinary
    EvalsModule -->|"Fire-and-forget"| EvalsModule
    
    GrpcServer -->|"Exports spans"| OtelCollector
    
    style GrpcServer fill:#4A90E2,color:#fff
    style PolicyEngine fill:#4A90E2,color:#fff
    style SecureAIApp fill:#E8F4F8,stroke:#4A90E2,stroke-width:3px
```

### How to Read This Diagram

**Light blue box** (SecureAIApp): Single deployable container/binary (monolithic)

**Component boxes**: Major subsystems within the application

**Arrows**: Dependencies and data flows between components

**External Services**: Outside the deployment boundary; can be swapped/disabled

**Data Stores**: Persistence mechanisms (some in-memory, some file-based)

### Important Observations

1. **Monolithic but Modular**: Single binary but 10 loosely-coupled modules
2. **Orchestrator Pattern**: PolicyEngine routes all requests through proper layers
3. **Optional Integration**: Auth, Guardrails, Cache, Queue, Evals can all be disabled via config
4. **No Synchronous Database**: Uses append-only file (audit) + in-memory structures (cache)
5. **External Services Are Real Dependencies**: 
   - OIDC is required for auth
   - NATS required for queue
   - Firecracker required for sandbox
   - ONNX required for guardrails

### Questions to Test Understanding

1. Why is PolicyEngine the central orchestrator rather than gRPC server?
   - (Answer: Separates API layer from business logic; allows CLI and gRPC to use same engine)

2. What would happen if CacheModule was removed?
   - (Answer: System would work but slower (no caching); feature is optional)

3. How does data flow from a user request to the audit ledger?
   - (Answer: gRPC → PolicyEngine → Business Logic → AuditModule → File)

4. Why do some modules connect to external services while others don't?
   - (Answer: External services are integration points; internal modules are pure logic)

5. Can EvalsModule work if QueueModule is disabled?
   - (Answer: Yes - evals uses internal tokio channel, not NATS queue)

---

## Diagram 3: Component Diagram - Auth Module

**Shows**: JWT validation, RBAC, permission checking

```mermaid
graph LR
    Request["Incoming Request<br/>(with JWT)"]
    
    subgraph AuthModule["🔐 Auth Module"]
        JwtValidator["JWT Validator"]
        JwksCache["JWKS Cache<br/>(LRU, 1h TTL)"]
        RbacEngine["RBAC Engine"]
        PermissionChecker["Permission Checker"]
    end
    
    OIDCProvider["OIDC Provider<br/>(external)"]
    
    AuthContext["AuthContext<br/>(user_id, roles,<br/>permissions)"]
    
    Request -->|"Extract Bearer token"| JwtValidator
    JwtValidator -->|"Check cache"| JwksCache
    JwksCache -->|"Cache miss"| OIDCProvider
    OIDCProvider -->|"JWKS keys"| JwksCache
    JwksCache -->|"Cached keys"| JwtValidator
    JwtValidator -->|"Verified claims"| RbacEngine
    RbacEngine -->|"Roles → Permissions"| PermissionChecker
    PermissionChecker -->|"Check permission"| AuthContext
    AuthContext -->|"Attached to request"| Request
    
    style JwtValidator fill:#FF6B6B,color:#fff
    style JwksCache fill:#FFD93D,color:#000
    style RbacEngine fill:#95E1D3,color:#000
    style PermissionChecker fill:#95E1D3,color:#000
```

### How to Read This Diagram

**Red box (JwtValidator)**: Cryptographic verification - the security checkpoint

**Yellow box (JwksCache)**: Performance optimization - caches public keys

**Green boxes (RBAC)**: Authorization logic - maps roles to permissions

**Flow**: Request → Validation → Role extraction → Permission mapping → Context attached

### Important Observations

1. **JWKS Caching is Critical**: Every 10th request hits OIDC provider; others use cache
2. **Separation of Concerns**: JWT validation separate from permission checking
3. **Role-Permission Mapping is Immutable**: Hardcoded in RbacEngine, not configurable
4. **Multi-tenant by Default**: tenant_id extracted from JWT claims
5. **Fail-Secure**: Invalid JWT rejects immediately; missing permission denies

### Questions to Test Understanding

1. Why cache JWKS keys at all? Why not fetch every time?
   - (Answer: Performance; JWKS changes rarely; 1h TTL balances freshness and speed)

2. What happens if OIDC provider is down?
   - (Answer: Cached keys still work; only new requests with unknown key IDs fail)

3. How does RBAC know which permissions belong to which roles?
   - (Answer: Hardcoded mapping in RbacEngine; static, not configurable)

4. Can a user have multiple roles?
   - (Answer: Yes - JWT can have multiple roles; permissions are union of all roles)

5. Why is permission checking separate from role extraction?
   - (Answer: Separation of concerns; allows flexible permission model if needed)

---

## Diagram 4: Dependency Graph - All Modules

**Shows**: Which modules depend on which

```mermaid
graph TD
    main["main.rs<br/>(CLI Entry)"]
    
    policy["Policy Engine"]
    identity["Identity"]
    
    auth["Auth<br/>(JWT+RBAC)"]
    guardrails["Guardrails<br/>(ONNX)"]
    audit["Audit<br/>(Ed25519)"]
    sandbox["Sandbox<br/>(Firecracker)"]
    queue["Queue<br/>(NATS)"]
    cache["Cache<br/>(LRU+Vector)"]
    evals["Evals<br/>(Drift)"]
    proxy["Proxy<br/>(SSE)"]
    telemetry["Telemetry<br/>(OTLP)"]
    router["Router<br/>(Multi-model)"]
    api["API<br/>(gRPC)"]
    
    main -->|"Creates"| policy
    main -->|"Creates"| identity
    main -->|"Initializes all features"| auth
    main -->|"Initializes all features"| guardrails
    main -->|"Initializes all features"| audit
    main -->|"Initializes all features"| sandbox
    main -->|"Initializes all features"| queue
    main -->|"Initializes all features"| cache
    main -->|"Initializes all features"| evals
    main -->|"Initializes all features"| proxy
    main -->|"Initializes all features"| telemetry
    main -->|"Initializes all features"| router
    main -->|"Starts"| api
    
    api -->|"Uses"| policy
    api -->|"Uses"| auth
    api -->|"Uses"| guardrails
    api -->|"Logs to"| audit
    api -->|"Enqueues"| evals
    
    policy -->|"Loads config"| auth
    policy -->|"Loads config"| guardrails
    policy -->|"Loads config"| audit
    policy -->|"Loads config"| sandbox
    policy -->|"Loads config"| queue
    policy -->|"Loads config"| cache
    policy -->|"Loads config"| evals
    policy -->|"Loads config"| proxy
    policy -->|"Loads config"| telemetry
    
    auth -.->|"(independent)"| guardrails
    audit -.->|"(independent)"| queue
    cache -.->|"(independent)"| evals
    proxy -.->|"(independent)"| sandbox
    
    style main fill:#4A90E2,color:#fff
    style policy fill:#50C878,color:#fff
    style api fill:#FFD93D,color:#000
    style auth fill:#FF6B6B,color:#fff
    style audit fill:#95E1D3,color:#000
    style sandbox fill:#E8A87C,color:#000
    style queue fill:#C8A2E5,color:#000
```

### How to Read This Diagram

**Solid arrows**: Hard dependencies

**Dotted arrows**: No dependencies (independent modules)

**Blue boxes**: Core orchestration layer

**Feature boxes**: Optional modules that can be toggled

### Important Observations

1. **Layered Dependency**: main.rs → policy → features (not circular)
2. **Feature Independence**: Most modules don't depend on each other
3. **API Integration**: All flows go through API layer; APIs orchestrate features
4. **Explicit Initialization Order**: Matters for startup consistency
5. **No Circular Dependencies**: Clean DAG (Directed Acyclic Graph)

### Questions to Test Understanding

1. Could you remove the Auth module without breaking anything else?
   - (Answer: Yes - it's optional, disabled by default, no other modules use it)

2. What depends on Audit module?
   - (Answer: Only policy/API for logging; audit is write-only)

3. Why does policy engine reference all feature configs?
   - (Answer: To load them from secureai.toml; provides configuration abstraction)

4. If you wanted to use Evals without Queue, would it work?
   - (Answer: Yes - evals uses internal tokio channel, not NATS queue)

5. How would adding a new feature (e.g., Webhooks) change this diagram?
   - (Answer: New "webhooks" box; policy loads its config; may depend on some modules)

---

## Diagram 5: Request Lifecycle - Policy Evaluation

**Shows**: Most important flow through the system

```mermaid
sequenceDiagram
    participant Client
    participant GrpcAPI
    participant PolicyEngine
    participant Auth
    participant Guardrails
    participant Cache
    participant Sandbox
    participant Audit
    participant Evals as EvalEngine

    Client->>GrpcAPI: 1. EvaluatePolicy(tool, prompt, context)
    GrpcAPI->>Auth: 2. authenticate_request(JWT)
    Auth->>Auth: 3. Validate JWT signature
    Auth->>Auth: 4. Extract roles, build AuthContext
    Auth-->>GrpcAPI: 5. AuthContext or 401 error
    
    alt Auth Failed
        GrpcAPI-->>Client: 401 Unauthenticated
    else Auth Success
        GrpcAPI->>PolicyEngine: 6. evaluate_policy(context)
        PolicyEngine->>Guardrails: 7. check_prompt(prompt)
        Guardrails->>Guardrails: 8. Vectorize via ONNX
        Guardrails-->>PolicyEngine: 9. Deny/Permit
        
        alt Threat Detected
            PolicyEngine-->>GrpcAPI: 10. Denied (threat)
        else No Threat
            PolicyEngine->>Cache: 11. lookup(request_hash)
            Cache-->>PolicyEngine: 12. Hit/Miss
            
            alt Cache Hit
                PolicyEngine-->>GrpcAPI: 13. Return cached result
            else Cache Miss
                PolicyEngine->>Sandbox: 14. spawn_vm(prompt)
                Sandbox->>Sandbox: 15. Execute in MicroVM
                Sandbox-->>PolicyEngine: 16. Result
                PolicyEngine->>Cache: 17. store(hash, result)
            end
            
            PolicyEngine->>Audit: 18. log_action(execute)
            Audit-->>PolicyEngine: 19. Entry ID
            
            PolicyEngine->>Evals: 20. evaluate_async(prompt, response)
            Note over Evals: Fire-and-forget<br/>(doesn't block)
            
            PolicyEngine-->>GrpcAPI: 21. Success response
        end
    end
    
    GrpcAPI-->>Client: 22. EvaluatePolicyResponse
```

### How to Read This Diagram

**Vertical lines**: Participants (system components)

**Arrows**: Synchronous calls (wait for response)

**Dashed arrows**: Async/fire-and-forget calls

**Boxes**: Decision points (Diamond = if/else)

**Sequence**: Numbered steps show execution order

### Important Observations

1. **Fail-Fast Security**: Auth happens first (fail-secure)
2. **Defense in Depth**: Multiple checks (auth → guardrail → policy)
3. **Caching is Transparent**: Caller doesn't know if result is cached
4. **Async Evaluation**: Evals doesn't block response (fire-and-forget)
5. **Audit is Mandatory**: Every execution logged (unless audit disabled)

### Questions to Test Understanding

1. What happens if guardrails detects a threat?
   - (Answer: Short-circuit; deny immediately; don't execute sandbox)

2. Why is cache lookup before sandbox execution?
   - (Answer: Performance; avoid expensive VM spawn if result cached)

3. Can cache hit and guardrail bypass both happen?
   - (Answer: No - guardrail happens before cache check; if threat, never reaches cache)

4. What if audit fails to log?
   - (Answer: Depends on configuration; can be fire-and-forget or blocking)

5. Why is evals fire-and-forget instead of waiting?
   - (Answer: Evals is optional monitoring; shouldn't block user response)

---

## Diagram 6: Data Flow Diagram

**Shows**: How data moves through the system

```mermaid
graph TB
    subgraph Input["📥 Input"]
        UserPrompt["User Prompt"]
        UserContext["User Context<br/>(flags, metadata)"]
    end
    
    subgraph Processing["⚙️ Processing"]
        AuthFlow["Auth Flow<br/>(Extract JWT claims)"]
        CacheFlow["Cache Lookup<br/>(Tier 1 + 2)"]
        GuardrailFlow["Guardrail Check<br/>(Vectorize & match)"]
        SandboxFlow["Sandbox Execution<br/>(Run in MicroVM)"]
    end
    
    subgraph Enrichment["📊 Enrichment"]
        EvalFlow["Eval Metrics<br/>(toxicity, quality)"]
        DriftFlow["Drift Detection<br/>(3-sigma)"]
    end
    
    subgraph Persistence["💾 Persistence"]
        AuditLog["Audit Log<br/>(signed entry)"]
        CacheStore["Cache Store<br/>(result + embedding)"]
        MetricsStore["Metrics Store<br/>(sliding windows)"]
    end
    
    subgraph Output["📤 Output"]
        Response["API Response<br/>(result + metadata)"]
        Traces["Telemetry Traces<br/>(to OTLP collector)"]
    end
    
    UserPrompt -->|"Prompt text"| AuthFlow
    UserContext -->|"Tenant, flags"| AuthFlow
    
    AuthFlow -->|"Validated context<br/>(user_id, roles)"| CacheFlow
    CacheFlow -->|"Request hash"| CacheFlow
    CacheFlow -->|"Cache miss"| GuardrailFlow
    
    UserPrompt -->|"Prompt vector"| GuardrailFlow
    GuardrailFlow -->|"Threat detected?"| SandboxFlow
    
    SandboxFlow -->|"Execution result"| EvalFlow
    SandboxFlow -->|"Result"| CacheStore
    
    EvalFlow -->|"Metrics (toxicity,<br/>hallucination)"| MetricsStore
    MetricsStore -->|"Samples"| DriftFlow
    DriftFlow -->|"Anomaly detected?"| Output
    
    UserPrompt -->|"Action"| AuditLog
    SandboxFlow -->|"Execution details"| AuditLog
    AuditLog -->|"Signed entry"| Persistence
    
    SandboxFlow -->|"Result"| Response
    DriftFlow -->|"Alerts"| Response
    CacheFlow -->|"Hit/miss metadata"| Response
    
    Response -->|"gRPC response"| Output
    AuthFlow -->|"spans"| Traces
    CacheFlow -->|"spans"| Traces
    GuardrailFlow -->|"spans"| Traces
    SandboxFlow -->|"spans"| Traces
    Traces -->|"OTLP batches"| Output
    
    style Input fill:#50C878,color:#fff
    style Processing fill:#4A90E2,color:#fff
    style Enrichment fill:#FFD93D,color:#000
    style Persistence fill:#C8A2E5,color:#fff
    style Output fill:#FF6B6B,color:#fff
```

### How to Read This Diagram

**Green (Input)**: Data entering the system

**Blue (Processing)**: Main business logic transformations

**Yellow (Enrichment)**: Optional monitoring/analytics

**Purple (Persistence)**: Data being written to storage

**Red (Output)**: Data leaving the system

**Arrows**: Data transformations and flows

### Important Observations

1. **Multi-path Data Flow**: Same prompt flows through cache, guardrail, and sandbox
2. **Metadata Enrichment**: Evals adds metadata (anomaly alerts) to response
3. **Tracing is Pervasive**: Spans collected from every layer
4. **Audit is Separate Stream**: Doesn't interact with response path
5. **Cache Stores Two Forms**: Both result AND embedding (for semantic search)

### Questions to Test Understanding

1. Where does the user's prompt appear in this diagram?
   - (Answer: Input → GuardrailFlow → SandboxFlow; also embedded for semantic search)

2. Why does cache store both result AND embedding?
   - (Answer: Tier 1 uses result; Tier 2 uses embedding for semantic matching)

3. How does drift detection get its data?
   - (Answer: From EvalFlow metrics; compares against historical windows)

4. What happens to traces if telemetry is disabled?
   - (Answer: Spans still generated but batch processor is disabled; no export)

5. If sandbox execution fails, does data still flow to audit?
   - (Answer: Yes - failure is still an action worth auditing)

---

## Diagram 7: Authentication & Authorization Flow

**Shows**: JWT validation and permission checking

```mermaid
sequenceDiagram
    participant Client
    participant GrpcServer
    participant JwtValidator
    participant OIDCProvider
    participant JwksCache
    participant RbacEngine
    participant PolicyService

    Client->>GrpcServer: 1. gRPC call + Authorization header
    note over GrpcServer: Authorization: Bearer eyJhb...
    
    GrpcServer->>JwtValidator: 2. validate_token(jwt)
    
    JwtValidator->>JwtValidator: 3. Decode JWT header (extract kid)
    
    JwtValidator->>JwksCache: 4. get_jwks(issuer)
    
    alt Cache Hit
        JwksCache-->>JwtValidator: 5. Return cached JWKS
    else Cache Miss
        JwksCache->>OIDCProvider: 5. GET /.well-known/openid-configuration
        OIDCProvider-->>JwksCache: 6. {jwks_uri, ...}
        JwksCache->>OIDCProvider: 7. GET /jwks.json
        OIDCProvider-->>JwksCache: 8. {keys: [...]}
        JwksCache->>JwksCache: 9. Cache for 1 hour
        JwksCache-->>JwtValidator: 10. Return JWKS
    end
    
    JwtValidator->>JwtValidator: 11. Find key by kid in JWKS
    JwtValidator->>JwtValidator: 12. RS256 signature verify
    JwtValidator->>JwtValidator: 13. Validate exp, aud, iss claims
    JwtValidator->>JwtValidator: 14. Extract claims (sub, roles, tenant_id)
    
    JwtValidator-->>GrpcServer: 15. JwtClaims or 401 error
    
    alt JWT Invalid
        GrpcServer-->>Client: 16. 401 Unauthenticated
    else JWT Valid
        GrpcServer->>RbacEngine: 17. roles_from_claims(claims.roles)
        RbacEngine-->>GrpcServer: 18. [Role::Admin, Role::AuditReader]
        
        GrpcServer->>RbacEngine: 19. check_permission(roles, tools:execute)
        RbacEngine-->>GrpcServer: 20. Ok(()) or Err(403)
        
        alt Permission Denied
            GrpcServer-->>Client: 21. 403 PermissionDenied
        else Permission Granted
            GrpcServer->>PolicyService: 22. Process request (auth passed)
            PolicyService-->>Client: 23. Response
        end
    end
```

### How to Read This Diagram

**Numbered steps**: Sequential operations

**Decision diamonds (alt blocks)**: Branching logic

**Dotted return arrows**: Responses coming back

**Error paths**: Shown as early returns (401, 403)

### Important Observations

1. **JWKS Caching Matters**: First request to new issuer fetches keys; subsequent requests use cache
2. **Fail-Secure**: Invalid JWT → 401 immediately; missing permission → 403 immediately
3. **Three Validation Levels**: 
   - Cryptographic (signature)
   - Temporal (expiration)
   - Semantic (aud, iss claims)
4. **Immutable Permission Mapping**: Roles hardcoded, not fetched from external source
5. **Multi-tenant by Default**: tenant_id from JWT claim enables isolation

### Questions to Test Understanding

1. What if the OIDC provider's JWKS changes?
   - (Answer: Cache TTL (1h) means old keys used until refresh; new keys picked up on cache miss)

2. Can someone use an expired token if signature is valid?
   - (Answer: No - expiration validated after signature; exp must be in future)

3. How does SecureAI know which permissions a role has?
   - (Answer: Hardcoded mapping in RbacEngine; not fetched from OIDC)

4. What if a user has no roles in their JWT?
   - (Answer: Empty roles list → Guest role → No permissions → 403 on execute)

5. Why validate exp, aud, iss after signature verification?
   - (Answer: Signature proves token came from issuer; claims validate it's for this app and not expired)

---

## Diagram 8: Data Persistence Model

**Shows**: What data is stored and where

```mermaid
erDiagram
    AUDIT_LEDGER ||--o{ AUDIT_ENTRY : contains
    POLICY_CONFIG ||--o{ FEATURE_CONFIG : has
    CACHE_TIER1 ||--o{ CACHED_RESPONSE : stores
    CACHE_TIER2 ||--o{ CACHED_EMBEDDING : stores
    JOB_QUEUE ||--o{ JOB : contains
    METRIC_WINDOW ||--o{ METRIC_SAMPLE : holds
    
    AUDIT_ENTRY {
        u64 id
        u64 timestamp
        string action
        string subject
        json details
        string hash
        string signature
    }
    
    CACHED_RESPONSE {
        string key_sha256
        string value_json
        u64 created_at
        u64 ttl_secs
    }
    
    CACHED_EMBEDDING {
        string key_hash
        vector embedding
        f32 similarity_threshold
        u64 created_at
    }
    
    JOB {
        string id
        string tool_name
        json params
        string state
        u64 created_at
        u32 retries
    }
    
    METRIC_SAMPLE {
        f32 value
        u64 timestamp
        string metric_type
    }
    
    POLICY_CONFIG {
        string allowed_paths
        bool network_access
        u32 max_memory_mb
    }
    
    FEATURE_CONFIG {
        bool enabled
        json settings
    }
```

### How to Read This Diagram

**Entities** (boxes): Data structures that are persisted

**Relationships** (lines): How entities relate to each other

**Cardinality** (||, ||--o): One-to-one, one-to-many

**Attributes**: Fields stored for each entity

### Important Observations

1. **Append-Only Ledger**: Audit entries never updated/deleted; immutable chain
2. **TTL-Based Cache**: Responses cached with time-to-live; auto-expired
3. **Dual Indexing**: Cache keyed by hash (Tier 1) and embedding (Tier 2)
4. **Job State Machine**: Jobs have explicit state progression
5. **Metric Windows**: Samples stored in sliding windows for drift detection
6. **No SQL Database**: All persistence is either files, in-memory, or queues

### Important Caveat

**UNKNOWN**: Secondary indices, physical storage format, cache eviction policies (inferred from code but not explicitly documented)

### Questions to Test Understanding

1. How are cache entries evicted when memory is full?
   - (Answer: LRU policy; oldest entries removed first; controlled by cache capacity config)

2. Can you query audit entries by action or subject?
   - (Answer: Not efficiently - it's append-only; would require scan; no indices)

3. What happens to job data after completion?
   - (Answer: Stays in NATS JetStream; subject to retention policy (default 30 days))

4. How long are metric samples kept?
   - (Answer: In sliding windows (1h + 24h); old samples fall out of window)

5. Why is embedding stored separately from response?
   - (Answer: Tier 1 cache doesn't need it; Tier 2 needs it for similarity search)

---

## Diagram 9: Sequence Diagrams - Five Important Operations

### 9a: Task Execution in Sandbox

```mermaid
sequenceDiagram
    participant API
    participant PolicyEngine
    participant Sandbox
    participant Firecracker
    participant Audit

    API->>PolicyEngine: 1. execute_task(prompt)
    PolicyEngine->>Sandbox: 2. spawn_vm(kernel, rootfs)
    Sandbox->>Firecracker: 3. Launch microVM
    Firecracker-->>Sandbox: 4. VM ID + status socket
    
    Sandbox->>Sandbox: 5. Apply Landlock policy
    Sandbox->>Sandbox: 6. Apply seccomp filters
    Sandbox->>Sandbox: 7. Apply cgroup limits
    
    Sandbox->>Firecracker: 8. Execute command in VM
    Firecracker->>Firecracker: 9. Run task
    Firecracker-->>Sandbox: 10. stdout/stderr/exit_code
    
    Sandbox->>Sandbox: 11. Collect resource usage
    Sandbox->>Sandbox: 12. Teardown VM
    Sandbox-->>PolicyEngine: 13. Result + metrics
    
    PolicyEngine->>Audit: 14. log_sandbox_execution(result)
    Audit-->>PolicyEngine: 15. Entry ID
    
    PolicyEngine-->>API: 16. Return result to client
```

### 9b: Cache Lookup (Two-Tier)

```mermaid
sequenceDiagram
    participant Caller
    participant CacheManager
    participant Tier1Cache
    participant Tier2Cache
    participant Compute

    Caller->>CacheManager: 1. get_or_compute(key, compute_fn)
    
    CacheManager->>Tier1Cache: 2. lookup(sha256_hash)
    Tier1Cache-->>CacheManager: 3. Hit OR Miss
    
    alt Tier1 Hit
        CacheManager-->>Caller: 4. Return cached result
    else Tier1 Miss
        CacheManager->>Tier2Cache: 5. compute_embedding(key)
        Tier2Cache->>Tier2Cache: 6. Vectorize using ONNX
        Tier2Cache-->>CacheManager: 7. Embedding
        
        CacheManager->>Tier2Cache: 8. search_similar(embedding)
        Tier2Cache-->>CacheManager: 9. Similar entry OR Miss
        
        alt Tier2 Hit
            CacheManager-->>Caller: 10. Return similar result
        else Tier2 Miss
            CacheManager->>Compute: 11. compute(key)
            Compute-->>CacheManager: 12. Result
            CacheManager->>Tier1Cache: 13. store(hash, result)
            CacheManager->>Tier2Cache: 14. store(embedding, result)
            CacheManager-->>Caller: 15. Return result
        end
    end
```

### 9c: Job Queue Processing

```mermaid
sequenceDiagram
    participant Producer
    participant NatsJetStream
    participant Consumer
    participant Worker
    participant Executor

    Producer->>NatsJetStream: 1. Enqueue job
    NatsJetStream-->>Producer: 2. Job ID
    
    Consumer->>NatsJetStream: 3. Pull next job (non-blocking)
    NatsJetStream-->>Consumer: 4. Job info (lease_token)
    
    Consumer->>Worker: 5. Job + lease_token
    Worker->>Executor: 6. Execute job
    
    par Heartbeat
        Worker->>NatsJetStream: 7. Extend lease (every 10s)
        NatsJetStream-->>Worker: 8. ACK
    and Execution
        Executor->>Executor: 9. Long-running task
        Executor-->>Worker: 10. Result
    end
    
    Worker->>NatsJetStream: 11. ACK job completion
    NatsJetStream-->>Worker: 12. Job marked complete
    
    alt Timeout (no heartbeat)
        NatsJetStream->>NatsJetStream: 13. Lease expired
        NatsJetStream->>Consumer: 14. Job requeued automatically
    end
```

### 9d: Drift Detection (3-Sigma Anomaly)

```mermaid
sequenceDiagram
    participant EvalEngine
    participant Sampler
    participant Evaluator
    participant MetricWindow
    participant DriftDetector
    participant AlertSystem

    EvalEngine->>Sampler: 1. should_evaluate(request, is_flagged)
    Sampler->>Sampler: 2. Random decision (baseline 10%, boosted 100%)
    Sampler-->>EvalEngine: 3. Evaluate YES/NO
    
    alt Should Evaluate
        EvalEngine->>Evaluator: 4. evaluate_async(prompt, response)
        Evaluator->>Evaluator: 5. Compute metrics (toxicity, hallucination)
        Evaluator-->>EvalEngine: 6. Metrics
        
        EvalEngine->>MetricWindow: 7. add_sample(value)
        MetricWindow->>MetricWindow: 8. Update short window (1h) + long window (24h)
        MetricWindow-->>EvalEngine: 9. Sample added
        
        EvalEngine->>DriftDetector: 10. detect_anomalies()
        DriftDetector->>DriftDetector: 11. Compute baseline (long window)
        DriftDetector->>DriftDetector: 12. Compute current (short window)
        DriftDetector->>DriftDetector: 13. z_score = (current - baseline) / stddev
        DriftDetector-->>EvalEngine: 14. z_score
        
        alt z_score > 3.0
            DriftDetector->>AlertSystem: 15. Alert: anomaly detected
            AlertSystem-->>DriftDetector: 16. Alert queued
        end
    else Should NOT Evaluate
        EvalEngine-->>EvalEngine: 17. Skip (no evaluation overhead)
    end
```

### 9e: RBAC Permission Checking

```mermaid
sequenceDiagram
    participant Request
    participant AuthMiddleware
    participant JwtValidator
    participant RbacEngine
    participant Handler

    Request->>AuthMiddleware: 1. gRPC call
    AuthMiddleware->>JwtValidator: 2. validate_token(jwt)
    JwtValidator-->>AuthMiddleware: 3. JwtClaims
    
    AuthMiddleware->>RbacEngine: 4. roles_from_claims(claims.roles)
    RbacEngine->>RbacEngine: 5. Map "admin" → Role::Admin, etc.
    RbacEngine-->>AuthMiddleware: 6. [Role::Admin, Role::AuditReader]
    
    AuthMiddleware->>RbacEngine: 7. permissions_from_roles(roles)
    RbacEngine->>RbacEngine: 8. Lookup permission set for each role
    RbacEngine->>RbacEngine: 9. Union all permissions
    RbacEngine-->>AuthMiddleware: 10. HashSet<Permission>
    
    AuthMiddleware->>AuthMiddleware: 11. Build AuthContext
    
    alt Execute requires tools:execute
        AuthMiddleware->>AuthMiddleware: 12. Check: tools:execute in permissions
        AuthMiddleware-->>Handler: 13. AuthContext + check passed
        Handler->>Handler: 14. Process request
    else Missing Permission
        AuthMiddleware-->>Request: 15. 403 PermissionDenied
    end
```

### How to Read These Diagrams

**Sequence diagrams** show time flowing top-to-bottom

**Arrows** = messages/calls between participants

**Boxes** = parallel operations (par block)

**alt/else blocks** = conditional paths

**Numbered steps** = execution order

### Important Observations

1. **Sandbox is Resource-Heavy**: VM spawn takes 500-1000ms per task
2. **Cache is Transparent**: Caller doesn't know if result is cached
3. **Queue Uses Leases**: Heartbeats keep lease alive; timeout auto-requeues
4. **Drift Detection is Async**: Doesn't block request handling
5. **Permission Checking is Eager**: Resolved once at request start; not per-resource

### Questions to Test Understanding

1. What happens if a VM task takes longer than 30 seconds?
   - (Answer: In current design, depends on implementation; likely timeout or heartbeat renewal)

2. Why does Tier 2 cache compute embedding instead of storing it initially?
   - (Answer: Only needed if Tier 1 misses; saves computation for requests that hit Tier 1)

3. Can a job be processed by multiple workers?
   - (Answer: No - NATS lease ensures only one worker; lease prevents duplicate processing)

4. If z_score is 2.5 (below 3.0), what happens?
   - (Answer: No alert generated; within normal variance)

5. Why check permissions once at request start instead of per-resource?
   - (Answer: Simpler; SecurityContext attached to request; no per-resource checks)

---

## Diagram 10: Deployment Architecture

**Shows**: How system is deployed, containerized, and scaled

```mermaid
graph TB
    subgraph Users["👥 Users"]
        CLIUser["CLI User<br/>(Local)"]
        APIClient["API Client<br/>(gRPC)"]
    end
    
    subgraph LoadBalancing["⚖️ Load Balancing"]
        LB["Load Balancer<br/>(K8s Service)"]
    end
    
    subgraph K8sCluster["☸️ Kubernetes Cluster"]
        Pod1["SecureAI Pod 1<br/>(Replica 1)"]
        Pod2["SecureAI Pod 2<br/>(Replica 2)"]
        Pod3["SecureAI Pod 3<br/>(Replica 3)"]
    end
    
    subgraph PersistentServices["📦 Persistent Services"]
        NATS["NATS JetStream<br/>StatefulSet"]
        OtelCollector["OpenTelemetry<br/>Collector"]
        Storage["Persistent Storage<br/>(Audit Logs)"]
    end
    
    subgraph ExternalDeps["🌐 External Dependencies"]
        OIDC["OIDC Provider<br/>(Okta/Auth0/Azure AD)"]
        Firecracker["Firecracker<br/>(Host OS)"]
        OnnxRuntime["ONNX Runtime<br/>(Built-in)"]
        OtelBackend["Monitoring Backend<br/>(Datadog/Jaeger)"]
    end
    
    subgraph Networking["🔗 Networking"]
        Ingress["Ingress Controller"]
        DNS["DNS"]
    end
    
    CLIUser -->|"Local binary"| Pod1
    APIClient -->|"gRPC"| DNS
    DNS -->|"Resolves to"| LB
    LB -->|"Distributes"| Pod1
    LB -->|"Distributes"| Pod2
    LB -->|"Distributes"| Pod3
    
    Ingress -->|"Routes HTTP/gRPC"| LB
    
    Pod1 -->|"Connects to"| NATS
    Pod2 -->|"Connects to"| NATS
    Pod3 -->|"Connects to"| NATS
    
    Pod1 -->|"Sends traces"| OtelCollector
    Pod2 -->|"Sends traces"| OtelCollector
    Pod3 -->|"Sends traces"| OtelCollector
    
    Pod1 -->|"Writes audit"| Storage
    Pod2 -->|"Writes audit"| Storage
    Pod3 -->|"Writes audit"| Storage
    
    Pod1 -->|"Validates JWT"| OIDC
    Pod2 -->|"Validates JWT"| OIDC
    Pod3 -->|"Validates JWT"| OIDC
    
    Pod1 -->|"Spawns VMs"| Firecracker
    Pod2 -->|"Spawns VMs"| Firecracker
    Pod3 -->|"Spawns VMs"| Firecracker
    
    OtelCollector -->|"Exports"| OtelBackend
    
    style K8sCluster fill:#4A90E2,stroke:#2E5C8A,stroke-width:3px,color:#fff
    style ExternalDeps fill:#FF6B6B,stroke:#CC5555,stroke-width:2px,color:#fff
    style PersistentServices fill:#50C878,stroke:#2E7D4E,stroke-width:2px,color:#fff
    style Users fill:#FFD93D,stroke:#CC9030,stroke-width:2px,color:#000
```

### How to Read This Diagram

**Blue box (K8s Cluster)**: Main deployment target

**Red boxes (External)**: Out-of-cluster dependencies

**Green boxes (Persistent)**: Long-lived services

**Yellow boxes (Users)**: Request sources

**Arrows**: Data flows and connections

### Important Observations

1. **Horizontal Scalability**: Multiple pods behind load balancer
2. **Stateless Pods**: Each pod can be replaced/restarted
3. **Shared State**: NATS, Storage, OIDC are shared resources
4. **Pod Affinity Possible**: Pods could be scheduled by zone/region
5. **Firecracker is Local**: Runs on each node (host OS requirement)

### Important Caveats

**UNKNOWN**:
- Actual replica count (assumed 3 for HA)
- Storage solution (could be NFS, object store, etc.)
- Node resource specs
- Pod resource requests/limits
- Network policies
- Ingress rules

### Questions to Test Understanding

1. Why are there 3 replicas instead of 1?
   - (Answer: High availability; if one pod fails, traffic routes to others)

2. Can you run SecureAI without Kubernetes?
   - (Answer: Yes - docker-compose or systemd; K8s is recommended for production)

3. What if NATS becomes unavailable?
   - (Answer: Queue feature fails; system degrades; enqueued jobs pending)

4. Why can't each pod have its own Firecracker?
   - (Answer: It can - but Firecracker resource-heavy; typically shared at node level)

5. How do pods share audit logs in Storage?
   - (Answer: Concurrent writes to shared file or distributed storage)

---

## Summary: Understanding Architecture Diagrams

### Reading Strategy

1. **Start with System Context**: Understand boundaries and external systems
2. **Move to Containers**: See major deployable units
3. **Dive into Components**: Understand internal structure of key modules
4. **Trace Flows**: Follow data and control flow through system
5. **Study Sequences**: Understand timing and ordering of operations

### Key Takeaways

- **Modular but Integrated**: Independent modules orchestrated by PolicyEngine
- **Defense in Depth**: Multiple security checkpoints
- **Optional Features**: Most modules can be disabled via config
- **Async Where Possible**: Evals and telemetry don't block responses
- **Scalable Design**: Stateless pods, shared services, no single points of failure

### Common Patterns Visible

1. **Layered Architecture**: API → Auth → Policy → Execution → Persistence
2. **Repository Pattern**: PolicyStore, CacheManager abstract data access
3. **Fire-and-Forget**: Evals, telemetry don't block
4. **State Machine**: Jobs have explicit state progression
5. **Fail-Secure**: Default to deny, not allow

---

## Diagram Reference Guide

| Diagram | Use When | Key Insight |
|---------|----------|------------|
| System Context | Explaining to stakeholders | External dependencies matter |
| Container | Deploying or scaling | Monolithic but modular |
| Component | Modifying a module | Most modules independent |
| Dependency Graph | Adding a feature | Clean DAG, no cycles |
| Request Lifecycle | Debugging a request | Multi-layer security checks |
| Data Flow | Understanding persistence | No traditional database |
| Auth/Authz | Security audit | JWT + RBAC + tenant isolation |
| Persistence Model | Data modeling | Append-only + in-memory caches |
| Sequence Diagrams | Detailed timing | Async where possible |
| Deployment | Production setup | K8s recommended |

---

[← Previous: Repository Map](01_REPOSITORY_MAP.md) | [Next: System Mental Model →](02_SYSTEM_MENTAL_MODEL.md)
