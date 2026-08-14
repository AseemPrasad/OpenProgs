# Architectural Decisions Deep Analysis

**Senior architect-level reasoning behind SecureAI's design decisions**

This document answers WHY for every significant architectural choice in SecureAI, by examining actual code evidence and evaluating realistic alternatives.

---

## Structure of Each Analysis

```
WHY THIS?
  ├─ What does it do
  ├─ Where is it implemented
  ├─ Problem it solves
  ├─ Engineering principle
  └─ Evidence in repository

WHY NOT THAT?
  ├─ Alternative 1: [How it works]
  │   ├─ Advantages
  │   ├─ Disadvantages
  │   ├─ Complexity
  │   ├─ Performance
  │   ├─ Scalability
  │   ├─ Maintainability
  │   ├─ Testability
  │   └─ Operational impact
  ├─ Alternative 2...
  └─ [Repeat for each realistic alternative]

TRADEOFF
  └─ Gains vs Sacrifices

FAILURE POINT
  └─ When this decision becomes problematic

CHANGE CONDITION
  └─ Future requirement to replace it

SCALE CONDITION
  └─ Scale at which architecture breaks

LEARNING QUESTION
  └─ Tests genuine understanding
```

---

# DECISION 1: Firecracker MicroVMs for Task Isolation

## WHY THIS?

**What it does**:
Every task executes in a separate Firecracker microVM with kernel-level isolation, mandatory resource limits (cgroups), syscall filtering (seccomp), and filesystem restrictions (Landlock LSM).

**Where implemented**:
- `src/sandbox/mod.rs` - Main SandboxManager
- `src/sandbox/firecracker.rs` - Firecracker process spawning
- `src/sandbox/seccomp.rs` - Syscall filtering rules
- `src/sandbox/landlock.rs` - Filesystem access policy
- `src/sandbox/cgroups.rs` - Resource limits (CPU, memory, processes)
- `main.rs:62-204` - CLI integration

**Problems solved**:
1. **Escape prevention**: Untrusted task cannot access host system
2. **Resource DOS**: Task cannot consume unlimited CPU/memory
3. **File system access**: Task cannot read files outside allowed paths
4. **Privilege escalation**: Task cannot make privileged syscalls
5. **Kernel panic**: Malicious syscall cannot crash host kernel

**Engineering principle**:
- **Fail-secure by default**: Deny all, allow only explicitly safe syscalls
- **Defense in depth**: Three independent isolation layers (kernel boundary + LSM + seccomp)
- **Mandatory safety**: Cannot be disabled, no opt-out

**Evidence in repository**:
```rust
// src/sandbox/mod.rs
pub struct SandboxManager {
    firecracker_binary: PathBuf,
    kernel_image: PathBuf,
    rootfs: PathBuf,
}

impl SandboxManager {
    pub async fn spawn_vm(&self, config: &IsolationPolicy) -> Result<VmHandle> {
        // 1. Spawn Firecracker process
        // 2. Apply Landlock policy (FS access)
        // 3. Apply seccomp filter (syscalls)
        // 4. Apply cgroup limits (resources)
        // Result: Completely isolated VM
    }
}
```

```rust
// src/sandbox/cgroups.rs - MANDATORY limits
pub struct CgroupLimits {
    pub memory_limit_mb: u64,    // Cannot be unlimited
    pub cpu_quota: u64,          // Cannot be unlimited
    pub max_processes: u64 = 100, // Hardcoded minimum
}
```

---

## WHY NOT THAT?

### Alternative 1: Docker Containers

**How it works**:
- Container image with task runtime
- Docker daemon manages container lifecycle
- seccomp + AppArmor/SELinux for isolation

**Advantages**:
- Faster startup (100-200ms vs 500-1000ms for MicroVM)
- Smaller memory footprint (50-100MB vs 100+MB)
- Simpler deployment (single binary)
- Richer ecosystem (Docker Hub, registries)
- Easier debugging (docker exec, logs)

**Disadvantages**:
- **Shared kernel**: All containers share host kernel → kernel exploit = all containers compromised
- **CVE history**: Container escapes regularly discovered (CVE-2021-30465, CVE-2021-32402, CVE-2021-34558)
- **Privilege escalation easier**: Some capabilities can be dropped but not all
- **Namespace vulnerabilities**: Namespace isolation weaker than VM boundary
- **Resource accounting weaker**: cgroups v1 has bypass vectors

**Complexity**:
- Docker daemon required (additional service to manage)
- Image versioning (what versions to keep)
- Registry infrastructure (where to store images)

**Performance**:
- **Startup**: ~100-200ms (faster)
- **Memory/task**: ~50-100MB (lower)
- **Runtime overhead**: Lower than MicroVM

**Scalability**:
- Can run more concurrent tasks per host (better resource utilization)
- Docker daemon becomes bottleneck at scale (container orchestration needed)

**Maintainability**:
- Docker ecosystem well-known
- But adds dependency on Docker daemon stability
- Container images need updates/patches

**Testability**:
- Easier local testing (docker run, inspect)
- Logs more accessible

**Operational consequences**:
- Need container registry
- Image garbage collection
- Container network management
- Persistent volume management

**Why NOT this**:
- Security is paramount for SecureAI (executive SLA)
- Kernel exploit means complete compromise
- Container escape CVEs happen regularly
- MVP prioritizes security over performance
- Evidence: `src/sandbox/mod.rs` comment: "Kernel isolation is non-negotiable"

---

### Alternative 2: Native Process + seccomp Only

**How it works**:
- Task runs as native Linux process
- Only seccomp filtering applied
- No VM overhead

**Advantages**:
- Maximum performance (no VM overhead)
- Minimal memory footprint
- Fastest startup (milliseconds)
- Simple to implement
- Easy to debug

**Disadvantages**:
- **No kernel isolation**: Kernel exploit = entire host compromise
- **Weaker namespace isolation**: Can be escaped with right syscalls
- **Shared memory space**: Potential information leakage
- **No resource hard limits**: Only soft cgroup limits (can be bypassed)
- **Single point of failure**: Task bug can crash host

**Complexity**:
- Simpler implementation
- But seccomp rules very complex (1000+ lines)

**Performance**:
- **Fastest**: No VM startup overhead
- **Lowest memory**: Only process memory
- **Runtime**: Direct execution

**Scalability**:
- Linear scalability (each task is light process)
- But no isolation between tasks

**Maintainability**:
- Smaller attack surface
- But harder to verify isolation properties

**Testability**:
- Trivial to test locally
- But harder to verify exploit prevents escapes

**Operational consequences**:
- Single host per workload (no multi-tenancy)
- Need separate machines for different trust domains
- Kernel patching critical (one exploit = all tasks lost)

**Why NOT this**:
- Kernel exploit is catastrophic failure
- No defense-in-depth
- Evidence: `src/sandbox/seccomp.rs` comment: "seccomp is necessary but not sufficient"

---

### Alternative 3: QEMU Full VMs

**How it works**:
- Full QEMU virtual machine per task
- Complete hardware simulation
- Independent kernel per VM

**Advantages**:
- Maximum isolation (complete hardware boundary)
- Can run different OS (Windows, Linux, etc.)
- Strong proven isolation model
- Hardware-level security

**Disadvantages**:
- **Much slower**: 2-5 seconds startup (vs 500ms MicroVM)
- **High memory**: 512MB+ per VM (vs 100MB MicroVM)
- **More complex**: QEMU has larger attack surface
- **Overkill for use case**: Don't need full hardware simulation
- **Licensing issues**: Some QEMU components GPL

**Complexity**:
- QEMU is complex (500k+ lines)
- Hypervisor management more complex
- Device emulation overhead

**Performance**:
- Very slow startup
- Memory intensive
- Acceptable for long-running tasks, but not for interactive prompts

**Scalability**:
- Low density (few VMs per host)
- Requires larger infrastructure

**Maintainability**:
- QEMU mature but complex
- Large surface area for bugs

**Testability**:
- Can test different OS behavior
- Slower tests

**Operational consequences**:
- High infrastructure cost
- Need resource management (overcommit impossible)

**Why NOT this**:
- Overkill for process isolation
- Firecracker designed specifically for this use case
- Performance unacceptable for interactive workload
- Evidence: Firecracker is AWS Lambda's sandbox (proven choice)

---

## TRADEOFF

**Gains**:
- Kernel-level isolation (exploit ≠ total compromise)
- Defense-in-depth (3 layers of isolation)
- Resource hard limits (cannot exceed)
- Production-proven (AWS Lambda uses Firecracker)
- Clear failure domain (failed task doesn't affect host)

**Sacrifices**:
- Performance (500-1000ms startup overhead per task)
- Memory efficiency (100+MB per task)
- Complexity (Firecracker binary dependency)
- Deployment (need vmlinux + rootfs)
- Debugging (no direct process access)

**Net result**: Slower and heavier, but fundamentally more secure. Acceptable tradeoff for MVP.

---

## FAILURE POINT

This decision becomes problematic when:

1. **Firecracker exploits discovered**: If kernel escape found in Firecracker, affects all tasks
2. **Resource exhaustion**: If host runs out of memory for new VMs, new requests denied
3. **Startup latency SLA**: If clients need <100ms response time, 500ms VM overhead breaks promise
4. **Multi-tenant concerns**: If isolation between VMs discovered to be weak (unlikely but possible)
5. **Licensing/compliance**: If Firecracker license changes incompatibly

---

## CHANGE CONDITION

Would justify replacing this approach:

1. **Performance requirement changes**: If SLA drops to <50ms (impossible with current model)
2. **Workload changes**: If tasks become long-running (>5 minutes), VM cost becomes acceptable
3. **Stronger isolation requirement**: If kernel exploits become regular (might switch to QEMU)
4. **Cost constraint**: If infrastructure cost becomes prohibitive (might consider containers)
5. **Operability requirement**: If debuggability becomes critical (might add container alternative)

---

## SCALE CONDITION

Architecture becomes problematic at:

**Per-host scale**:
- Host with 64 GB RAM: ~500-600 concurrent VMs (100MB each) = ~500 concurrent requests
- Beyond this: Need to scale horizontally (add hosts)

**Cluster scale**:
- At 1000+ hosts: Firecracker version management becomes difficult
- At 10,000+ hosts: Kernel update coordination required
- At 100,000+ hosts: Firecracker exploit patch SLA becomes critical

**Request scale**:
- At 1000 req/sec: VM startup becomes bottleneck (need larger VM pool or faster startup)
- At 10,000 req/sec: Need VM pooling + pre-warming strategy
- At 100,000 req/sec: Would need container alternative or stateless design

**Latency scale**:
- Current: No latency SLA (batch processing assumed)
- If SLA becomes <1s: VM startup remains acceptable
- If SLA becomes <100ms: This architecture breaks (need containers or pooling)

---

## LEARNING QUESTION

**Question**: Given that Firecracker VMs provide kernel-level isolation but with 500ms startup overhead, and Docker containers are faster but share the kernel, what configuration would you choose for:

a) A system running trusted internal tools with <50ms latency requirement?
b) A system running user-submitted code with no latency requirement but high security requirement?
c) A system running both trusted and untrusted code with mixed requirements?

**Answer should demonstrate understanding of**:
- Why kernel isolation matters (type of exploits it prevents)
- Why startup time matters (request latency SLA)
- How to mix approaches (not binary choice)
- Operational tradeoffs (complexity of managing both)

---

# DECISION 2: Opt-In Features Architecture

## WHY THIS?

**What it does**:
SecureAI has 10 major features (sandbox, audit, auth, guardrails, queue, cache, evals, proxy, grpc, telemetry). Each can be independently enabled/disabled via configuration file. No feature depends on any other (except through explicit config).

**Where implemented**:
- `src/policy/mod.rs:IsolationPolicy` - Feature flags (one line each)
- `src/policy/config.rs` - Config loading
- `src/main.rs:100-140` - Subsystem initialization (conditional)
- `secureai.toml` - Configuration file

**Problems solved**:
1. **Modularity**: Organizations use only features they need
2. **Complexity**: Simpler systems for simple use cases
3. **Upgrade path**: Add features without breaking existing deployments
4. **Testing**: Test features independently
5. **Operational**: Disable problematic features without recompile

**Engineering principle**:
- **Non-breaking evolution**: Add features without breaking existing code
- **Composition over inheritance**: Features compose via config, not inheritance
- **Principle of least privilege**: Only enable what's needed

**Evidence in repository**:
```rust
// src/policy/mod.rs
pub struct IsolationPolicy {
    // Core (always on)
    pub model: String,
    
    // Optional features
    #[serde(default)]
    pub enable_sandbox: bool,
    
    #[serde(default)]
    pub enable_audit: bool,
    
    #[serde(default = "default_enable_auth")]
    pub enable_auth: bool,
    
    #[serde(default)]
    pub enable_guardrails: bool,
    
    // ... more features
}

// src/main.rs
if config.enable_sandbox {
    let sandbox_mgr = SandboxManager::new()?;
    // Initialize sandbox subsystem
}

if config.enable_audit {
    let audit = AuditLedger::new()?;
    // Initialize audit subsystem
}
```

```toml
# secureai.toml
enable_sandbox = true       # Can disable for local testing
enable_audit = true         # Optional for compliance
enable_auth = false         # Optional for internal systems
enable_guardrails = false   # Optional if policy-trusted tasks
```

---

## WHY NOT THAT?

### Alternative 1: All Features Always Enabled

**How it works**:
- Compile all features into binary
- No configuration for feature toggles
- Simplified deployment

**Advantages**:
- Simpler codebase (fewer conditional paths)
- Better tested (fewer combinations)
- Predictable behavior (everyone same config)
- No configuration mistakes

**Disadvantages**:
- **Unnecessary overhead**: Systems don't need all features pay cost
- **Larger binary**: Every feature adds code
- **Breaking changes**: Can't disable features if they break
- **Operational inflexibility**: Can't selectively disable broken feature
- **Cognitive load**: Every operator needs to understand all features

**Complexity**:
- Simpler codebase (no feature flags)
- But everyone pays cost of all features

**Performance**:
- Unnecessary initialization overhead
- Larger memory footprint (all subsystems loaded)
- Slower startup (all subsystems initialize)

**Scalability**:
- Same resource usage regardless of actual need
- Wasteful at scale

**Maintainability**:
- Easier (no feature interactions to test)
- Harder (must update all features together)

**Testability**:
- Fewer code paths to test
- But less realistic (tests include all features)

**Operational consequences**:
- No way to disable problematic feature without rebuild
- If audit system buggy, must disable binary (can't just turn it off)
- All systems pay the cost

**Why NOT this**:
- Different organizations have different needs
- Some run on constrained infrastructure
- Evidence: Config file structure expects feature flags

---

### Alternative 2: Build-Time Features (Cargo Features)

**How it works**:
- Compile binary with selected features enabled/disabled
- `cargo build --features "sandbox,auth"` compiles without other features
- Features conditionally compiled via `#[cfg(feature = "...")]`

**Advantages**:
- **Zero runtime overhead**: Unused code not in binary
- **Smallest binary size**: Only compiled features included
- **Compile-time safety**: Invalid feature combinations detected at compile
- **Rust ecosystem standard**: Familiar pattern
- **Type system helps**: Compiler enforces feature requirements

**Disadvantages**:
- **No runtime flexibility**: Can't disable feature without rebuild
- **Separate binaries**: Each configuration needs separate build
- **Operational complexity**: Must manage multiple binary versions
- **Deployment complexity**: Which binary for which environment?
- **M × N problem**: If 10 features, 2^10 = 1024 possible combinations
- **Testing explosion**: Must test all combinations

**Complexity**:
- Increases build/deployment complexity
- Requires careful matrix management

**Performance**:
- Best (no unused code)
- But complicated to achieve this benefit

**Scalability**:
- Scales to many features (combinatorial explosion)

**Maintainability**:
- Harder (feature interactions compile-time checked but complex)
- Binary management overhead

**Testability**:
- Must test many combinations
- CI matrix becomes large

**Operational consequences**:
- Multiple binaries to manage
- Hard to disable feature after deployment

**Why NOT this**:
- Operational complexity outweighs binary size benefit
- Runtime config is more flexible for quick fixes
- Evidence: Runtime config chosen (not compile-time features)

---

### Alternative 3: No Configuration (Hardcoded Minimal)

**How it works**:
- Core functionality only (maybe just sandbox)
- Everything else must be explicitly requested
- No config file

**Advantages**:
- Simplest possible system
- Smallest complexity
- Easiest to understand

**Disadvantages**:
- **Not extensible**: Adding features requires code changes
- **Not flexible**: Can't adapt to organizational needs
- **No upgrade path**: Adding features breaks compatibility
- **Not suitable for enterprise**: Customers want flexibility

**Complexity**:
- Minimal

**Performance**:
- Minimal overhead

**Scalability**:
- Simple but not scalable to diverse use cases

**Maintainability**:
- Easiest (nothing optional)

**Testability**:
- Simple

**Operational consequences**:
- Every feature needs code + test + deployment

**Why NOT this**:
- Not suitable for MVP that aims to be useful for varied organizations
- Evidence: Extensive config file exists

---

## TRADEOFF

**Gains**:
- Operational flexibility (disable broken feature at runtime)
- Resource efficiency (don't load unused features)
- Use-case variety (organizations pick features they need)
- Upgrade path (add features without breaking)
- Cognitive load reduced (operators only understand enabled features)

**Sacrifices**:
- Code complexity (conditional initialization paths)
- Testing complexity (feature interaction testing)
- Config management (must verify config is valid)
- Potential for misconfiguration (operator error)
- Slightly more overhead (config lookup, conditional branches)

**Net result**: More flexible and pragmatic, but more complex to test.

---

## FAILURE POINT

Becomes problematic when:

1. **Feature interactions**: If disabled feature is still required by enabled feature (silent failure)
2. **Configuration mistakes**: If config is invalid, silent failures
3. **Testing explosion**: If testing all combinations becomes infeasible
4. **Performance sensitive path**: If config lookup in hot path causes latency

---

## CHANGE CONDITION

Would justify replacing this:

1. **Simplification**: If only one feature combination ever used, hardcode it
2. **Build-time optimization**: If binary size becomes critical constraint
3. **Safety**: If runtime misconfiguration becomes common, move to compile-time

---

## SCALE CONDITION

Becomes problematic at:

- **10+ features**: Feature interaction testing becomes O(2^n) 
- **50+ configurations**: Management becomes complex
- **Mission-critical**: If misconfiguration causes outages, need better validation

---

## LEARNING QUESTION

**Question**: In SecureAI's opt-in architecture, suppose you enable `enable_auth = true` but disable `enable_audit = false`. The system starts successfully. Later, a security requirement emerges that authentication actions must be audited. What are the problems with this architecture's approach, and how would you redesign it?

**Answer should address**:
- Why the current design doesn't catch this (config validation is weak)
- What invariant is violated (auth → audit dependency)
- How to add dependency checking
- Tradeoff between flexibility and safety

---

# DECISION 3: OAuth2/OIDC for Authentication (Not In-House)

## WHY THIS?

**What it does**:
Authentication delegated to external OAuth2/OIDC provider. SecureAI validates JWT tokens, caches JWKS, checks claims. No password storage, no session management in SecureAI.

**Where implemented**:
- `src/auth/jwt.rs:JwtValidator` - Token validation
- `src/auth/jwks.rs:JwksCache` - Key caching
- `src/auth/rbac.rs` - RBAC on top of JWT claims
- `src/api/grpc.rs` - gRPC middleware that validates JWT
- `src/auth/config.rs` - Provider configuration

**Problems solved**:
1. **Password security**: Don't store passwords (OWASP priority)
2. **Session management**: Don't manage sessions (complex, error-prone)
3. **Key rotation**: Provider handles key rotation (we just cache)
4. **Multi-organization**: Different orgs use their own provider
5. **Compliance**: Delegating to compliant provider (SOC2, HIPAA)

**Engineering principle**:
- **Principle of least privilege**: Do only what's necessary, delegate expertise
- **Defense-in-depth**: Rely on provider security + our validation
- **Separation of concerns**: Auth is separate from business logic

**Evidence in repository**:
```rust
// src/auth/jwt.rs
pub struct JwtValidator {
    issuer: String,
    audience: String,
    jwks_cache: JwksCache,  // Caches remote JWKS (1h TTL)
}

impl JwtValidator {
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        // 1. Decode without verification (get kid)
        // 2. Fetch JWKS from cache (or remote if expired)
        // 3. Verify signature with public key
        // 4. Check claims (issuer, audience, expiry)
        // 5. Return Claims if valid
    }
}

// src/api/grpc.rs - Authentication middleware
impl GrpcService {
    pub async fn authenticate_request(&self, metadata: &Metadata) -> Result<AuthContext> {
        let token = metadata.get("authorization")?.extract_bearer()?;
        self.jwt_validator.validate_token(&token)?
        // Token invalid → 401 Unauthenticated
    }
}
```

```toml
# secureai.toml
[auth]
provider_url = "https://accounts.google.com"  # Or Okta, Auth0, etc.
client_id = "xxx"
audience = "secureai-api"
```

---

## WHY NOT THAT?

### Alternative 1: In-House Password Authentication

**How it works**:
- Users register with username/password
- SecureAI stores password hash (bcrypt + salt)
- Login endpoint issues JWT token
- Sessions stored in-memory or database

**Advantages**:
- **Complete control**: All auth logic owned by team
- **Simpler integration**: No external dependency
- **Offline capable**: Works without external service

**Disadvantages**:
- **Password security**: Must implement correctly (bcrypt, salting, rate limiting)
- **Session management**: Complex (session storage, invalidation, expiry)
- **Password reset**: Must implement securely (token-based, expiry)
- **MFA**: Must implement MFA (TOTP, SMS, etc.)
- **Compliance**: Must audit password storage, implement encryption
- **Operational**: Must backup/restore password database
- **Security liability**: One bug = all passwords compromised
- **User burden**: Users must manage passwords (password manager dependency)

**Complexity**:
- Much higher (password reset alone is complex)

**Performance**:
- Same latency (token validation)
- But password database queries add latency

**Scalability**:
- Adds database dependency for auth
- Password database becomes critical resource

**Maintainability**:
- Hard (security is complex, many edge cases)
- Ongoing: password reset flow, MFA bugs, etc.

**Testability**:
- Hard to test security properties
- Need adversarial testing

**Operational consequences**:
- Password database is critical (breach = disaster)
- Must implement audit logging for access attempts
- Compliance implications (SOC2 password handling)
- GDPR implications (password data storage)

**Why NOT this**:
- SecureAI is not an identity provider (doesn't need to be)
- Delegating to expert providers is safer
- Evidence: Auth module is 300 lines (mostly validation, not implementation)

---

### Alternative 2: SAML

**How it works**:
- Enterprise SAML provider (Okta, AzureAD)
- SecureAI validates SAML assertions
- Similar to OIDC but different protocol

**Advantages**:
- **Enterprise friendly**: Okta/AzureAD native (common in enterprises)
- **Strong integration**: Works with enterprise SSO
- **No password**: Like OAuth, delegates auth
- **Legacy support**: Some enterprises only support SAML

**Disadvantages**:
- **More complex**: SAML protocol more verbose than OAuth
- **XML parsing**: Need XML library (attack surface)
- **No standard JWT**: Need to parse SAML assertions (not standardized)
- **Fewer libraries**: Less community support than OAuth
- **Legacy baggage**: Protocol designed 20+ years ago

**Complexity**:
- More complex than OIDC (XML instead of JSON)

**Performance**:
- Same as OIDC (validation only)
- XML parsing slightly slower

**Scalability**:
- Same as OIDC

**Maintainability**:
- Harder (SAML more complex)

**Testability**:
- Harder (XML validation complex)

**Operational consequences**:
- Need SAML-capable IdP (not all providers)

**Why NOT this**:
- OIDC is simpler and more modern
- Majority of providers support OIDC
- Evidence: Code uses OIDC, not SAML

---

### Alternative 3: Mutual TLS (mTLS)

**How it works**:
- Each client has certificate signed by CA
- gRPC connection verified via mutual TLS
- No JWT token needed

**Advantages**:
- **Simple**: No token to manage
- **Cryptographic**: Based on certificates
- **Mutual auth**: Both sides verify each other

**Disadvantages**:
- **Certificate management**: Must issue/revoke/rotate certificates
- **Certificate storage**: Clients must securely store cert + key
- **Distribution**: How to deliver certs to clients
- **Revocation**: No simple revocation mechanism
- **User management**: Hard to map certificate → user
- **Legacy systems**: Difficult for web browser clients

**Complexity**:
- Certificate management complex
- Client-side storage complex

**Performance**:
- TLS handshake adds latency (~50ms)
- But session caching reduces overhead

**Scalability**:
- Certificate authority scalability question
- CRL/OCSP revocation checks add latency

**Maintainability**:
- Hard (certificate lifecycle management)

**Testability**:
- Hard (need to generate test certificates)

**Operational consequences**:
- Must manage CA
- Must handle certificate expiry
- Client onboarding complex

**Why NOT this**:
- Not suitable for multi-user systems
- Hard to map certificate → identity
- OAuth is easier for this use case
- Evidence: gRPC uses standard auth metadata (not mTLS)

---

## TRADEOFF

**Gains**:
- Password security delegated (not our problem)
- Session management delegated
- Multi-org support (each org has own provider)
- Compliance easier (provider handles SOC2/HIPAA)
- Key rotation automatic (we cache, provider rotates)
- MFA support free (provider implements)

**Sacrifices**:
- External dependency (provider downtime = auth broken)
- Network latency (JWKS fetch if cache expired)
- Trust in provider (must trust their security)
- Limited customization (must use provider's claims format)
- Vendor lock-in (switching providers requires reconfig)

**Net result**: Simpler and safer, but dependent on external service.

---

## FAILURE POINT

Becomes problematic when:

1. **Provider downtime**: If provider unreachable, new tokens can't be validated (mitigated by JWKS cache)
2. **Key rotation**: If provider rotates key, cache becomes stale (mitigated by TTL)
3. **Compromised provider**: If provider account compromised, attacker controls access
4. **Unusual claims**: If business logic needs custom claims, limited flexibility

---

## CHANGE CONDITION

Would justify replacing this:

1. **Offline requirement**: If system must work without internet, would need local auth
2. **Custom identity attributes**: If need complex custom claims beyond standard
3. **Cost**: If provider pricing becomes prohibitive (would build in-house)
4. **Regulatory**: If regulator prohibits using external provider

---

## SCALE CONDITION

Becomes problematic at:

- **JWKS cache TTL**: At 100,000 req/sec, cache becomes stale (need lower TTL = more remote calls)
- **Provider rate limits**: If SecureAI scales past provider's rate limits
- **Provider SLA**: If provider SLA is 99.9%, that's 43 minutes of downtime/month

---

## LEARNING QUESTION

**Question**: JWKS cache has 1-hour TTL. If an attacker compromises the OAuth provider and rotates keys, what happens to SecureAI requests? How would you design the system to detect this faster?

**Answer should address**:
- Current behavior (served from cache for 1h after compromise)
- Why TTL exists (balance between cache freshness and provider load)
- How to detect key rotation (signature verification failures)
- Tradeoff between latency (cache) and security (fresh keys)

---

# DECISION 4: Ed25519 Signatures for Audit Trail (Not HMAC)

## WHY THIS?

**What it does**:
Every audit log entry signed with Ed25519 asymmetric cryptography. Each entry includes: hash of previous entry, signature of current entry, then immutable. Private key signs, public key verifies, signature proves identity.

**Where implemented**:
- `src/audit/ledger.rs:AuditLedger::append_entry()` - Signing
- `src/audit/keys.rs:KeyManager` - Key management
- `src/audit/verify.rs:verify_chain()` - Signature verification
- `src/audit/persist.rs` - Persistence with signature

**Problems solved**:
1. **Non-repudiation**: Entry signed by specific key (can't deny later)
2. **Identity**: Signature proves who signed (if key is protected)
3. **Integrity**: Signature verification detects tampering
4. **Chain integrity**: Hash chain breaks if entry modified

**Engineering principle**:
- **Non-repudiation**: Cryptographic proof of action
- **Defense-in-depth**: Both signature + hash chain
- **Asymmetric trust**: Can verify without sharing secret

**Evidence in repository**:
```rust
// src/audit/ledger.rs
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: i64,
    pub action: String,
    pub subject: String,
    pub details: serde_json::Value,
    pub hash: String,           // SHA256 of previous_hash || entry_data
    pub signature: String,      // Ed25519 signature of hash
}

impl AuditLedger {
    pub fn append_entry(&mut self, mut entry: AuditEntry) -> Result<u64> {
        // 1. Compute hash: SHA256(prev_hash || serialize(entry))
        entry.hash = self.compute_hash(&entry)?;
        
        // 2. Sign hash with Ed25519 private key
        entry.signature = self.sign_entry(&entry)?;
        
        // 3. Add to in-memory chain
        self.entries.push(entry.clone());
        
        // 4. Persist to file (if enabled)
        self.persist.write_entry(&entry)?;
        
        Ok(entry.id)
    }
}

// src/audit/verify.rs
pub fn verify_chain(entries: &[AuditEntry], public_key: &[u8]) -> Result<()> {
    let mut prev_hash = String::new();
    
    for entry in entries {
        // 1. Verify hash includes previous hash
        let expected_hash = compute_hash(&prev_hash, entry)?;
        if entry.hash != expected_hash {
            return Err("Hash chain broken");
        }
        
        // 2. Verify Ed25519 signature
        ed25519_verify(public_key, &entry.hash, &entry.signature)?;
        
        prev_hash = entry.hash.clone();
    }
    
    Ok(())
}
```

---

## WHY NOT THAT?

### Alternative 1: HMAC (Symmetric Signing)

**How it works**:
- Shared secret key (e.g., HMAC-SHA256)
- Same key used to sign and verify
- Signature doesn't prove identity, only knowledge of key

**Advantages**:
- **Simpler**: HMAC easier to implement
- **Faster**: Symmetric crypto faster than asymmetric
- **Standard**: HMAC built into all crypto libraries
- **Familiar**: Most developers know HMAC

**Disadvantages**:
- **No non-repudiation**: HMAC can be computed by anyone with secret key
- **Identity ambiguous**: Can't prove *which* entity signed
- **Shared secret problem**: Must distribute/rotate secret securely
- **Compliance weak**: Doesn't meet audit log requirements (can deny)
- **Key compromise**: If key stolen, can't prove who did what

**Complexity**:
- Simpler implementation

**Performance**:
- Faster (symmetric crypto)

**Scalability**:
- Same

**Maintainability**:
- Simpler (no asymmetric key management)

**Testability**:
- Simpler

**Operational consequences**:
- Shared secret must be managed
- Audit logs don't prove identity

**Why NOT this**:
- Audit trail must be non-repudiable
- HMAC doesn't prove identity
- If audit is subpoenaed, HMAC doesn't prove who acted
- Evidence: `src/audit/keys.rs` uses asymmetric Ed25519, not HMAC

---

### Alternative 2: RSA Signatures

**How it works**:
- RSA public/private keypair (2048+ bits)
- Private key signs, public key verifies
- Similar to Ed25519 but different algorithm

**Advantages**:
- **Non-repudiation**: Same as Ed25519
- **Well-known**: RSA standard for decades
- **Wide support**: Every crypto library supports RSA
- **Proven**: Most audited algorithm

**Disadvantages**:
- **Large signatures**: RSA-2048 = 256-byte signature (vs Ed25519 = 64 bytes)
- **Large keys**: 2048-bit keys vs 32-byte Ed25519 keys
- **Slower**: RSA crypto slower than Ed25519
- **Legacy complexity**: RSA has many variants (OAEP, PKCS#1, etc.)
- **Padding oracle attacks**: Historical RSA vulnerabilities
- **Key generation**: RSA key generation slow and complex

**Complexity**:
- More complex (padding, key generation)

**Performance**:
- Slower than Ed25519 (~10-50x)
- Larger overhead

**Scalability**:
- At scale, Ed25519 more efficient

**Maintainability**:
- Harder (RSA more complex)

**Testability**:
- Same difficulty

**Operational consequences**:
- Larger storage (larger signatures and keys)

**Why NOT this**:
- Ed25519 designed specifically for this (faster, simpler, smaller)
- No advantage of RSA for audit trail
- Evidence: Repository uses Ed25519, not RSA

---

### Alternative 3: No Signatures (Hash Chain Only)

**How it works**:
- Only hash chain (no signature)
- Each entry includes SHA256(prev_hash || entry_data)
- Can detect tampering but not prove who wrote

**Advantages**:
- **Simplest**: Only hash computation
- **Fastest**: Hash operations are very fast
- **Smallest**: Just hash, no signature

**Disadvantages**:
- **No identity**: Can't prove who wrote entry
- **No non-repudiation**: Anyone can generate valid hash chain
- **Insufficient for audit**: Doesn't meet compliance (must prove identity)
- **Weak guarantee**: Tampering detected but source unknown

**Complexity**:
- Simplest (only hashing)

**Performance**:
- Fastest

**Scalability**:
- Best (minimal overhead)

**Maintainability**:
- Simplest

**Testability**:
- Simplest

**Operational consequences**:
- Audit logs don't prove identity

**Why NOT this**:
- Audit trails must prove identity (SOC2/HIPAA requirement)
- Hash chain alone insufficient
- Evidence: Repository includes Ed25519 signatures

---

## TRADEOFF

**Gains**:
- Non-repudiation (cryptographic proof of identity)
- Chain integrity (tampering detected)
- Compliance (meets audit requirements)
- Legal defensibility (signature proves identity)

**Sacrifices**:
- Performance (signature computation ~1ms per entry)
- Complexity (asymmetric key management)
- Storage (signature adds 64 bytes per entry)
- Key management (private key must be protected)

**Net result**: Stronger audit trail, but more complex.

---

## FAILURE POINT

Becomes problematic when:

1. **Private key compromised**: Attacker can forge signatures
2. **Key loss**: If private key lost, can't sign new entries
3. **Audit trail too large**: At 1TB+ of entries, verification slow

---

## CHANGE CONDITION

Would justify replacing this:

1. **Performance critical**: If signature computation becomes bottleneck (unlikely)
2. **Compliance relaxation**: If audit identity proof no longer required
3. **Hardware acceleration**: If hardware signatures become available (would optimize, not replace)

---

## SCALE CONDITION

Becomes problematic at:

- **Verification time**: At 1 billion entries, verify_chain() takes hours
- **Storage**: At extreme scale, signatures become significant (64 bytes × 1B = 64GB)
- **Key management**: At 1000+ servers, managing private keys becomes complex

---

## LEARNING QUESTION

**Question**: The audit system stores both hash chain and Ed25519 signatures. Suppose an administrator modifies an entry on disk (bypassing the application). What would be detected and what wouldn't?

**Answer should demonstrate understanding of**:
- What hash chain detects (entry modification)
- What signature detects (entry replacement or tampering)
- Why both are needed (defense-in-depth)
- Limitations (key compromise would allow forging)

---

# DECISION 5: NATS JetStream for Task Queue (Not Redis/SQS)

## WHY THIS?

**What it does**:
Tasks queued in NATS JetStream (distributed message broker with persistence). Queue provides: at-least-once delivery, ordered messages, multiple consumer groups, persistent storage on disk.

**Where implemented**:
- `src/queue/mod.rs:NatsProducer` - Task enqueue
- `src/queue/consumer.rs:NatsConsumer` - Task dequeue
- `src/queue/config.rs` - JetStream configuration
- `main.rs:120-130` - Queue initialization

**Problems solved**:
1. **Distributed queuing**: Multiple workers process tasks
2. **Persistence**: Tasks survive crashes (on-disk persistence)
3. **Ordering**: Tasks processed in order
4. **Multiple consumers**: Different consumers for different task types
5. **At-least-once delivery**: Failed tasks automatically requeued

**Engineering principle**:
- **Resilience**: Distributed queue survives failures
- **Ordering**: FIFO for deterministic behavior
- **Scalability**: Multiple workers scale linearly

**Evidence in repository**:
```rust
// src/queue/mod.rs
pub struct NatsProducer {
    client: nats::asynk::Connection,
    stream: String,  // JetStream stream name
}

impl NatsProducer {
    pub async fn enqueue_task(&self, task: Task) -> Result<u64> {
        let subject = format!("tasks.{}", task.task_type);
        let payload = serde_json::to_vec(&task)?;
        
        // Publish to JetStream (persisted)
        let ack = self.client.publish_async(&subject, payload).await?;
        
        Ok(ack.sequence)
    }
}

// src/queue/consumer.rs
pub struct NatsConsumer {
    client: nats::asynk::Connection,
    stream: String,
}

impl NatsConsumer {
    pub async fn get_next_task(&self) -> Result<Task> {
        // 1. Pull from JetStream (blocks until task available)
        let msg = self.client.pull_subscribe("tasks.>")?.next_msg(timeout).await?;
        
        // 2. Deserialize task
        let task: Task = serde_json::from_slice(&msg.data)?;
        
        // 3. Return (caller must ack)
        Ok(task)
    }
    
    pub async fn ack_task(&self, task_id: u64) -> Result<()> {
        // Acknowledge delivery (JetStream removes message)
        self.ack_message(task_id).await
    }
}
```

---

## WHY NOT THAT?

### Alternative 1: Redis (Using RPUSH/LPOP)

**How it works**:
- Use Redis lists: RPUSH to enqueue, BLPOP to dequeue
- In-memory queue (fast)
- Optional persistence (RDB/AOF)

**Advantages**:
- **Very fast**: In-memory operations (~microseconds)
- **Simple**: List operations are trivial
- **Widely used**: Most teams know Redis
- **Operational**: Most deployments already have Redis
- **Multiple consumers**: BLPOP supports multiple workers

**Disadvantages**:
- **Memory-bound**: Limited by RAM (can't queue more than memory)
- **Limited persistence**: RDB/AOF is eventual consistency (can lose data)
- **No ordering guarantees**: Rebalancing can break order
- **No message acknowledgment**: No way to know if task failed
- **No dead-letter queue**: Failed tasks lost
- **Single point of failure**: Master failure = queue lost (until replication recovers)

**Complexity**:
- Simpler initial setup
- But reliability requires cluster setup (complex)

**Performance**:
- Faster than NATS (in-memory)
- But limited by single node throughput

**Scalability**:
- Limited by single Redis instance memory
- Sharding adds complexity

**Maintainability**:
- Operational simplicity if small queue
- Redis cluster management complex at scale

**Testability**:
- Easy (in-memory, no network)

**Operational consequences**:
- Memory management (when queue full, what happens?)
- Persistence unreliable (can lose tasks)
- No built-in monitoring

**Why NOT this**:
- Not reliable enough for critical tasks
- Loss of tasks unacceptable
- Evidence: Repository uses NATS, not Redis

---

### Alternative 2: AWS SQS

**How it works**:
- AWS managed service for queuing
- HTTP API for produce/consume
- Persistent, distributed, managed

**Advantages**:
- **Managed service**: AWS operates it (high reliability)
- **Scalable**: Unlimited queue depth
- **Reliable**: Durable storage, replicated
- **Decoupled**: Producer/consumer completely independent
- **Monitoring**: CloudWatch integration

**Disadvantages**:
- **Vendor lock-in**: AWS-specific API
- **Latency**: HTTP API slower than binary protocol (100ms+ vs 10ms)
- **Cost**: Per-request pricing adds up (millions of messages = expensive)
- **Ordering**: Standard SQS doesn't guarantee order (FIFO SQS does)
- **Exactly-once**: SQS is at-least-once only
- **Complexity**: AWS-specific configuration
- **Limited debugging**: Can't inspect queues locally

**Complexity**:
- Simpler operationally (AWS manages)
- But vendor-specific APIs

**Performance**:
- Slower than local NATS (network latency)

**Scalability**:
- Unlimited (AWS manages)

**Maintainability**:
- Simpler (AWS manages operations)
- But vendor-locked

**Testability**:
- Harder (requires AWS SDK, can't test locally easily)

**Operational consequences**:
- AWS dependency (cost, availability)
- Debugging difficult (opaque queue state)
- Monitoring through CloudWatch

**Why NOT this**:
- For on-prem or multi-cloud, NATS is better
- Vendor lock-in acceptable tradeoff depends on deployment
- Evidence: Repository self-hosts NATS

---

### Alternative 3: Apache Kafka

**How it works**:
- Distributed event streaming platform
- Topic-based publish/subscribe
- Persistent storage, consumer groups

**Advantages**:
- **Scalable**: Handles millions of messages/sec
- **Reliable**: Replicated, durable
- **Consumer groups**: Multiple consumers independently
- **Monitoring**: Built-in metrics
- **Topic replication**: Fault-tolerant

**Disadvantages**:
- **Complex**: Requires cluster management (ZooKeeper until 3.0)
- **Operational overhead**: Must manage brokers, partitions
- **Storage-heavy**: Stores all messages (not good for short-lived tasks)
- **Overkill**: For moderate throughput, Kafka is heavyweight
- **Learning curve**: Complex concepts (partitions, offsets, consumer lag)
- **Latency**: Not optimized for low-latency (batch-oriented)

**Complexity**:
- High (cluster management, partition balancing)

**Performance**:
- High throughput but higher latency (~50ms+ due to batching)

**Scalability**:
- Scales to petabytes
- Overkill for SecureAI

**Maintainability**:
- Hard (operational complexity)

**Testability**:
- Hard (need cluster for testing)

**Operational consequences**:
- Cluster management required
- Significant operational overhead

**Why NOT this**:
- Overkill for SecureAI's throughput (hundreds of tasks/sec, not millions)
- Operational complexity not justified
- Evidence: NATS chosen (simpler alternative)

---

## TRADEOFF

**Gains**:
- Reliable delivery (persistent, replicated)
- Ordering guarantee (FIFO)
- Message acknowledgment (exactly-once semantics with ack)
- Multiple consumer groups (flexible scaling)
- Operational simplicity (simpler than Kafka)
- Cost-effective (no per-request pricing)

**Sacrifices**:
- Operational complexity (must run NATS cluster)
- Latency slightly higher than Redis (network protocol)
- Operational knowledge (team must know NATS)
- Monitoring complexity (need NATS-specific tools)

**Net result**: Reliable and scalable, with acceptable operational overhead.

---

## FAILURE POINT

Becomes problematic when:

1. **NATS cluster failure**: All nodes down = queue unavailable
2. **Disk full**: Persistence fails if disk full
3. **Network partition**: Split-brain scenarios (NATS handles poorly)
4. **High throughput**: At 100k+ msg/sec, NATS becomes bottleneck

---

## CHANGE CONDITION

Would justify replacing this:

1. **Massive scale**: At 1M+ msg/sec, Kafka makes sense
2. **AWS requirement**: If AWS-only deployment, SQS makes sense
3. **Redis already deployed**: If Redis cluster exists, might use it
4. **Simplification**: If queue complexity not needed, could use database

---

## SCALE CONDITION

Becomes problematic at:

- **100k msg/sec**: Single NATS node saturated
- **1TB+ of messages**: Disk storage becomes expensive
- **Global distribution**: Single cluster in one region has latency

---

## LEARNING QUESTION

**Question**: If a NATS broker crashes mid-task-processing, what happens to:
a) Tasks already enqueued but not yet pulled by consumer?
b) Tasks being processed by a consumer that dies?
c) Task that was pulled and acknowledged before crash?

**Answer should demonstrate understanding of**:
- NATS persistence model (messages stored on disk)
- At-least-once delivery (redelivery if consumer dies)
- Acknowledgment semantics (ack = safe to remove)
- How crash recovery works

---

# DECISION 6: Two-Tier Semantic Cache (Exact + Semantic)

## WHY THIS?

**What it does**:
Cache has two layers:
- **Tier 1 (Exact)**: Exact prompt match (hash-based lookup, O(1), 100% match)
- **Tier 2 (Semantic)**: Prompt similarity via embeddings (cosine distance, O(n), ~80% match)

If Tier 1 misses, falls back to Tier 2. If both miss, computes and caches.

**Where implemented**:
- `src/cache/mod.rs:CacheManager` - Two-tier coordination
- `src/cache/exact.rs:ExactCache` - Hash-based exact matching
- `src/cache/semantic.rs:SemanticCache` - Embedding-based similarity
- `src/cache/embedder.rs:Embedder` - ONNX model for embeddings

**Problems solved**:
1. **Exact duplicate**: Same prompt = instant response (cache hit)
2. **Similar prompts**: Similar prompts = similar response (semantic cache hit)
3. **Cache efficiency**: Two strategies reduce misses (60-80% hit rate)
4. **Latency**: Hit in Tier 1 = 1ms; hit in Tier 2 = 60ms; miss = 5000ms

**Engineering principle**:
- **Caching strategy**: Multiple levels for different scenarios
- **Optimization**: Focus on common case (exact match) but handle similar (semantic)
- **Graceful degradation**: Tier 1 miss doesn't prevent Tier 2 hit

**Evidence in repository**:
```rust
// src/cache/mod.rs
pub struct CacheManager {
    exact_cache: ExactCache,
    semantic_cache: SemanticCache,
}

impl CacheManager {
    pub async fn get_or_compute(
        &self,
        prompt: &str,
    ) -> Result<CachedResponse> {
        // Tier 1: Exact match
        if let Some(cached) = self.exact_cache.get(prompt)? {
            metrics.cache_hit_tier1.inc();
            return Ok(cached);
        }
        
        // Tier 2: Semantic similarity
        let embedding = self.embedder.embed(prompt).await?;
        if let Some(similar) = self.semantic_cache.find_similar(&embedding)? {
            metrics.cache_hit_tier2.inc();
            return Ok(similar.response);
        }
        
        // Miss: Compute and cache both tiers
        metrics.cache_miss.inc();
        let response = compute_response(prompt).await?;
        self.exact_cache.set(prompt, &response)?;
        self.semantic_cache.set(&embedding, &response)?;
        Ok(response)
    }
}

// src/cache/exact.rs
pub struct ExactCache {
    store: HashMap<String, CachedResponse>,  // prompt_hash -> response
    ttl_secs: u64,
    max_size: usize,
}

// src/cache/semantic.rs
pub struct SemanticCache {
    embeddings: Vec<Embedding>,  // Vector store
    responses: Vec<CachedResponse>,
    embedder: Arc<Embedder>,
    similarity_threshold: f32 = 0.85,  // Cosine similarity
}

impl SemanticCache {
    pub fn find_similar(&self, query_embedding: &[f32]) -> Result<Option<CacheEntry>> {
        let mut best_match = None;
        let mut best_score = 0.0;
        
        // Compute cosine distance to all embeddings
        for (idx, stored_embedding) in self.embeddings.iter().enumerate() {
            let score = cosine_similarity(query_embedding, &stored_embedding);
            
            if score > self.similarity_threshold && score > best_score {
                best_match = Some(idx);
                best_score = score;
            }
        }
        
        best_match.map(|idx| CacheEntry {
            response: self.responses[idx].clone(),
            similarity_score: best_score,
        })
    }
}
```

---

## WHY NOT THAT?

### Alternative 1: Exact Cache Only

**How it works**:
- Only hash-based matching
- Same prompt = cache hit
- Different prompt = cache miss

**Advantages**:
- **Simplest**: Only exact matching logic
- **Perfect accuracy**: No false positives
- **Fastest**: O(1) lookup
- **Easiest verification**: Deterministic results

**Disadvantages**:
- **Low hit rate**: Only exact matches (20-30% typical)
- **Limited value**: Similar prompts miss
- **Unchanged problem**: User asks different version of same question, no cache benefit

**Complexity**:
- Minimal

**Performance**:
- Best (O(1) lookups)
- But low hit rate

**Scalability**:
- Memory proportional to unique prompts

**Maintainability**:
- Easiest

**Testability**:
- Simplest (deterministic)

**Operational consequences**:
- Cache misses frequently

**Why NOT this**:
- Hit rate too low (~20-30% vs 60-80% with semantic)
- Users frequently rephrase same question
- Evidence: Repository implements both tiers

---

### Alternative 2: Semantic Cache Only

**How it works**:
- Only embedding-based similarity
- Similar prompts = cache hit
- Exact matching not optimized

**Advantages**:
- **Higher hit rate**: Catches similar prompts
- **Smarter**: Understands semantic equivalence
- **Fewer cache misses**: More prompts match

**Disadvantages**:
- **Slower**: O(n) similarity search (n = cache size)
- **False positives**: Different prompts might match (if threshold low)
- **Embedding quality**: Depends on model quality
- **Latency**: Embedding computation (~20-60ms)
- **No perfect match**: Even exact duplicates need embedding computation

**Complexity**:
- Higher (embeddings, similarity search, threshold tuning)

**Performance**:
- Slower (O(n) vs O(1))
- Hits more often but each hit slower

**Scalability**:
- Vector search becomes bottleneck at large cache (O(n) per lookup)
- Would need approximate NN (FAISS, Annoy) at scale

**Maintainability**:
- Harder (tuning similarity threshold is art)

**Testability**:
- Harder (similarity results non-deterministic)

**Operational consequences**:
- Must monitor embedding quality
- False positive rates important

**Why NOT this**:
- Tier 1 handles exact duplicates much faster (1ms vs 60ms)
- Exact matches happen frequently enough to optimize
- Evidence: Two-tier approach chosen

---

### Alternative 3: Database (SQLite/PostgreSQL)

**How it works**:
- Store prompts and responses in database
- Query for exact match or similarity search

**Advantages**:
- **Persistent**: Survives restarts
- **Queryable**: SQL-based queries
- **Scalable**: Can index for performance
- **Durable**: ACID transactions

**Disadvantages**:
- **Slower**: Disk I/O (5-50ms vs 1-10ms memory)
- **Operational**: Need to manage database
- **Embedding complexity**: Still need embedding model
- **Connection pooling**: Threading concerns
- **Query cost**: Full table scan for similarity search

**Complexity**:
- Higher (database management)

**Performance**:
- Slower than memory (disk I/O)

**Scalability**:
- Can scale larger than memory (disk-backed)
- But slower

**Maintainability**:
- Harder (database operational overhead)

**Testability**:
- Harder (need test database)

**Operational consequences**:
- Database backup/restore required
- Schema management

**Why NOT this**:
- In-memory is faster
- Persistence not required (cache, not source of truth)
- Evidence: Repository uses in-memory cache

---

## TRADEOFF

**Gains**:
- High hit rate (60-80% vs 20-30% exact-only)
- Optimized for common case (exact match is fast)
- Graceful degradation (Tier 2 if Tier 1 misses)
- Fast common case (1ms for exact hits)
- Reasonable worst case (60ms for semantic hits)

**Sacrifices**:
- Complexity (two cache implementations)
- Memory usage (both caches must coexist)
- Tuning complexity (similarity threshold tuning)
- Embedding dependency (ONNX model required)
- Non-deterministic results (semantic cache similarity)

**Net result**: Much higher hit rate with acceptable complexity.

---

## FAILURE POINT

Becomes problematic when:

1. **Cache size explodes**: O(n) semantic search becomes slow (need FAISS)
2. **Embedding model fails**: If model unavailable, Tier 2 broken
3. **Threshold tuning**: If too strict (no semantic hits); too loose (false positives)
4. **Memory pressure**: Cache eats all available memory

---

## CHANGE CONDITION

Would justify replacing this:

1. **Persistence required**: If cache survives restarts needed, use database
2. **Extreme scale**: At 10M+ cached prompts, need database + FAISS
3. **Deterministic requirement**: If non-determinism unacceptable, drop semantic
4. **Embedding cost**: If embedding computation too expensive, drop Tier 2

---

## SCALE CONDITION

Becomes problematic at:

- **100k cached prompts**: Semantic search O(n) becomes 100ms per lookup
- **1GB cache**: Memory pressure, LRU eviction starts
- **1M cached prompts**: Semantic search O(n) becomes 1+ second (unacceptable)

---

## LEARNING QUESTION

**Question**: A user submits prompt "What is the capital of France?" at 10am and gets result cached. At 11am, another user submits "What's the capital of France?" (same question, different phrasing). Which cache tier will hit?

a) Tier 1 only
b) Tier 2 only
c) Tier 1 will miss, Tier 2 will miss (miss)
d) Tier 1 will miss, Tier 2 will hit (semantic match)

**Answer should demonstrate understanding of**:
- Exact matching requires identical hash
- Semantic matching uses embeddings (captures meaning, not exact text)
- How two-tier strategy works (fast path + fallback)
- Why semantic cache is valuable for natural language

---

# DECISION 7: Async/Await (Tokio) for Concurrency

## WHY THIS?

**What it does**:
All I/O operations (disk, network, semaphore waits) use async/await with Tokio runtime. No blocking calls in async contexts. Enables 1000+ concurrent connections with limited threads.

**Where implemented**:
- `src/main.rs` - `#[tokio::main]` async runtime init
- `src/api/grpc.rs` - Async gRPC handler
- `src/queue/consumer.rs` - Async queue consumption
- `src/cache/semantic.rs` - Async embedding computation
- All I/O operations `.await`

**Problems solved**:
1. **High concurrency**: Handle 1000+ concurrent requests with few threads
2. **Resource efficiency**: Threads are expensive (1MB+ stack each)
3. **Responsiveness**: Slow I/O doesn't block other requests
4. **Latency**: Fast requests aren't delayed by slow ones

**Engineering principle**:
- **Non-blocking concurrency**: Async instead of threads
- **Resource efficiency**: Maximize throughput with minimal resources
- **Composability**: Easy to build concurrent operations

**Evidence in repository**:
```rust
// src/main.rs
#[tokio::main]
async fn main() {
    // Tokio runtime initialized
    let server = grpc::start_server().await?;
    server.serve().await?;
}

// src/api/grpc.rs
#[async_trait]
impl Policy for PolicyService {
    async fn evaluate_policy(
        &self,
        request: Request<EvaluatePolicyRequest>,
    ) -> Result<Response<EvaluatePolicyResponse>, Status> {
        // Async function
        let req = request.into_inner();
        
        // All I/O operations are awaited
        let validated = self.engine.validate_task(&req).await?;
        let cached = self.cache.get(&req.prompt).await?;
        let result = self.execute(&req).await?;
        
        Ok(Response::new(result))
    }
}

// src/queue/consumer.rs
pub async fn consume_tasks(consumer: Arc<NatsConsumer>) {
    loop {
        // Non-blocking wait for next task
        if let Ok(task) = consumer.next_task().await {
            tokio::spawn(async move {
                // Each task processed concurrently
                let result = execute_task(task).await;
                consumer.ack_task(task.id).await.ok();
            });
        }
    }
}

// src/cache/semantic.rs
pub async fn embed_prompt(&self, prompt: &str) -> Result<Vec<f32>> {
    // Embedding computation is async (might be blocking internally)
    // but doesn't block event loop
    tokio::task::spawn_blocking(move || {
        self.model.embed(prompt)
    })
    .await?
}
```

---

## WHY NOT THAT?

### Alternative 1: OS Threads (Thread Per Request)

**How it works**:
- One thread per request
- Thread blocks on I/O
- OS schedules threads

**Advantages**:
- **Simple**: Straightforward programming model
- **Familiar**: Most languages use this (Java, Python)
- **Debugging**: Easy to debug (stack traces clear)
- **Blocking I/O**: Can use standard blocking libraries

**Disadvantages**:
- **Memory**: Thread stack = 1-8 MB (limited to 1000s of concurrent connections)
- **Context switching**: OS scheduler overhead
- **Scalability**: At 10k concurrent connections, performance degrades
- **Resource**: 10k threads = 10-80GB of thread stacks

**Complexity**:
- Simpler programming (no async/await)
- But synchronization complexity (mutexes, deadlocks)

**Performance**:
- Acceptable for moderate load (<1000 concurrent)
- Bad for high concurrency (10k+)

**Scalability**:
- Limited to ~1000 concurrent connections per machine
- Doesn't scale to 10k+ connections

**Maintainability**:
- Easier (simpler programming model)
- But harder to reason about concurrency (deadlocks, races)

**Testability**:
- Easier (no async runtime needed)

**Operational consequences**:
- Memory usage high (10k connections = 10-80GB)
- CPU overhead from context switching

**Why NOT this**:
- Doesn't scale to high concurrency
- Resource usage unacceptable
- Evidence: Tokio used (async approach)

---

### Alternative 2: Green Threads (Goroutines / Virtual Threads)

**How it works**:
- Language-level lightweight threads
- Like OS threads but managed by runtime
- Go, Erlang use this

**Advantages**:
- **Simpler than async**: More like traditional threads
- **Efficient**: Lightweight (100KB stack vs 1MB)
- **Scalable**: Can have millions of green threads
- **Familiar paradigm**: Thread-like programming model

**Disadvantages**:
- **GC overhead**: Green threads need garbage collection
- **Scheduling complexity**: Runtime scheduler complexity
- **Rust limitation**: No stable green thread implementation (only experimental)
- **Context switching**: Still has scheduling overhead (less than OS threads but present)

**Complexity**:
- Similar to async (need scheduler)
- Slightly simpler programming model

**Performance**:
- Better than OS threads (10k+ concurrent possible)
- Worse than async (scheduling overhead)

**Scalability**:
- Can scale to 100k+ concurrent
- But with overhead vs async

**Maintainability**:
- Slightly easier than async (no .await)
- But similar complexity

**Testability**:
- Similar to async

**Operational consequences**:
- GC pauses possible
- Scheduling overhead

**Why NOT this**:
- Rust doesn't have stable green threads
- Async is more efficient
- Evidence: Rust ecosystem standardized on async

---

### Alternative 3: Event Loop (Node.js Style)

**How it works**:
- Single thread with event loop
- Callbacks for I/O operations

**Advantages**:
- **Simple**: One thread, sequential execution
- **Efficient**: No context switching
- **Lightweight**: Minimal overhead

**Disadvantages**:
- **Callback hell**: Complex nesting for multiple operations
- **No parallelism**: Single thread can't use multiple CPU cores
- **Error handling**: Exceptions in callbacks hard to handle
- **Debugging**: Stack traces incomplete (lost across events)
- **Latency sensitive**: Single slow operation blocks all others

**Complexity**:
- Complex due to callback chains

**Performance**:
- Good for I/O-heavy (no context switching)
- Bad for CPU-heavy (can't parallelize)

**Scalability**:
- Limited to single core
- Doesn't scale to multi-CPU

**Maintainability**:
- Hard (callback chains)

**Testability**:
- Hard (callback testing complex)

**Operational consequences**:
- Single-threaded (can't use multiple cores)
- CPU-heavy tasks bottleneck entire system

**Why NOT this**:
- Single-threaded limits scalability
- Callback style hard to read
- Rust async is better (structured concurrency, multiple cores)
- Evidence: Tokio supports multiple cores, not single-threaded

---

## TRADEOFF

**Gains**:
- High concurrency (10k+ concurrent connections)
- Resource efficiency (minimal memory per connection)
- Responsiveness (fast requests aren't delayed)
- Multi-core utilization (Tokio uses work stealing)
- Rust safety (no data races if done correctly)

**Sacrifices**:
- Complexity (async/await learning curve)
- Debugging difficulty (async stack traces unclear)
- Library availability (must use async libraries)
- Latency: Embedding task blocks event loop (mitigated with spawn_blocking)
- Cognitive load (must understand async concepts)

**Net result**: More complex to write, but scales much better.

---

## FAILURE POINT

Becomes problematic when:

1. **Blocking in async**: If someone calls blocking I/O in async context, starves event loop
2. **Many CPU-bound tasks**: Async doesn't help (still single-threaded for CPU work)
3. **Debugging complexity**: Hard to trace execution flow across awaits
4. **Library mismatch**: Must use async libraries (blocking libraries deadlock)

---

## CHANGE CONDITION

Would justify replacing this:

1. **Workload change**: If workload becomes mostly CPU-bound, threads better
2. **Simplicity requirement**: If complexity unacceptable, use thread pool
3. **Team expertise**: If team doesn't understand async, use threads
4. **Latency requirement**: If tail latencies matter more than throughput, threads better

---

## SCALE CONDITION

Becomes problematic at:

- **100k+ concurrent requests**: Tokio still handles but context switching overhead
- **CPU-bound work**: Async doesn't help (need true parallelism)
- **Cascade failures**: Single slow task can cause request timeouts (need bulkheads)

---

## LEARNING QUESTION

**Question**: You have an async function that calls `.embed(prompt)` which blocks for 500ms. The Tokio runtime has one worker thread. What happens when this function is called?

a) Event loop blocks for 500ms (no other requests processed)
b) Other requests processed immediately (no blocking)
c) Event loop waits but other requests queued (fairness loss)
d) Program panics (invalid async)

**Answer should demonstrate understanding of**:
- Async/await does not provide true parallelism for blocking work
- spawn_blocking() solution (run blocking work in thread pool)
- Event loop starvation (single blocker starves system)
- Difference between async (non-blocking) and parallelism (multi-threaded)

---

# DECISION 8: gRPC for Control Plane API (Not REST)

## WHY THIS?

**What it does**:
Policy evaluation and task execution exposed via gRPC (binary RPC protocol using HTTP/2). Tonic Rust library implements server. Protocol buffers define contract.

**Where implemented**:
- `src/api/grpc.rs` - gRPC service implementation
- `proto/policy.proto` - Service definition
- `src/api/service.rs` - Service trait implementation
- `main.rs:170-190` - gRPC server startup

**Problems solved**:
1. **Performance**: Binary protocol faster than JSON
2. **Latency**: Multiplexing via HTTP/2 reduces round-trips
3. **Type safety**: Protocol buffers enforce schema
4. **Streaming**: Bidirectional streaming support (SSE events)
5. **Deadline propagation**: gRPC deadline/context propagation

**Engineering principle**:
- **Efficient serialization**: Binary instead of text
- **Multiplexing**: HTTP/2 multiplexing over single connection
- **Schema-driven**: Protocol buffers define contract

**Evidence in repository**:
```rust
// proto/policy.proto
syntax = "proto3";

service Policy {
    rpc EvaluatePolicy (EvaluatePolicyRequest) returns (EvaluatePolicyResponse) {}
    rpc StreamEvents (StreamRequest) returns (stream Event) {}
}

// src/api/grpc.rs
#[tonic::async_trait]
impl Policy for PolicyService {
    async fn evaluate_policy(
        &self,
        request: Request<EvaluatePolicyRequest>,
    ) -> Result<Response<EvaluatePolicyResponse>, Status> {
        // Tonic handles deserialization, error mapping
        let req = request.into_inner();
        
        // Business logic
        let result = self.engine.evaluate(&req)?;
        
        Ok(Response::new(result))
    }
    
    async fn stream_events(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<impl Stream<Item = Result<Event>>>, Status> {
        // gRPC streaming
        let stream = self.engine.stream_events().await?;
        Ok(Response::new(stream))
    }
}

// main.rs
let grpc_server = tonic::transport::Server::builder()
    .add_service(PolicyServer::new(service))
    .serve(addr)
    .await?;
```

---

## WHY NOT THAT?

### Alternative 1: REST (JSON over HTTP)

**How it works**:
- HTTP POST/GET endpoints
- JSON request/response bodies
- Standard HTTP semantics

**Advantages**:
- **Ubiquitous**: Every language, platform supports HTTP
- **Debuggable**: curl, browser, postman
- **Human-readable**: JSON easy to inspect
- **Firewalls**: HTTP usually allowed
- **Caching**: HTTP caching semantics available
- **Familiar**: Most developers know REST

**Disadvantages**:
- **Verbose**: JSON larger than binary (2-5x)
- **Latency**: Text parsing slower than binary
- **No multiplexing**: HTTP/1.1 one request per connection
- **No streaming**: Server → client streaming difficult (need WebSockets)
- **Schema enforcement**: No built-in schema (must validate manually)
- **Versioning**: API versioning manual (v1/v2 endpoints)

**Complexity**:
- Simpler initially (HTTP well-known)
- But schema management manual

**Performance**:
- Slower due to JSON size and parsing
- Latency: gRPC 10ms vs REST 50ms for same operation

**Scalability**:
- At 10k concurrent, HTTP/1.1 becomes bottleneck (connection limit)
- HTTP/2 better but still less efficient than gRPC

**Maintainability**:
- Easier (REST patterns well-known)
- But schema management manual

**Testability**:
- Easier (can test with curl, browser)

**Operational consequences**:
- Easier monitoring (HTTP standard)
- Larger bandwidth usage (JSON)

**Why NOT this**:
- Performance sensitive (10+ ms latency matters)
- Binary protocol better for performance
- Evidence: gRPC chosen

---

### Alternative 2: GraphQL

**How it works**:
- Query language for APIs
- Client specifies fields needed
- Server returns only requested data

**Advantages**:
- **Flexible queries**: Client specifies fields needed
- **No over-fetching**: Only requested data returned
- **Type system**: GraphQL schema provides types

**Disadvantages**:
- **Complexity**: Query language adds complexity
- **Parsing overhead**: Must parse and validate queries
- **Caching complexity**: Query-based caching hard
- **Subscriptions complexity**: Subscriptions add complexity
- **Security**: Query complexity attacks possible
- **Learning**: GraphQL learning curve

**Complexity**:
- Higher (query parsing, validation)

**Performance**:
- Slower than REST (query parsing overhead)
- Slower than gRPC (not optimized for performance)

**Scalability**:
- Complex query handling at scale

**Maintainability**:
- Harder (schema + resolver complexity)

**Testability**:
- Harder (query validation complex)

**Operational consequences**:
- Query monitoring needed
- Debugging complex queries hard

**Why NOT this**:
- Overkill for SecureAI API
- Performance concern (query parsing overhead)
- Evidence: gRPC chosen (simpler, faster)

---

### Alternative 3: WebSockets

**How it works**:
- Persistent TCP connection
- Bidirectional messaging
- Message framing over socket

**Advantages**:
- **Bidirectional**: Server can push to client
- **Low latency**: Keep-alive connection reduces handshake
- **Streaming**: Natural streaming support

**Disadvantages**:
- **Stateful**: Server must track connections
- **Scaling**: Sticky sessions required
- **Load balancing**: Hard to load balance
- **Debugging**: Custom protocol hard to debug
- **Firewalls**: Some firewalls block WebSocket
- **Complexity**: Manual framing, error handling

**Complexity**:
- Higher (stateful connections, framing)

**Performance**:
- Good for streaming
- But complexity adds overhead

**Scalability**:
- Hard to scale (sticky connections)

**Maintainability**:
- Harder (custom protocol)

**Testability**:
- Harder (need WebSocket client)

**Operational consequences**:
- Connection state to manage
- Load balancer affinity required

**Why NOT this**:
- Request/response model not suited for WebSockets
- gRPC streaming better (standard protocol)
- Evidence: gRPC not WebSocket

---

## TRADEOFF

**Gains**:
- Performance (binary protocol, HTTP/2 multiplexing)
- Type safety (protocol buffers schema)
- Streaming support (bidirectional with standard protocol)
- Efficient serialization (binary format)
- Deadline propagation (gRPC context metadata)

**Sacrifices**:
- Debuggability (binary not human-readable)
- Ubiquity (not every language has gRPC)
- Simplicity (protocol buffers need compilation)
- Ecosystem (fewer tools than HTTP)
- Monitoring (HTTP monitoring tools don't work)

**Net result**: More efficient, but less debuggable.

---

## FAILURE POINT

Becomes problematic when:

1. **Cross-language complexity**: Some languages have poor gRPC support
2. **Debuggability requirement**: Binary format hard to inspect
3. **Network debugging**: tcpdump shows binary (can't see data)
4. **Monitoring**: HTTP monitoring tools don't work with gRPC

---

## CHANGE CONDITION

Would justify replacing this:

1. **Public API**: If API exposed to public, REST easier
2. **Debuggability**: If debugging becomes critical, REST helpful
3. **Ecosystem requirement**: If language has no gRPC support, REST fallback
4. **Performance unimportant**: If latency not critical, REST simpler

---

## SCALE CONDITION

Becomes problematic at:

- **10k concurrent connections**: gRPC handles well
- **Performance monitoring**: Need specialized gRPC monitoring

---

## LEARNING QUESTION

**Question**: A client makes 100 sequential gRPC calls via REST over HTTP/1.1 vs gRPC over HTTP/2. Which is faster and why?

**Answer should demonstrate understanding of**:
- HTTP/1.1 connection pooling (multiple TCP connections)
- HTTP/2 multiplexing (single connection, multiple streams)
- gRPC binary efficiency (smaller messages)
- When each matters (high concurrency vs sequential)

---

# DECISION 9: No Database (Mostly Stateless)

## WHY THIS?

**What it does**:
System is mostly stateless. No user database, no stored policies, no config database. State comes from:
- Configuration file (secureai.toml) loaded at startup
- Audit ledger file (immutable append-only)
- In-memory caches (ephemeral)
- External queue (NATS)
- External auth provider

**Where implemented**:
- `src/policy/config.rs` - Config file loading (startup only)
- `src/audit/persist.rs` - File-backed audit (append-only)
- No traditional database layer
- `main.rs:40-100` - Startup loads config, no DB init

**Problems solved**:
1. **Operational simplicity**: No database to manage, backup, upgrade
2. **Scalability**: Stateless allows horizontal scaling (no session affinity)
3. **Reliability**: No database failure means no system failure
4. **Deployment**: Single binary (no schema migrations)
5. **Cost**: No database infrastructure

**Engineering principle**:
- **Statelessness**: No state persisted in service
- **Immutable infrastructure**: Configuration at startup, not runtime

**Evidence in repository**:
```rust
// src/policy/config.rs
pub struct IsolationPolicy {
    // Loaded from config file at startup
    pub model: String,
    pub enable_sandbox: bool,
    pub enable_audit: bool,
    // ... 20+ fields
}

impl IsolationPolicy {
    pub fn load(path: &str) -> Result<Self> {
        // Load from TOML at startup only
        let content = std::fs::read_to_string(path)?;
        let policy: IsolationPolicy = toml::from_str(&content)?;
        Ok(policy)
    }
}

// src/main.rs
fn main() {
    // Load config at startup (once)
    let config = IsolationPolicy::load("secureai.toml")?;
    
    // Initialize subsystems with config (no database)
    let policy_engine = PolicyEngine::new(config)?;
    
    // No database initialization
    // No schema migrations
    // No connection pooling
    
    // Start server with loaded config
    start_server(policy_engine)?;
}

// src/audit/persist.rs
pub struct FileBackedStore {
    file_path: String,  // Single file, append-only
}

impl FileBackedStore {
    pub async fn write_entry(&self, entry: &AuditEntry) -> Result<()> {
        // Append to file (not insert into database)
        let json = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}
```

---

## WHY NOT THAT?

### Alternative 1: Relational Database (PostgreSQL)

**How it works**:
- Store configuration, audit logs, cache, users in database
- Query API for data access
- Schema migrations for updates

**Advantages**:
- **Queryable**: SQL queries for analytics
- **Transactional**: ACID transactions for consistency
- **Scalable**: Can replicate, shard
- **Durable**: Data survives crashes
- **Runtime updates**: Change config without restart
- **Multi-instance**: Shared state across instances

**Disadvantages**:
- **Operational**: Must manage database (backup, upgrade, patching)
- **Deployment**: Schema migrations required
- **Latency**: Database queries add latency (5-50ms vs in-memory <1ms)
- **Cost**: Database infrastructure
- **Availability**: Database failure = system failure
- **Complexity**: ORM/query complexity
- **Scaling**: Database becomes bottleneck at high load

**Complexity**:
- Much higher (schema design, migration management)

**Performance**:
- Slower (disk I/O vs memory)
- Query latency overhead

**Scalability**:
- Database becomes bottleneck
- Sharding complexity

**Maintainability**:
- Harder (schema changes complex)
- Database expertise needed

**Testability**:
- Harder (need test database)

**Operational consequences**:
- Database maintenance required
- Backup/restore critical
- Monitoring database health
- Schema versioning

**Why NOT this**:
- Operational burden not justified
- No need for runtime updates (restart acceptable)
- No need for ACID transactions (audit is append-only)
- Evidence: No database layer exists

---

### Alternative 2: Key-Value Store (Redis/Memcached)

**How it works**:
- Store configuration, cache, sessions in Redis
- No persistence (or optional)

**Advantages**:
- **Fast**: In-memory access
- **Simple**: No schema
- **Runtime updates**: Can update config at runtime
- **Distributed**: Redis cluster available

**Disadvantages**:
- **Not durable**: Loss of data on crash
- **Limited**: No transactions, no joins
- **Management**: Redis cluster management
- **Cost**: Redis infrastructure
- **Complexity**: Cache invalidation hard

**Complexity**:
- Moderate (easier than database)

**Performance**:
- Fast (in-memory)

**Scalability**:
- Can scale but complexity increases

**Maintainability**:
- Easier than database

**Testability**:
- Easier (in-memory)

**Operational consequences**:
- Redis cluster management
- No persistence (acceptable for cache but not audit)

**Why NOT this**:
- Audit ledger must be durable (can't use Redis)
- Configuration file sufficient (no runtime updates needed)
- Evidence: File-backed audit used

---

### Alternative 3: Hybrid (PostgreSQL + File Audit)

**How it works**:
- Store configuration in PostgreSQL
- Keep audit file (current approach)
- Best of both

**Advantages**:
- **Queryable config**: SQL queries on config
- **Durable audit**: Audit immutable file
- **Runtime updates**: Can change config without restart
- **Transactional**: Config updates atomic

**Disadvantages**:
- **Operational**: Still need database
- **Complexity**: Multiple storage backends
- **Scaling**: Database still bottleneck
- **Deployment**: Schema migrations still needed
- **Cost**: Database infrastructure

**Complexity**:
- Higher (two storage systems)

**Performance**:
- Config queries add latency

**Scalability**:
- Database still bottleneck

**Maintainability**:
- Harder (two systems to manage)

**Testability**:
- Harder (need test database)

**Operational consequences**:
- Database management required
- Config consistency (if different instances update config)

**Why NOT this**:
- Configuration file sufficient
- No runtime updates needed
- Database operational overhead not justified
- Evidence: Pure file-based config chosen

---

## TRADEOFF

**Gains**:
- Operational simplicity (no database)
- Low latency (no database queries)
- Easy scaling (no state affinity required)
- Fault isolation (database failure doesn't affect system)
- Deployment simplicity (single binary)
- Cost reduction (no database infrastructure)

**Sacrifices**:
- No queryable storage (analytics need post-processing)
- No runtime updates (restart required for config changes)
- No ACID transactions (audit is fire-and-forget)
- No multi-instance coordination (each instance independent)
- Limited debugging (can't query audit logs via SQL)
- Audit queries slower (must read file)

**Net result**: Simpler and more reliable, but less flexible.

---

## FAILURE POINT

Becomes problematic when:

1. **Runtime config changes**: If config must change without restart
2. **Audit analysis**: If need complex queries on audit logs
3. **Multi-instance**: If multiple instances need shared state
4. **Compliance**: If regulator requires queryable audit database

---

## CHANGE CONDITION

Would justify replacing this:

1. **Runtime updates required**: If config must change at runtime, add database
2. **Audit analytics**: If need to query audit logs by SQL, add database
3. **Distributed**: If need shared state across regions, database needed
4. **Feature evolution**: If system grows to need transactional state

---

## SCALE CONDITION

Becomes problematic at:

- **Audit log size**: At 1TB+ of audit logs, file queries slow
- **Configuration complexity**: At 1000+ config options, file harder to manage
- **Multi-region**: Need database for cross-region consistency

---

## LEARNING QUESTION

**Question**: The system loads configuration at startup from file. A user wants to change an auth provider without restarting. Why can't this be done, and what would need to change?

**Answer should demonstrate understanding of**:
- Stateless architecture (config loaded at startup)
- Why restart required (config not reloadable)
- Design tradeoff (simplicity vs flexibility)
- What would be needed (watch file, hot reload, or database)

---

[Continued in next section - 5 more major decisions]

---

# DECISION 10: Fail-Secure Error Handling (Explicit Denials)

## WHY THIS?

**What it does**:
On any error, system denies access explicitly. Failed validation → request rejected. Failed authorization → 403. Failed sandbox → task rejected. No fallback or silent degradation.

**Where implemented**:
- `src/policy/mod.rs:PolicyEngine::validate_task()` - Returns error, not default
- `src/auth/jwt.rs` - Invalid token → 401 Unauthenticated
- `src/sandbox/mod.rs` - Sandbox failure → request fails
- `src/api/grpc.rs` - Error converted to gRPC Status (Unauthenticated, PermissionDenied)

**Problems solved**:
1. **Security**: No implicit allow (fail-open vulnerabilities)
2. **Clarity**: Error means denial, not partial success
3. **Compliance**: Audit trail shows denials
4. **Debuggability**: Failures explicit and logged

**Engineering principle**:
- **Fail-secure**: Default deny, explicit allow
- **No magic**: No silent degradation

**Evidence in repository**:
```rust
// src/policy/mod.rs
impl PolicyEngine {
    pub async fn validate_task(&self, req: &Request) -> Result<ValidationResult> {
        // 1. Check model allowed
        if !self.allowed_models.contains(&req.model) {
            return Err(PolicyError::ModelNotAllowed(req.model.clone()));
        }
        
        // 2. Check paths allowed
        if req.input_path.is_some() {
            let path = req.input_path.as_ref().unwrap();
            if !self.allowed_paths.iter().any(|p| path.starts_with(p)) {
                return Err(PolicyError::PathNotAllowed(path.clone()));
            }
        }
        
        // All checks must pass (implicit allow on OK)
        Ok(ValidationResult {
            model: req.model.clone(),
            paths: req.input_path.clone(),
        })
    }
}

// src/auth/jwt.rs
pub fn validate_token(&self, token: &str) -> Result<Claims> {
    // Invalid token → error (not default claims)
    let claims = decode::<Claims>(token, &self.public_key)?;
    
    if claims.exp < now() {
        return Err(JwtError::TokenExpired);
    }
    
    Ok(claims)
}

// src/api/grpc.rs
async fn evaluate_policy(&self, request: Request) -> Result<Response, Status> {
    // Any error → explicit Status returned
    let auth = self.authenticate(&request)
        .map_err(|_| Status::unauthenticated("invalid token"))?;
    
    let validation = self.validate(&request)
        .map_err(|e| Status::permission_denied(e.to_string()))?;
    
    Ok(Response::new(...))
}
```

---

## WHY NOT THAT?

### Alternative 1: Fail-Open with Fallback

**How it works**:
- If validation fails, allow with reduced privileges
- If cache misses, compute (instead of denying)
- If auth fails, proceed as unauthenticated
- Graceful degradation

**Advantages**:
- **User experience**: System doesn't deny due to failures
- **Availability**: Continues operating on failures
- **Resilience**: Doesn't require perfect uptime

**Disadvantages**:
- **Security risk**: Unintended access possible
- **Silent failures**: Operator doesn't know failure happened
- **Compliance violation**: Audit shows access, not why
- **Debugging hard**: Silent failures hard to diagnose

**Complexity**:
- Higher (fallback logic complex)

**Performance**:
- Better (no denials)

**Scalability**:
- Same

**Maintainability**:
- Harder (fallback behavior complex)

**Testability**:
- Harder (many paths to test)

**Operational consequences**:
- Silent failures (bad for security)
- Compliance issue (access log shows access but auth actually failed)

**Why NOT this**:
- Security violation (implicit allow)
- Not suitable for security system
- Evidence: Explicit errors returned

---

### Alternative 2: Partial Degradation

**How it works**:
- Some failures allow reduced functionality
- Cache miss → degrade to lower quality
- Secondary service down → use primary only

**Advantages**:
- **Availability**: System partially available on failures
- **User experience**: Some functionality on degradation

**Disadvantages**:
- **Complex**: Many degradation paths to test
- **Unclear contract**: API behavior differs on failures
- **Compliance**: Audit shows degraded behavior
- **Security**: Reduced security on failures

**Complexity**:
- Higher (degradation paths)

**Performance**:
- Variable (degraded is slower)

**Scalability**:
- Same

**Maintainability**:
- Harder (many behaviors)

**Testability**:
- Harder (many paths)

**Operational consequences**:
- Unclear API behavior
- Degradation hard to monitor

**Why NOT this**:
- Unclear and hard to test
- Evidence: Explicit failure chosen

---

## TRADEOFF

**Gains**:
- Security (no implicit allow)
- Clarity (error means denial)
- Compliance (explicit denials in audit)
- Debuggability (failures visible)

**Sacrifices**:
- User experience (requests denied on failures)
- Availability (single failure cascades)
- Operational complexity (must handle all edge cases)

**Net result**: More secure but less forgiving.

---

## FAILURE POINT

Becomes problematic when:

1. **Cascading failures**: Single failure causes many denials
2. **Configuration errors**: Wrong config blocks legitimate users
3. **Transient failures**: Temporary outages deny access

---

## CHANGE CONDITION

Would justify replacing this:

1. **Availability SLA**: If 99.99% uptime required, need graceful degradation
2. **User experience**: If too many legitimate rejections, soften errors
3. **Cost**: If reducing failures expensive, some degradation acceptable

---

## LEARNING QUESTION

**Question**: If the authentication server is unreachable (network down), should SecureAI:
a) Deny all requests (fail-secure)
b) Allow requests as unauthenticated (fail-open)
c) Allow only cached tokens (partial degradation)

Discuss tradeoffs.

**Answer should demonstrate understanding of**:
- Fail-secure principle (default deny)
- Security vs availability tradeoff
- Compliance implications (each choice)
- Operator burden (each choice)

---

[End of comprehensive deep-dive. Remaining decisions follow similar structure]

---

# Summary of Major Decisions

This document analyzed 10 significant architectural decisions:

1. ✅ **Firecracker MicroVMs** - Why not containers/threads
2. ✅ **Opt-In Features** - Why not monolithic
3. ✅ **OAuth2/OIDC Auth** - Why not in-house
4. ✅ **Ed25519 Signatures** - Why not HMAC/RSA
5. ✅ **NATS JetStream Queue** - Why not Redis/SQS/Kafka
6. ✅ **Two-Tier Cache** - Why not exact-only/semantic-only
7. ✅ **Async/Await (Tokio)** - Why not threads/event loop
8. ✅ **gRPC API** - Why not REST/GraphQL
9. ✅ **No Database** - Why not PostgreSQL/Redis
10. ✅ **Fail-Secure** - Why not fail-open

Each analyzed using framework:
- **WHY THIS?** (what, where, problems, principles, evidence)
- **WHY NOT THAT?** (alternatives with detailed tradeoffs)
- **TRADEOFF** (gains vs sacrifices)
- **FAILURE POINT** (when problematic)
- **CHANGE CONDITION** (when to replace)
- **SCALE CONDITION** (scale limits)
- **LEARNING QUESTION** (test understanding)

---

# Pattern Recognition

**Common themes across decisions**:

1. **Security-first**: Prefer secure even if slower (MicroVM over containers, fail-secure over fail-open, Ed25519 over HMAC)

2. **Operational simplicity**: Prefer stateless, no databases, no complex clusters (file-based config, opt-in features, no database)

3. **Performance efficiency**: Prefer async/await, caching, efficient protocols (Tokio, two-tier cache, gRPC)

4. **Pragmatism**: Accept complexity when justified (two-tier cache vs exact-only) but avoid unnecessary (QEMU vs Firecracker)

5. **Separation of concerns**: Delegate expertise (OAuth to provider, queuing to NATS), don't build in-house

6. **Non-breaking evolution**: Features can be added/removed without affecting others (opt-in architecture)

---

# Questions for Further Analysis

These decisions warrant deeper investigation:

1. **Private key management**: Where is Ed25519 private key stored? TPM? Filesystem?
2. **NATS availability**: What happens if NATS cluster fails? Tasks lost?
3. **Cache consistency**: If Tier 1 and Tier 2 disagree, which wins?
4. **Configuration validation**: What prevents invalid config from breaking system?
5. **Guardrails accuracy**: How is semantic threat detection threshold determined?
6. **Audit log retention**: Is there a rotation/archival policy?
7. **Multi-region**: How would this scale across regions?
8. **Cost model**: What's the cost per request (VM startup, embeddings, audit)?

---

**Last updated**: 2026-08-14

