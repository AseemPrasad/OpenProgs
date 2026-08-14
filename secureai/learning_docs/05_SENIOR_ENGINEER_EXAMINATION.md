# Senior Engineer Examination: SecureAI MVP

**Comprehensive assessment of your understanding**

This document contains the complete examination framework. Answer each question, then self-evaluate using the provided rubric. Document your reasoning and any follow-up questions.

---

## How to Use This Examination

1. **Read the question carefully** - What is actually being asked?
2. **Answer completely** - Show your reasoning, not just a conclusion
3. **Reference the code** - Point to specific files/functions when relevant
4. **Be specific** - "It's a queue" is not sufficient; explain *why* NATS was chosen
5. **Self-score** - Use the rubric after each question
6. **Document everything** - Write your answers in the space provided

---

## Scoring Rubric

**10/10 - Expert**
- Complete, accurate answer
- Demonstrates deep understanding
- Identifies edge cases/implications
- Connects to broader system
- No missing concepts

**8-9/10 - Advanced**
- Mostly correct and complete
- Minor gaps or imprecision
- Good reasoning
- Could explain to a colleague

**6-7/10 - Competent**
- Correct main points
- Some missing depth
- Basic reasoning present
- Could implement with guidance

**4-5/10 - Partial**
- Mostly correct but significant gaps
- Misses key implications
- Some incorrect assumptions
- Would need review before shipping

**2-3/10 - Weak**
- Correct on surface
- Major misunderstandings
- Missing key concepts
- Would fail code review

**0-1/10 - Insufficient**
- Incorrect answer
- Fundamental misunderstanding
- Would require teaching before proceeding

---

# LEVEL 1: Repository Structure

## Question 1.1: System Overview

**Question:**
Without looking at diagrams, describe what SecureAI does in 2-3 sentences. Then name the 10 major features and which 5 are critical (must-have) vs optional (nice-to-have).

**Your Answer:**
```
[Write your answer here]
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Accurate system purpose | ☐ Full ☐ Partial ☐ Missing |
| All 10 features named | ☐ Full ☐ Partial ☐ Missing |
| Correct critical/optional split | ☐ Full ☐ Partial ☐ Missing |
| Understanding of why split | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- System purpose: "Enterprise AI task execution platform with multi-layered security"
- Must-have features (cannot run without): Sandbox, Auth, Guardrails, Audit, gRPC API
- Optional features (nice-to-have): Cache, Queue, Evals, Proxy, Telemetry
- Reasoning: Why are some critical? (Because they're on security path)

**Key Misconceptions to Avoid:**
- ❌ Thinking all 10 features are equally critical
- ❌ Confusing features with modules
- ❌ Not understanding why certain features are mandatory

**If You Missed Anything:**
- What's the difference between critical and optional in your codebase?
- Can the system run without authentication? Why/why not?
- Which feature would you remove first if scaling down?

**Your Score: ___ / 10**

**Follow-Up Question (if needed):**
[Will be based on your answer]

---

## Question 1.2: Filesystem Organization

**Question:**
Look at `src/` directory structure. Name the 12 core modules and their ONE-sentence responsibility. Then, identify which module would handle each of these scenarios:
- a) Checking if user has permission to execute a task
- b) Storing a cryptographic proof that an action happened
- c) Retrieving a previously computed response to the same prompt
- d) Preventing dangerous patterns in user input

**Your Answer:**
```
12 Modules:
1. [name]: [responsibility]
2. [name]: [responsibility]
... (complete all 12)

Scenario assignments:
a) Permission check → _____ module
b) Proof of action → _____ module
c) Previous response → _____ module
d) Pattern prevention → _____ module
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct module names (12/12) | ☐ Full ☐ 10-11 ☐ 8-9 ☐ <8 |
| Accurate one-sentence summaries | ☐ All ☐ Most ☐ Some ☐ None |
| Correct scenario mappings (4/4) | ☐ All ☐ 3/4 ☐ 2/4 ☐ <2 |
| Shows module understanding | ☐ Deep ☐ Good ☐ Shallow ☐ Missing |

**What I'm Looking For:**
- Modules: sandbox, audit, auth, guardrails, queue, cache, evals, proxy, api, telemetry, policy, identity
- a) auth (specifically RBAC)
- b) audit
- c) cache
- d) guardrails

**Key Misconceptions:**
- ❌ Confusing policy with individual feature modules
- ❌ Thinking queue is for caching
- ❌ Not knowing what evals/proxy do

**If You Got Lost:**
- Open `src/` and list directories
- For each scenario, where would YOU put that code? Why?
- What's the difference between policy (orchestrator) and individual modules?

**Your Score: ___ / 10**

---

## Question 1.3: Configuration

**Question:**
Open `secureai.toml`. What does each of these config sections control?
- [model]
- [sandbox]
- [auth]
- [guardrails]
- [audit]

Then, explain: **Why is configuration separate from code?** What would break if these settings were hardcoded?

**Your Answer:**
```
[model]:
[sandbox]:
[auth]:
[guardrails]:
[audit]:

Why separate from code?
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct config section purposes | ☐ All 5 ☐ 4/5 ☐ 3/5 ☐ <3 |
| Understands why separate | ☐ Deep ☐ Partial ☐ Surface ☐ Missing |
| Identifies breakage risk | ☐ Full ☐ Partial ☐ None |

**What I'm Looking For:**
- Configuration separation = deployment flexibility
- If hardcoded: Can't change behavior without recompile, can't test different scenarios, operationally inflexible
- Shows understanding of 12-factor app principles

**Key Misconceptions:**
- ❌ Thinking config is just nice-to-have
- ❌ Not understanding that different deployments need different config

**Your Score: ___ / 10**

---

# LEVEL 2: Components

## Question 2.1: Sandbox Execution

**Question:**
Explain the sandbox execution path from user input to result. Specifically:
- Where does the sandbox create a new VM?
- What three security layers are applied and in what order?
- Why is each layer necessary?
- What would happen if we skipped the seccomp layer?

**Your Answer:**
```
Execution path:
1. [step]
2. [step]
...

Three security layers:
1. [name] - Why necessary:
2. [name] - Why necessary:
3. [name] - Why necessary:

If seccomp skipped:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct execution flow | ☐ Full ☐ 80% ☐ 60% ☐ <60% |
| All 3 layers identified | ☐ All ☐ 2/3 ☐ 1/3 ☐ 0/3 |
| Can explain why each | ☐ All ☐ Most ☐ Some ☐ None |
| Understands layer failure | ☐ Full ☐ Partial ☐ None |

**What I'm Looking For:**
- Three layers: Firecracker (kernel isolation), Landlock (FS access), seccomp (syscall filtering)
- Each provides defense-in-depth
- Without seccomp: Legitimate syscalls might cause trouble, but less protection
- Shows understanding of "defense in depth" principle

**Code to reference:**
- `src/sandbox/mod.rs` - SandboxManager::spawn_vm()
- `src/sandbox/seccomp.rs` - apply_filter()
- `src/sandbox/landlock.rs` - apply_policy()
- `src/sandbox/cgroups.rs` - apply_limits()

**Key Misconceptions:**
- ❌ Thinking Firecracker is just one layer
- ❌ Not understanding why multiple layers needed
- ❌ Confusing seccomp with Landlock (different purposes)

**Your Score: ___ / 10**

---

## Question 2.2: Authentication & Authorization

**Question:**
Trace a gRPC request through the authentication system:
- Where does JWT validation happen?
- What happens if token is invalid?
- How does JWKS caching work? Why cache instead of fetching every time?
- What's the difference between authentication and authorization in this system?

**Your Answer:**
```
JWT validation location:
Invalid token handling:
JWKS caching mechanism:
Why cache:
Authentication vs Authorization:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct validation location | ☐ Correct ☐ Partial ☐ Wrong |
| Understands invalid handling | ☐ Full ☐ Partial ☐ Missing |
| Explains caching correctly | ☐ Full ☐ Partial ☐ Missing |
| Distinguishes Auth/AuthZ | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- JWT validation in gRPC middleware (before request reaches service)
- Invalid → 401 Unauthenticated error
- JWKS caching: Fetch public keys from provider, cache with TTL (1 hour)
- Why cache: Avoid repeated provider calls, reduce latency
- Authentication = proving identity (JWT), Authorization = checking permissions (RBAC)

**Code to reference:**
- `src/auth/jwt.rs` - JwtValidator::validate_token()
- `src/auth/jwks.rs` - JwksCache
- `src/api/grpc.rs` - gRPC middleware
- `src/auth/rbac.rs` - Permission checking

**Key Misconceptions:**
- ❌ Thinking JWKS cached indefinitely (no, TTL of 1h)
- ❌ Confusing JWT validation with permission checking
- ❌ Not understanding middleware pattern

**Your Score: ___ / 10**

---

## Question 2.3: Audit Ledger

**Question:**
The audit ledger stores every action. Explain:
- What does each audit entry contain?
- Why are entries signed with Ed25519 (asymmetric) instead of HMAC (symmetric)?
- How does the hash chain prevent tampering?
- If someone modifies an entry on disk, what gets detected and what doesn't?

**Your Answer:**
```
Audit entry contents:
Ed25519 vs HMAC reasoning:
Hash chain tampering prevention:
If entry modified:
  - Detected:
  - Not detected:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Complete entry structure | ☐ Full ☐ Partial ☐ Missing |
| Explains signing choice | ☐ Full ☐ Partial ☐ Missing |
| Understands hash chain | ☐ Full ☐ Partial ☐ Missing |
| Predicts tampering detection | ☐ Full ☐ Partial ☐ Partial ☐ Missing |

**What I'm Looking For:**
- Entry: id, timestamp, action, subject, details, hash, signature
- Ed25519: Proves *which key* signed (non-repudiation); HMAC only proves *someone with key*
- Hash chain: Each entry includes SHA256(prev_hash || entry_data); tampering breaks chain
- If modified: Signature fails verification (entry's signature invalid), hash chain breaks (next entry's hash doesn't match), BUT original entry hash/sig undetected (need to know original value)

**Code to reference:**
- `src/audit/ledger.rs` - AuditEntry, append_entry()
- `src/audit/keys.rs` - Ed25519 key management
- `src/audit/verify.rs` - verify_chain()

**Key Misconceptions:**
- ❌ Thinking Ed25519 signature alone proves tampering (it doesn't unless verified)
- ❌ Not understanding that hash chain is also part of defense
- ❌ Confusing what "tamper detection" means (detect attempt, not prevent)

**Your Score: ___ / 10**

---

# LEVEL 3: Data Flow

## Question 3.1: Task Execution Request Flow

**Question:**
A user submits a task via CLI: `secureai run "What is 2+2?" --model llama3`. 

Trace the complete data flow:
1. Where does the prompt go first?
2. What checks happen and in what order?
3. What caches are consulted?
4. If everything passes, where does execution happen?
5. Where is the result stored/returned?

**Your Answer:**
```
Prompt entry point:
Checks and order:
1. [check]
2. [check]
...

Caches consulted:
1. [cache]
2. [cache]
...

Execution location:
Result storage:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct entry point | ☐ Correct ☐ Partial ☐ Wrong |
| All checks identified | ☐ All ☐ Most ☐ Some ☐ None |
| Correct check order | ☐ Full ☐ Partial ☐ Wrong |
| Cache identification | ☐ All ☐ Most ☐ Some ☐ None |
| Complete execution path | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
1. Entry: `main.rs` → CLI parser → Commands::Run handler
2. Checks and order:
   - Authentication (if enabled)
   - Policy validation (model allowed, paths allowed)
   - Guardrails check (semantic threat detection)
   - Cache lookup (Tier 1 exact, Tier 2 semantic)
3. Caches: ExactCache, SemanticCache
4. Execution: SandboxManager::spawn_vm() with Firecracker
5. Result: Printed to stdout, logged to audit ledger

**Code to reference:**
- `src/main.rs:62-204` - Commands::Run handler
- `src/policy/mod.rs` - PolicyEngine coordination
- `src/cache/mod.rs` - Two-tier cache
- `src/sandbox/mod.rs` - Execution

**Key Misconceptions:**
- ❌ Thinking cache is checked before validation
- ❌ Not knowing order of security checks
- ❌ Confusing which cache is consulted

**Your Score: ___ / 10**

---

## Question 3.2: Cache Hit vs Miss Path

**Question:**
Two identical prompts are submitted:
- Prompt A at 10:00am (new)
- Prompt A at 10:05am (repeat)

What happens in each scenario?
- Tier 1 exact cache
- Tier 2 semantic cache
- Latency difference

Then ask yourself: Why have two tiers? What would change if we only had Tier 1?

**Your Answer:**
```
First submission (10:00):
- Tier 1: [what happens]
- Tier 2: [what happens]
- Latency: [expected]

Second submission (10:05):
- Tier 1: [what happens]
- Tier 2: [what happens]
- Latency: [expected]

Why two tiers?
If only Tier 1:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Tier 1 behavior (first) | ☐ Correct ☐ Partial ☐ Wrong |
| Tier 2 behavior (first) | ☐ Correct ☐ Partial ☐ Wrong |
| Tier 1 behavior (second) | ☐ Correct ☐ Partial ☐ Wrong |
| Tier 2 behavior (second) | ☐ Correct ☐ Partial ☐ Wrong |
| Explains two-tier value | ☐ Full ☐ Partial ☐ Missing |
| Understands tradeoffs | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- First: Both miss, compute, cache in both tiers, latency ~5 seconds
- Second: Tier 1 hits (hash match), return cached result, latency ~1ms
- Why two tiers: Exact match is fast (1ms), but similar prompts also valuable (~80% semantic hit vs ~30% exact-only)
- If only Tier 1: Much lower hit rate (~30%), more computation

**Code to reference:**
- `src/cache/exact.rs` - ExactCache (O(1) hash lookup)
- `src/cache/semantic.rs` - SemanticCache (O(n) similarity search)
- `src/cache/mod.rs` - CacheManager::get_or_compute()

**Key Misconceptions:**
- ❌ Thinking both tiers are checked simultaneously
- ❌ Not understanding hash-based matching
- ❌ Confusing latency impact (1ms vs 5000ms is huge)

**Your Score: ___ / 10**

---

# LEVEL 4: Runtime Behavior

## Question 4.1: Concurrent Requests

**Question:**
Suppose 100 users submit 100 different tasks simultaneously. Using your knowledge of async/await and Tokio:
- How many OS threads run these tasks? (Exact number or reasoning)
- Can one slow task block other tasks?
- If the embedding model is slow (1 second per call), what happens?
- If the audit ledger write is slow (100ms), what happens?

**Your Answer:**
```
OS threads:
Can slow task block others?
Slow embedding model:
Slow audit write:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Thread count reasoning | ☐ Correct ☐ Partial ☐ Wrong |
| Understands non-blocking | ☐ Full ☐ Partial ☐ Missing |
| Explains embedding impact | ☐ Full ☐ Partial ☐ Missing |
| Explains audit impact | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- OS threads: Few (Tokio default is 2 × CPU cores, typically 8-16)
- Can slow task block? No, async doesn't block event loop on I/O, BUT if blocking call used, yes
- Slow embedding: Call is spawn_blocking() (separate thread pool), doesn't block async tasks
- Slow audit write: Fire-and-forget (async write), doesn't block request return

**Code to reference:**
- `src/main.rs` - #[tokio::main] runtime initialization
- `src/cache/semantic.rs` - embed_prompt() uses tokio::task::spawn_blocking()
- `src/audit/hooks.rs` - Async logging without blocking

**Key Misconceptions:**
- ❌ Thinking 100 threads for 100 concurrent requests
- ❌ Not understanding spawn_blocking() purpose
- ❌ Confusing async/await with parallelism

**Your Score: ___ / 10**

---

## Question 4.2: Error Handling

**Question:**
Three scenarios with errors:

1. User provides invalid JWT token
2. Sandbox execution crashes
3. Audit ledger write fails

For each, answer:
- Does the request succeed or fail?
- What does user see?
- Is error logged?
- Is audit entry created?

**Your Answer:**
```
Scenario 1 (Invalid JWT):
- Request succeeds/fails?
- User sees:
- Error logged?
- Audit entry:

Scenario 2 (Sandbox crash):
- Request succeeds/fails?
- User sees:
- Error logged?
- Audit entry:

Scenario 3 (Audit write fails):
- Request succeeds/fails?
- User sees:
- Error logged?
- Audit entry:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Scenario 1 (JWT) | ☐ All correct ☐ 2/3 ☐ 1/3 ☐ Wrong |
| Scenario 2 (Sandbox) | ☐ All correct ☐ 2/3 ☐ 1/3 ☐ Wrong |
| Scenario 3 (Audit) | ☐ All correct ☐ 2/3 ☐ 1/3 ☐ Wrong |
| Shows fail-secure understanding | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
1. JWT: Fails (401), user sees error, logged, no audit entry (auth failed before audit)
2. Sandbox: Fails, user sees error, logged, audit might partially write
3. Audit: Succeeds (execution already done), error logged (but silent to user), audit fail is non-fatal

Shows understanding of fail-secure design and error propagation.

**Code to reference:**
- `src/api/grpc.rs` - Error handling and Status mapping
- `src/sandbox/mod.rs` - Failure handling
- `src/audit/hooks.rs` - Non-blocking audit

**Key Misconceptions:**
- ❌ Thinking audit failure fails the request (it shouldn't)
- ❌ Not understanding fail-secure (invalid token should deny)
- ❌ Confusing log level with request failure

**Your Score: ___ / 10**

---

# LEVEL 5: Design Patterns

## Question 5.1: Command Pattern

**Question:**
In `src/main.rs`, the CLI uses Clap with `enum Commands { Run, ... }`. Explain:
- What design pattern is this?
- Why use enum instead of separate functions?
- How does this enable opt-in features?
- What would break if you added a new command but forgot to handle it?

**Your Answer:**
```
Design pattern:
Why enum vs separate functions:
Enables opt-in features:
If command unhandled:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct pattern identification | ☐ Correct ☐ Partial ☐ Wrong |
| Understands enum advantage | ☐ Full ☐ Partial ☐ Missing |
| Connects to opt-in features | ☐ Full ☐ Partial ☐ Missing |
| Predicts failure mode | ☐ Correct ☐ Partial ☐ Wrong |

**What I'm Looking For:**
- Pattern: Command pattern (encapsulates request as object)
- Enum: Type-safe, compiler checks all cases handled
- Opt-in: Each command can conditionally initialize features
- Unhandled: Compiler error (non-exhaustive match) forces handling

**Code to reference:**
- `src/main.rs:50-70` - CLI enum and matching

**Key Misconceptions:**
- ❌ Not recognizing command pattern
- ❌ Not understanding rust enum exhaustiveness

**Your Score: ___ / 10**

---

## Question 5.2: Trait Pattern for Components

**Question:**
Many components use traits (AuthContext, CachedResponse, AuditEntry). Instead of using concrete types, why use traits? What's the benefit?

Then ask: If you wanted to add a new cache implementation (DatabaseCache), what would you need to do?

**Your Answer:**
```
Why traits instead of concrete types:
Benefits:
Coupling reduction:

Adding new cache implementation (DatabaseCache):
1. [step]
2. [step]
...
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Understands trait purpose | ☐ Full ☐ Partial ☐ Missing |
| Identifies benefits | ☐ Multiple ☐ Some ☐ None |
| Understands loose coupling | ☐ Full ☐ Partial ☐ Missing |
| Can add new implementation | ☐ Clear ☐ Partial ☐ Confused |

**What I'm Looking For:**
- Traits: Abstraction, loose coupling, testing, swappability
- Benefits: Can swap implementations, test with mocks, add features without changing existing code
- DatabaseCache: Implement trait, no changes to callers (polymorphism)

**Key Misconceptions:**
- ❌ Not understanding loose coupling value
- ❌ Thinking you'd need to modify PolicyEngine to add new cache

**Your Score: ___ / 10**

---

# LEVEL 6: Architecture Decisions

## Question 6.1: Why Firecracker, Not Containers

**Question:**
Explain why SecureAI uses Firecracker MicroVMs instead of Docker containers for task isolation. Your answer must include:
- Security difference
- Performance difference
- When would Docker be better?
- When would full VMs (QEMU) be better?
- Why is this the right choice for MVP?

**Your Answer:**
```
Security difference:
Performance difference:
Docker would be better when:
QEMU would be better when:
Why right for MVP:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Accurate security comparison | ☐ Full ☐ Partial ☐ Wrong |
| Accurate performance comparison | ☐ Full ☐ Partial ☐ Wrong |
| Realistic Docker scenario | ☐ Relevant ☐ Partial ☐ Unrealistic |
| Realistic QEMU scenario | ☐ Relevant ☐ Partial ☐ Unrealistic |
| Understands MVP tradeoff | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- Security: Firecracker has kernel boundary (escape harder); containers share kernel (easier escape)
- Performance: Containers 100-200ms startup (faster); Firecracker 500-1000ms (slower)
- Docker better: High throughput, untrusted code acceptable, performance critical
- QEMU better: Extreme isolation needed, performance acceptable, multi-OS support
- MVP: Security > performance, kernel isolation non-negotiable, Firecracker proven (AWS Lambda)

**Code to reference:**
- `src/sandbox/mod.rs` - Firecracker usage
- Architecture decisions document (04_ARCHITECTURAL_DECISIONS_DEEP_ANALYSIS.md)

**Key Misconceptions:**
- ❌ Thinking containers equally secure (they're not)
- ❌ Not understanding performance tradeoff
- ❌ Not knowing AWS Lambda uses Firecracker

**Your Score: ___ / 10**

---

## Question 6.2: Why OAuth2, Not In-House Auth

**Question:**
SecureAI delegates authentication to OAuth2/OIDC provider. Justify this decision:
- What does SecureAI avoid by delegating?
- What does it gain?
- What's the risk?
- When would in-house auth be necessary?

**Your Answer:**
```
Avoids:
Gains:
Risk:
In-house auth necessary when:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Lists what's avoided | ☐ Multiple ☐ Some ☐ None |
| Lists what's gained | ☐ Multiple ☐ Some ☐ None |
| Identifies real risk | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Realistic scenario | ☐ Relevant ☐ Partial ☐ Unrealistic |

**What I'm Looking For:**
- Avoids: Password storage, session management, key rotation, MFA, password reset complexity
- Gains: Compliance (provider handles), multi-org support, security (delegated to expert)
- Risk: Provider outage affects auth, vendor lock-in, limited customization
- In-house when: Offline required, extreme customization, cost concern, regulatory prohibition

**Key Misconceptions:**
- ❌ Not understanding delegation value
- ❌ Downplaying security complexity of passwords
- ❌ Not considering provider risks

**Your Score: ___ / 10**

---

# LEVEL 7: Tradeoffs

## Question 7.1: Two-Tier Cache Tradeoff

**Question:**
Analyze the two-tier cache decision:
- What's gained by having Tier 2 (semantic)?
- What's sacrificed?
- What if we only had Tier 1 (exact)?
- What if we only had Tier 2 (semantic)?
- At what scale does Tier 2 become problematic?

**Your Answer:**
```
Tier 2 gains:
Tier 2 sacrifices:

Tier 1 only:
- Hit rate:
- Latency:
- Complexity:

Tier 2 only:
- Hit rate:
- Latency:
- Complexity:

Scale problem:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Identifies real gains | ☐ Full ☐ Partial ☐ Missing |
| Identifies real sacrifices | ☐ Full ☐ Partial ☐ Missing |
| Understands Tier 1 only impact | ☐ Full ☐ Partial ☐ Missing |
| Understands Tier 2 only impact | ☐ Full ☐ Partial ☐ Missing |
| Identifies scale limitation | ☐ Correct ☐ Partial ☐ Wrong |

**What I'm Looking For:**
- Tier 2 gains: 60-80% hit rate vs 20-30% (huge difference), semantic understanding
- Tier 2 sacrifices: Complexity (embeddings, similarity search), latency (O(n) search)
- Tier 1 only: 20-30% hit rate, minimal complexity, but limited value
- Tier 2 only: Higher hit rate, but latency always ~60ms (no fast path)
- Scale: At 100k+ cached prompts, O(n) search becomes bottleneck

**Code to reference:**
- `src/cache/mod.rs` - Two-tier coordination
- Performance implications

**Key Misconceptions:**
- ❌ Not understanding hit rate vs latency tradeoff
- ❌ Thinking both tiers necessary (each provides value independently)
- ❌ Not predicting scale problems

**Your Score: ___ / 10**

---

## Question 7.2: Async vs Threads Tradeoff

**Question:**
SecureAI uses async/await (Tokio) instead of thread per request. Compare:

| Attribute | Async/Await | Thread Per Request |
|-----------|------------|-------------------|
| Concurrency at 10k requests | ? | ? |
| Memory per concurrent | ? | ? |
| Code complexity | ? | ? |
| Debugging | ? | ? |
| Blocking I/O | ? | ? |

Fill in the table, then answer: When would you switch to threads?

**Your Answer:**
```
[Fill in the table]

Switch to threads when:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct async row | ☐ All 5 ☐ 4/5 ☐ 3/5 ☐ <3 |
| Correct thread row | ☐ All 5 ☐ 4/5 ☐ 3/5 ☐ <3 |
| Realistic switch scenario | ☐ Relevant ☐ Partial ☐ Unrealistic |

**What I'm Looking For:**
- Async: Handles 10k well, ~100 bytes, complex, hard to debug, non-blocking I/O works great
- Threads: Can't handle 10k well, ~1MB per thread, simple, easy to debug, blocking I/O simple
- Switch: If team doesn't know async, if CPU-bound work (async doesn't help), if debugging critical, if latency SLA requires threads

**Key Misconceptions:**
- ❌ Thinking async is always better (not for CPU-bound)
- ❌ Not understanding memory overhead of threads (1000 threads = 1GB)
- ❌ Not knowing spawn_blocking() for blocking operations

**Your Score: ___ / 10**

---

# LEVEL 8: Failure Modes

## Question 8.1: Cascading Failures

**Question:**
Trace cascading failures:

**Scenario**: The NATS queue server becomes unreachable.

What happens to:
- a) New task submissions
- b) Audit logging (if tasks log to queue)
- c) Concurrent requests in flight
- d) System recovery

**Your Answer:**
```
New submissions:
Audit logging:
Concurrent requests:
Recovery:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct new submission impact | ☐ Correct ☐ Partial ☐ Wrong |
| Correct audit impact | ☐ Correct ☐ Partial ☐ Wrong |
| Correct concurrent impact | ☐ Correct ☐ Partial ☐ Wrong |
| Realistic recovery strategy | ☐ Realistic ☐ Partial ☐ Unrealistic |

**What I'm Looking For:**
- New submissions: Queue fails, new enqueue operations fail, error returned
- Audit: If audit logging uses queue (it doesn't in current code, uses file), would fail
- Concurrent: Unaffected (they use queue only for opt-in async feature)
- Recovery: Reconnect when NATS available, replay queued tasks

Shows understanding of failure domains.

**Key Misconceptions:**
- ❌ Not knowing which components depend on NATS
- ❌ Thinking all requests fail (only queue-dependent ones)
- ❌ Not understanding bulkheads/isolation

**Your Score: ___ / 10**

---

## Question 8.2: Partial Failure

**Question:**
A user submits a task. Execution succeeds, but:
1. Sandbox crashes after returning result (not before)
2. Audit logging fails
3. Cache write fails

For each:
- Does user get result?
- Is system consistent?
- What's lost?

**Your Answer:**
```
Sandbox crashes (after result):
- User gets result?
- System consistent?
- Lost:

Audit fails:
- User gets result?
- System consistent?
- Lost:

Cache write fails:
- User gets result?
- System consistent?
- Lost:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct result delivery (1) | ☐ Correct ☐ Partial ☐ Wrong |
| Correct consistency (1) | ☐ Correct ☐ Partial ☐ Wrong |
| Correct loss (1) | ☐ Correct ☐ Partial ☐ Wrong |
| Same for (2) | ☐ All 3 ☐ 2/3 ☐ 1/3 ☐ Wrong |
| Same for (3) | ☐ All 3 ☐ 2/3 ☐ 1/3 ☐ Wrong |

**What I'm Looking For:**
1. Sandbox: Yes, yes, nothing (result returned before crash)
2. Audit: Yes, maybe (action happened but audit doesn't reflect), non-repudiation proof
3. Cache: Yes, yes, future cache hit (performance impact, not correctness)

Shows understanding of idempotency and failure isolation.

**Key Misconceptions:**
- ❌ Thinking audit failure means action didn't happen (it did)
- ❌ Not understanding that some failures are acceptable
- ❌ Confusing consistency with availability

**Your Score: ___ / 10**

---

# LEVEL 9: Security

## Question 9.1: Authentication Attack

**Question:**
How would an attacker compromise authentication in this system? Consider:
- Forging a JWT token
- Stealing a token
- Replaying a token
- Provider account compromise
- JWKS cache poisoning

For each attack:
- Is it possible?
- What's the defense?
- How would you detect it?

**Your Answer:**
```
Forging JWT:
- Possible?
- Defense:
- Detection:

Stealing token:
- Possible?
- Defense:
- Detection:

[Continue for all 5]
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct feasibility (5/5) | ☐ All ☐ 4/5 ☐ 3/5 ☐ <3 |
| Identifies defenses | ☐ Full ☐ Partial ☐ Missing |
| Realistic detection | ☐ Realistic ☐ Partial ☐ Unrealistic |

**What I'm Looking For:**
- Forge: Hard (Ed25519 private key needed)
- Steal: Possible (token in memory), defense: TLS, short expiry, detection: unusual usage pattern
- Replay: Hard (tokens expire, provider checks)
- Provider compromise: Critical failure, defense: key rotation
- JWKS cache poison: Mitigated by TTL (1h), detection: signature verification failure

**Key Misconceptions:**
- ❌ Not understanding Ed25519 prevents forging
- ❌ Not considering token expiry
- ❌ Not understanding TTL mitigation

**Your Score: ___ / 10**

---

## Question 9.2: Sandbox Escape

**Question:**
A sophisticated attacker wants to escape the sandbox and access the host system. They have:
- Control over task prompt (arbitrary code execution in sandbox)
- Knowledge of all public software (no zero-days)

What are the defense layers preventing escape?

For each layer:
- How does it work?
- Can it be bypassed?
- What would bypass it?

**Your Answer:**
```
Layer 1 (Firecracker):
- Works by:
- Bypass:
- Requires:

Layer 2 (Landlock):
- Works by:
- Bypass:
- Requires:

Layer 3 (seccomp):
- Works by:
- Bypass:
- Requires:

All three bypassed?
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct mechanism (3/3) | ☐ All ☐ 2/3 ☐ 1/3 ☐ None |
| Realistic bypass path | ☐ Realistic ☐ Partial ☐ Speculative |
| Understands defense-in-depth | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- Firecracker: Hardware-level VM isolation, requires hypervisor exploit (KVM bug)
- Landlock: FS access control, requires LSM bypass (kernel bug)
- seccomp: Syscall filtering, requires unfiltered syscall that enables escape
- All three: Would require kernel exploit + Firecracker exploit + seccomp bypass (extremely unlikely)

Defense in depth = layering reduces escape probability dramatically.

**Key Misconceptions:**
- ❌ Not understanding that each layer requires different exploit type
- ❌ Thinking one layer sufficient (it's not)
- ❌ Not realizing defense-in-depth significantly reduces risk

**Your Score: ___ / 10**

---

# LEVEL 10: Performance

## Question 10.1: Latency Analysis

**Question:**
Break down the latency for a typical request:

**Cache hit (Tier 1)**: How long?
**Cache miss (Tier 2 hit)**: How long?
**Cache miss (full computation)**: How long?

For each, estimate latency contribution:
- Authentication: ?
- Policy validation: ?
- Cache lookup: ?
- Embedding: ?
- Sandbox execution: ?
- Audit logging: ?
- Other: ?

**Your Answer:**
```
Tier 1 hit:
- Auth: ? → Total: ?
- Policy: ?
- Cache: ?

Tier 2 hit:
- Auth: ? → Total: ?
- Policy: ?
- Cache: ?
- Embedding: ?

Full computation:
- Auth: ? → Total: ?
- Policy: ?
- Embedding: ?
- Sandbox: ?
- Audit: ?

Bottleneck:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Reasonable Tier 1 estimate | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Reasonable Tier 2 estimate | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Reasonable full compute estimate | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Identifies real bottleneck | ☐ Correct ☐ Partial ☐ Wrong |

**What I'm Looking For:**
- Tier 1: Auth 5ms + policy 5ms + cache 1ms = ~10ms
- Tier 2: Same as above + embedding 50ms + similarity search 10ms = ~70ms
- Full: Auth + policy + embedding + sandbox 500-1000ms + audit 10ms = ~1 second
- Bottleneck: Sandbox startup (500ms), embedding computation (50ms)

**Key Misconceptions:**
- ❌ Not understanding sandbox dominates latency
- ❌ Forgetting auth latency (5-10ms adds up)
- ❌ Underestimating embedding cost

**Your Score: ___ / 10**

---

## Question 10.2: Scaling Analysis

**Question:**
You're running this system. Request volume increases 10x (from 100 req/sec to 1000 req/sec). What breaks first, and in what order?

Consider:
- CPU (Tokio workers)
- Memory (cache, VM storage)
- Disk (audit logging)
- Network (to NATS, auth provider)
- Firecracker VMs (spawn bottleneck)

Rank by when they become bottleneck:
1. [First to bottleneck]
2. [Second]
3. [Third]
4. [Fourth]
5. [Fifth]

Then: What would you change first?

**Your Answer:**
```
Bottleneck order:
1. [component] - because:
2. [component] - because:
3. [component] - because:
4. [component] - because:
5. [component] - because:

Change first:
Reason:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Reasonable bottleneck order | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Justified reasoning | ☐ Sound ☐ Partial ☐ Weak |
| Identifies real constraint | ☐ Correct ☐ Partial ☐ Wrong |

**What I'm Looking For:**
- Firecracker spawn (500ms × 1000 req = 500s!) - critical bottleneck first
- Then memory (100MB × 10 concurrent VMs = 1GB per 10 VMs, becomes problem)
- Then audit logging disk I/O (1000 appends/sec)
- Then CPU (computation still affordable)
- Then network (usually least constrained)

Change first: VM pooling or pre-warming (reduces startup from 500ms to ~10ms)

Shows understanding of actual system constraints.

**Key Misconceptions:**
- ❌ Thinking CPU bottlenecks first (it doesn't)
- ❌ Not realizing VM spawn is sequential (massive bottleneck)
- ❌ Not understanding memory impact of many VMs

**Your Score: ___ / 10**

---

# LEVEL 11: Scalability

## Question 11.1: Multi-Region Deployment

**Question:**
Design how you'd deploy SecureAI across 3 regions (US, EU, Asia). Assume each region has high traffic but users expect <100ms latency.

Consider:
- Configuration (same or per-region?)
- Authentication (JWKS caching, provider)
- Audit trail (centralized or per-region?)
- Cache (replicated or per-region?)
- Task queue (centralized or per-region?)

**Your Answer:**
```
Configuration:
- Strategy:
- Reasoning:

Authentication:
- Strategy:
- Reasoning:

Audit trail:
- Strategy:
- Reasoning:

Cache:
- Strategy:
- Reasoning:

Task queue:
- Strategy:
- Reasoning:

Trade-offs:
- Centralized:
- Distributed:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Coherent strategy (each component) | ☐ Mostly ☐ Partial ☐ Incoherent |
| Understands latency impact | ☐ Full ☐ Partial ☐ Missing |
| Realistic tradeoffs | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Identifies consistency challenges | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- Config: Per-region (each region independent, no central delay)
- Auth: Per-region JWKS cache (lower latency), but potential inconsistency
- Audit: Centralized (source of truth), but higher latency; or per-region + async central replication
- Cache: Per-region (low latency), but inconsistent (if same prompt in EU vs US, different cache)
- Queue: Per-region (no cross-region latency)

Tradeoffs: Consistency vs latency, simplicity vs complexity.

**Key Misconceptions:**
- ❌ Thinking "centralize everything" (hurts latency)
- ❌ Thinking "decentralize everything" (consistency nightmare)
- ❌ Not understanding eventual consistency tradeoff

**Your Score: ___ / 10**

---

## Question 11.2: Horizontal Scaling

**Question:**
You need to scale from 1 server to 100 servers handling 100,000 requests/sec. What do you need to change?

Rank these changes by difficulty (1=easiest, 5=hardest):

- [ ] Add more Firecracker VMs (just buy more servers)
- [ ] Distribute audit logging
- [ ] Replicate caches
- [ ] Distribute NATS cluster
- [ ] Share configuration across instances

Then ask: Which change breaks the most assumptions in the current architecture?

**Your Answer:**
```
Difficulty ranking:
1. [component] - Reason:
2. [component] - Reason:
3. [component] - Reason:
4. [component] - Reason:
5. [component] - Reason:

Breaking most assumptions:
Reason:

What needs to fundamentally change:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Reasonable difficulty ranking | ☐ Mostly ☐ Partial ☐ Unrealistic |
| Understands scaling challenges | ☐ Full ☐ Partial ☐ Missing |
| Identifies architectural assumption | ☐ Full ☐ Partial ☐ Missing |

**What I'm Looking For:**
- Easiest: Add servers (stateless design helps)
- Hard: Audit logging (append-only file doesn't distribute)
- Hard: Cache replication (consistency problem)
- Hard: NATS cluster (operational complexity)
- Hardest: Shared configuration (need central config server)

Breaking most assumptions: Audit logging (file-based doesn't scale to 100+ servers)

Current architecture assumes: Single-server audit log, independent caches, statelessness for horizontal scale.

**Key Misconceptions:**
- ❌ Thinking horizontal scale is "just add servers"
- ❌ Not understanding audit logging becomes bottleneck
- ❌ Not realizing file-based audit doesn't scale

**Your Score: ___ / 10**

---

# LEVEL 12: Architectural Redesign

## Question 12.1: Redesign for Cost Optimization

**Question:**
You've deployed SecureAI to production. Cost is the main issue (infrastructure expensive). You need to reduce cost by 50% while maintaining performance and security.

Propose changes to (pick 3 components to re-architect):
- Sandbox execution (currently 500ms startup per task)
- Cache (currently two-tier, expensive embeddings)
- Audit logging (currently all entries logged)
- Queue (currently NATS cluster)
- Authentication (currently caching JWKS for 1 hour)

For each chosen component:
1. What's the current cost driver?
2. What would you change?
3. What's the tradeoff?
4. How much cost savings?

**Your Answer:**
```
Component 1: [chosen]
- Cost driver:
- Change:
- Tradeoff:
- Savings:

Component 2: [chosen]
- Cost driver:
- Change:
- Tradeoff:
- Savings:

Component 3: [chosen]
- Cost driver:
- Change:
- Tradeoff:
- Savings:

Total strategy:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Identifies real cost drivers | ☐ Full ☐ Partial ☐ Missing |
| Realistic optimizations | ☐ Realistic ☐ Partial ☐ Speculative |
| Understands tradeoffs | ☐ Full ☐ Partial ☐ Missing |
| Coherent strategy | ☐ Coherent ☐ Partial ☐ Incoherent |

**What I'm Looking For:**
- Sandbox: Cost driver = VM startup overhead. Change = VM pooling (reuse VMs, reduce startup overhead). Tradeoff = complexity, state isolation concerns. Savings = 70% (reduce 500ms per task to 50ms)
- Cache: Cost driver = embedding model computation. Change = sampling (only embed 10% of requests), or simpler model. Tradeoff = lower hit rate. Savings = 40%
- Audit: Cost driver = disk I/O per entry. Change = batch writes or sampling (only audit 50% of actions). Tradeoff = compliance concerns. Savings = 50%

Coherent strategy might combine: VM pooling (biggest savings), reduce embedding sampling (moderate), batch audit writes (moderate).

**Key Misconceptions:**
- ❌ Not identifying real cost drivers
- ❌ Proposing changes with unacceptable tradeoffs (e.g., disable security)
- ❌ Not understanding which components dominate cost

**Your Score: ___ / 10**

---

## Question 12.2: Redesign for Security Hardening

**Question:**
You've deployed SecureAI. A security review reveals concerns:
- Audit trail can be modified (if attacker gets root)
- Configuration can be modified at runtime (hard to verify)
- Sandbox escape exploits possible (like CVE-2024-XXXXX)
- Private key not protected (stored as plaintext file)

Choose 2-3 to address. For each:
1. What's the current weakness?
2. How would you fix it?
3. What's the complexity?
4. What's the operational impact?

**Your Answer:**
```
Weakness 1: [chosen]
- Current:
- Fix:
- Complexity:
- Impact:

Weakness 2: [chosen]
- Current:
- Fix:
- Complexity:
- Impact:

Weakness 3: [chosen] (optional)
- Current:
- Fix:
- Complexity:
- Impact:

Prioritization:
1. [first to fix]
2. [second]
3. [third]
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Correct weakness analysis | ☐ Full ☐ Partial ☐ Missing |
| Realistic fixes | ☐ Realistic ☐ Partial ☐ Speculative |
| Honest complexity assessment | ☐ Honest ☐ Partial ☐ Underestimated |
| Thoughtful prioritization | ☐ Realistic ☐ Partial ☐ Arbitrary |

**What I'm Looking For:**
- Audit modification: Store to immutable storage (e.g., AWS S3 with object lock, append-only cloud storage)
- Configuration: Load at startup only, don't reload. Or cryptographic signing of config. Or config in immutable storage.
- Sandbox escape: Upgrade Firecracker, run in latest kernel, add additional seccomp rules, monitor for escapes
- Private key: Move to TPM, HSM, or encrypted key store. Use kernel keyring.

Prioritization might be:
1. Private key protection (highest impact if compromised)
2. Audit immutability (compliance/forensics critical)
3. Sandbox escape monitoring (defense in depth)

Shows architectural thinking about security at scale.

**Key Misconceptions:**
- ❌ Proposing unrealistic fixes (e.g., "make it unbreakable")
- ❌ Not understanding operational impact (e.g., TPM availability)
- ❌ Not prioritizing by impact/risk

**Your Score: ___ / 10**

---

## Question 12.3: Redesign for Different Use Case

**Question:**
A new customer wants to use SecureAI for:
- **Medical diagnosis assistance** (regulatory: HIPAA, audit mandatory, offline capability required)
- **Financial risk calculation** (regulatory: PCI-DSS, performance critical <100ms, stateful sessions)

For each use case:
1. What features must change?
2. What can't change?
3. What architectural assumptions break?
4. Rough redesign for this use case

**Your Answer:**
```
Medical Use Case:

Must change:
- [change]
- [change]
...

Can't change:
- [requirement]
- [requirement]
...

Broken assumptions:
- [assumption]
- [assumption]
...

Redesign:


Financial Use Case:

Must change:
- [change]
- [change]
...

Can't change:
- [requirement]
- [requirement]
...

Broken assumptions:
- [assumption]
- [assumption]
...

Redesign:
```

**Evaluation Rubric:**

| Criteria | Your Assessment |
|----------|-----------------|
| Understands regulatory needs | ☐ Full ☐ Partial ☐ Missing |
| Realistic feature changes | ☐ Realistic ☐ Partial ☐ Unrealistic |
| Identifies architectural breaks | ☐ Full ☐ Partial ☐ Missing |
| Coherent redesign | ☐ Coherent ☐ Partial ☐ Incoherent |

**What I'm Looking For:**
- Medical: Must add encryption at rest (HIPAA), must support offline (preload models), audit mandatory, data residency requirements. Can't remove sandbox (still need isolation). Breaks: Assumes internet connectivity, assumes centralized auth provider. Redesign: Offline-first with sync, encrypted audit store, local auth fallback.

- Financial: Must reduce latency <100ms (sandbox 500ms is killer), must maintain session state (stateless breaks). Can't remove auth/audit (compliance). Breaks: Stateless assumption, latency budget. Redesign: Keep Firecracker but add VM pooling, add session store (Redis), optimize for latency over throughput.

Tests whether you understand:
- Regulatory requirements drive architecture
- Different use cases need different tradeoffs
- Some core principles non-negotiable (security)

**Key Misconceptions:**
- ❌ Thinking "just add encryption" solves HIPAA
- ❌ Not understanding latency/statelessness tradeoff
- ❌ Proposing designs that ignore regulatory requirements

**Your Score: ___ / 10**

---

# Self-Assessment Summary

## Scoring Summary

| Level | Question | Score |
|-------|----------|-------|
| 1 | System Overview | ___ / 10 |
| 1 | Filesystem | ___ / 10 |
| 1 | Configuration | ___ / 10 |
| 2 | Sandbox | ___ / 10 |
| 2 | Auth | ___ / 10 |
| 2 | Audit | ___ / 10 |
| 3 | Data Flow | ___ / 10 |
| 3 | Cache Path | ___ / 10 |
| 4 | Concurrency | ___ / 10 |
| 4 | Error Handling | ___ / 10 |
| 5 | Command Pattern | ___ / 10 |
| 5 | Trait Pattern | ___ / 10 |
| 6 | Firecracker Decision | ___ / 10 |
| 6 | OAuth Decision | ___ / 10 |
| 7 | Cache Tradeoff | ___ / 10 |
| 7 | Async Tradeoff | ___ / 10 |
| 8 | Cascading Failure | ___ / 10 |
| 8 | Partial Failure | ___ / 10 |
| 9 | Auth Attack | ___ / 10 |
| 9 | Sandbox Escape | ___ / 10 |
| 10 | Latency Analysis | ___ / 10 |
| 10 | Scaling Analysis | ___ / 10 |
| 11 | Multi-Region | ___ / 10 |
| 11 | Horizontal Scale | ___ / 10 |
| 12 | Cost Optimization | ___ / 10 |
| 12 | Security Hardening | ___ / 10 |
| 12 | Use Case Redesign | ___ / 10 |

**Average Score: ___ / 10**

---

## Overall Assessment

### Level Breakdown

| Level | Average | Assessment |
|-------|---------|------------|
| 1 (Structure) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 2 (Components) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 3 (Data Flow) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 4 (Runtime) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 5 (Patterns) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 6 (Decisions) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 7 (Tradeoffs) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 8 (Failures) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 9 (Security) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 10 (Performance) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 11 (Scalability) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |
| 12 (Redesign) | ___ | ☐ Mastered ☐ Competent ☐ Needs work |

---

## Strength Areas (8-10/10)
```
[List levels/topics where you scored 8+]
```

## Weak Areas (5-7/10)
```
[List levels/topics where you scored 5-7]
```

## Growth Areas (<5/10)
```
[List levels/topics where you scored <5]
```

---

## Next Steps

Based on your assessment:

### If average >8/10: You're ready
- Can contribute to codebase
- Can make architectural decisions
- Can mentor others
- Next: Read source code, implement features

### If average 6-8/10: You're competent
- Can implement with guidance
- Understand most concepts
- Weak areas need study
- Next: Deep-dive weak areas, then contribute

### If average <6/10: Continue learning
- Fundamental gaps remain
- Revisit relevant documents
- Focus on weak levels
- Next: Re-read curriculum, retry exam

---

**Last updated**: 2026-08-14

**Duration to complete**: 3-6 hours (all questions)

**Next steps**: Choose a weak area, re-read relevant documentation, retry those questions.

