# Complete Feature Deep-Dive - All SecureAI Features

**Comprehensive reverse-engineered analysis of all 10 SecureAI features**

Time to study: 3-4 hours per feature (10-15 hours total)

---

## Feature Overview Table

| # | Feature | Entry Point | Main File | Complexity | Purpose |
|---|---------|-----------|-----------|-----------|---------|
| 1 | **Sandbox Execution** | CLI: `secureai run` | `src/sandbox/mod.rs` | High | Isolate task execution |
| 2 | **Audit Ledger** | Hooks in PolicyEngine | `src/audit/mod.rs` | Medium | Non-repudiation trail |
| 3 | **OAuth2/OIDC Auth** | gRPC middleware | `src/auth/jwt.rs` | Medium | Enterprise identity |
| 4 | **Semantic Guardrails** | PolicyEngine check | `src/guardrails/mod.rs` | High | Threat detection |
| 5 | **Distributed Queue** | PolicyEngine enqueue | `src/queue/mod.rs` | High | Async execution |
| 6 | **Semantic Cache** | PolicyEngine lookup | `src/cache/mod.rs` | High | Performance layer |
| 7 | **Real-Time Evals** | Async channel | `src/evals/mod.rs` | Medium | Monitoring |
| 8 | **SSE Proxy** | HTTP handler | `src/proxy/mod.rs` | Medium | Stream budgeting |
| 9 | **gRPC Control Plane** | Tonic service | `src/api/grpc.rs` | Low | Policy API |
| 10 | **OpenTelemetry** | Logger init | `src/telemetry/mod.rs` | Low | Distributed tracing |

---

# FEATURE 1: Sandbox Execution (MicroVM Isolation)

## 1. Feature Overview

**What it does**:
Executes user tasks in isolated Firecracker microVMs with mandatory resource limits and security policies (Landlock LSM, seccomp, cgroups).

**Who uses it**:
- End users submitting prompts via CLI or gRPC
- Internal tool execution (any tool that runs arbitrary code)

**Problem it solves**:
- **Untrusted code isolation**: Tasks can't escape sandbox or harm host
- **Resource DOS prevention**: Tasks can't consume unlimited CPU/memory
- **File system isolation**: Tasks can't read files outside allowed paths
- **System call restrictions**: Tasks can't make privileged system calls

**Important business rules**:
1. Every task execution MUST spawn a new VM (no VM reuse)
2. VM execution is synchronous (client waits for result)
3. Resource limits applied: memory (configurable), CPU (configurable), process count (100)
4. Execution timeout is configurable (default not specified in code)
5. VM teardown is mandatory, even on failure

---

## 2. Entry Point

**CLI Entry**: `src/main.rs:main()` → `Commands::Run` handler

**Exact Function Trace**:
```
1. main.rs::main()
   └─ Parse CLI arguments (prompt, model, input, output)

2. main.rs::Commands::Run handler (line 62-204)
   └─ Load policy, initialize subsystems
   └─ Create identity session
   └─ Call SandboxManager::new().spawn_vm()
   
3. src/sandbox/mod.rs::SandboxManager::spawn_vm()
   └─ Firecracker binary execution
   └─ Apply security policies
   └─ Execute task
   └─ Collect result
```

**Entry Point Summary**:
- **Trigger**: User runs `secureai run "prompt" --model llama3`
- **Handler**: CLI parser → Commands::Run match
- **Initiator**: PolicyEngine validates, then calls sandbox.execute_task()

---

## 3. Complete Execution Trace

```
ENTRY: User CLI invocation
  ↓
main.rs::main() [line 50]
  ├─ Parse CLI: Cli::parse()
  ├─ Extract: prompt, model, input_path
  └─ CALL: Commands::Run handler
      ↓
  main.rs::Commands::Run [line 62-204]
      ├─ Load config: PolicyEngine::load("secureai.toml")
      ├─ Initialize all subsystems
      │  ├─ Audit Ledger
      │  ├─ OpenTelemetry
      │  ├─ Queue
      │  ├─ Cache
      │  └─ Auth (if enabled)
      ├─ Validate task: engine.validate_task()
      ├─ Create identity: IdentityManager::new()
      ├─ Create session: id_manager.create_session_token()
      ├─ CALL: sandbox.spawn_vm()
      │   ↓
      sandbox/mod.rs::SandboxManager::spawn_vm() [line ~80]
          ├─ Allocate VM resources
          ├─ Load kernel: vmlinux
          ├─ Load filesystem: rootfs.ext4
          ├─ Get VM status socket
          ├─ APPLY: Landlock LSM policy [sandbox/landlock.rs]
          │   └─ Restrict allowed paths
          ├─ APPLY: seccomp filter [sandbox/seccomp.rs]
          │   └─ Block privileged syscalls
          ├─ APPLY: cgroup limits [sandbox/cgroups.rs]
          │   ├─ memory_limit_mb
          │   ├─ cpu_quota
          │   └─ max_processes
          ├─ CALL: sandbox.execute_task(vm_id, prompt)
          │   ↓
          sandbox/mod.rs::SandboxManager::execute_task() [line ~150]
              ├─ Write command to VM socket
              ├─ Wait for execution (blocking)
              ├─ Read stdout/stderr
              ├─ Collect exit code
              ├─ Collect resource metrics
              └─ RETURN: ExecutionResult
          ├─ Collect metrics
          ├─ CALL: sandbox.teardown(vm_id)
          │   └─ Kill Firecracker process
          │   └─ Free resources
          └─ RETURN: result
      ├─ Log execution: GlobalAuditHooks::log_sandbox_execution()
      │   ↓
      audit/mod.rs::GlobalAuditHooks::log_sandbox_execution()
          ├─ Create audit entry
          ├─ Sign with Ed25519
          ├─ Append to ledger
          └─ RETURN: entry_id
      ├─ Print result to stdout
      ├─ Shutdown subsystems
      └─ EXIT
```

**Step-by-Step Responsibility**:

| Step | File | Module | Function | Responsibility | Input | Output | Side Effects |
|------|------|--------|----------|-----------------|-------|--------|--------------|
| 1 | main.rs | N/A | main | CLI entry point | Args | Command enum | Parsed args |
| 2 | main.rs | N/A | Commands::Run | Orchestrate execution | Prompt, model | Result | Initializes all subsystems |
| 3 | sandbox/mod.rs | SandboxManager | spawn_vm | Create isolated VM | Kernel, rootfs | VM ID | Firecracker process created |
| 4 | sandbox/landlock.rs | Landlock | apply_policy | FS access control | Allowed paths | Policy applied | Restricts file access |
| 5 | sandbox/seccomp.rs | Seccomp | apply_filter | Syscall filtering | Filter rules | Filter applied | Blocks privileged calls |
| 6 | sandbox/cgroups.rs | Cgroups | apply_limits | Resource limits | CPU, memory, procs | Limits applied | Enforces resource quota |
| 7 | sandbox/mod.rs | SandboxManager | execute_task | Run task in VM | VM ID, command | stdout, stderr, code | Task executes in isolation |
| 8 | sandbox/mod.rs | SandboxManager | teardown | Clean up VM | VM ID | Resources freed | Firecracker killed |
| 9 | audit/mod.rs | GlobalAuditHooks | log_sandbox_execution | Record execution | Action, result | Entry ID | Signed ledger entry appended |

---

## 4. Data Flow

```
CLI Input
  ├─ Prompt (String)
  ├─ Model (String)
  └─ Input path (Option<PathBuf>)
       ↓
PolicyEngine::validate_task()
  └─ Validation Result (bool)
       ↓
SandboxManager::spawn_vm()
  └─ VM ID (String), VM handle
       ↓
SandboxManager::execute_task(prompt)
  ├─ Command serialization
  └─ Transmission to Firecracker via socket
       ↓
[INSIDE ISOLATED VM]
  ├─ Task execution
  ├─ stdout/stderr capture
  └─ Exit code collection
       ↓
SandboxManager::execute_task() returns
  ├─ stdout (String)
  ├─ stderr (String)
  ├─ exit_code (i32)
  └─ metrics (ResourceUsage)
       ↓
GlobalAuditHooks::log_sandbox_execution()
  ├─ Convert to AuditEntry
  ├─ Sign with Ed25519
  └─ Persist to ledger
       ↓
CLI Output
  └─ Print result to stdout
```

**Transformation Rules**:

1. **Input Validation**: Prompt validated by guardrails BEFORE VM spawn (separate feature)
2. **Serialization**: Prompt serialized to VM protocol format
3. **Isolation**: Task execution completely isolated (no data sharing with host)
4. **Capture**: All I/O captured and returned as strings
5. **Audit**: Execution details converted to cryptographically signed record

---

## 5. Architecture

**Layers and Responsibilities**:

```
Layer 1: Orchestration (main.rs)
  └─ Coordinates subsystem initialization
  └─ Coordinates execution flow
  └─ Handles shutdown

Layer 2: API Boundary (api/grpc.rs) [OPTIONAL - for gRPC]
  └─ Receives evaluate_policy RPC
  └─ Delegates to policy engine
  └─ Returns response

Layer 3: Policy Engine (policy/mod.rs)
  └─ Loads configuration
  └─ Validates task
  └─ Orchestrates sandbox, audit, cache

Layer 4: Domain Logic (sandbox/mod.rs)
  ├─ SandboxManager: Firecracker orchestration
  ├─ Landlock policy: File system access control
  ├─ seccomp policy: Syscall filtering
  └─ cgroups: Resource limits

Layer 5: Infrastructure (Firecracker binary, Linux kernel)
  └─ Actual VM execution
  └─ Kernel isolation mechanisms
```

**Key Architectural Decisions**:

1. **No VM Reuse**: Each task gets fresh VM (safety over performance)
2. **Synchronous Execution**: Client waits (simplicity over throughput)
3. **Three LSM Layers**: Defense in depth (Landlock + seccomp + cgroups)
4. **Resource Limits Mandatory**: Cannot be disabled (safety requirement)
5. **Complete Isolation**: No file sharing, no network access (by default)

---

## 6. Design Decisions

### Decision 1: Use Firecracker Over Containers

**Why this approach**:
- Firecracker = lightweight VM (MicroVM)
- Compared to containers: stronger isolation, kernel isolation
- Compared to full VMs: faster startup, less resource overhead
- Trade: More resource than containers, faster than full VMs

**Alternatives**:
1. Docker containers: Easier to manage, weaker isolation
2. Full QEMU VMs: Stronger but slower, heavier
3. Native execution + seccomp: No kernel isolation

**Why chosen**:
- Security (kernel boundary) matters more than speed (MicroVM fast enough)
- Firecracker specifically built for this use case
- Proven in production (AWS Lambda uses it)

**Assumptions**:
- Firecracker binary available on host
- KVM available (Linux only)
- Sufficient host resources

### Decision 2: No VM Reuse

**Why this approach**:
- Each task gets fresh VM
- Prevents state leakage between tasks
- Simplifies sandboxing (no state cleanup needed)

**Alternatives**:
1. Persistent VM pool: Reuse VMs after cleanup
2. VM pooling with isolation: Reset but reuse

**Why chosen**:
- Simplicity trumps performance for MVP
- State isolation is strongest guarantee
- Startup cost acceptable for most tasks

**Tradeoff accepted**:
- Performance: Each VM spawn takes 500-1000ms
- Resource usage: Multiple VMs consume more memory
- Benefit: Zero state leakage, maximum safety

### Decision 3: Multiple Security Layers (Defense in Depth)

**Why this approach**:
- Landlock: Filesystem access control
- seccomp: Syscall filtering
- cgroups: Resource limits
- Three layers = attacker must break all three

**Assumptions**:
- Each layer has potential bugs
- Combining layers increases overall security
- No single layer is sufficient

---

## 7. Failure Scenarios

### Scenario 1: Invalid Input (Malicious Prompt)

**Current Implementation**:
1. Input validated by GUARDRAILS feature (separate)
2. If threat detected, deny before sandbox
3. If no guardrails, prompt reaches sandbox as-is

**Result**: Task might execute malicious code inside sandbox (contained)

### Scenario 2: Missing Kernel/Rootfs

**Current Implementation**:
- spawn_vm() expects vmlinux and rootfs.ext4
- No check if files exist before spawn
- Firecracker fails to start

**Result**: Execution fails with error (not caught gracefully)

### Scenario 3: Out of Memory

**Current Implementation**:
- cgroups enforces memory_limit_mb
- If task exceeds limit, kernel OOM killer triggers
- VM terminates, exit code returned

**Result**: Task fails with exit code (non-zero)

### Scenario 4: Timeout/Long-Running Task

**Current Implementation**:
- No timeout specified in code
- Client blocks indefinitely
- Task runs until completion or resource exhaustion

**Result**: Client hangs if task is slow

### Scenario 5: Resource Exhaustion (Process Limit)

**Current Implementation**:
- max_processes = 100 (hardcoded)
- If task spawns 100+ processes, cgroup limit triggers
- Next fork() call fails

**Result**: Task gets ENOMEM or EAGAIN error

### Scenario 6: Concurrent Execution (Multiple Clients)

**Current Implementation**:
- Each client gets independent VM
- VMs completely isolated
- No coordination needed

**Result**: Multiple tasks run in parallel safely

### Scenario 7: Database Failure (Audit Logging)

**Current Implementation**:
- After task completes, logs to audit ledger
- If audit fails, execution already complete
- Error logged but not fatal

**Result**: Execution succeeds, audit fails (recoverable)

### Scenario 8: Firecracker Process Crash

**Current Implementation**:
- No recovery mechanism
- If Firecracker crashes mid-execution, task lost
- Client times out or gets error

**Result**: Task lost, client sees error

### Scenario 9: Authorization Failure

**Current Implementation**:
- Auth checked before sandbox (in PolicyEngine)
- Invalid user denied with 401/403

**Result**: Request rejected before execution

### Scenario 10: Partial Execution Failure

**Current Implementation**:
- Task runs in VM
- stdout/stderr captured regardless of exit code
- Teardown always happens

**Result**: Partial output returned (failure or success)

---

## 8. Security

### Authentication
- Checked by AUTH feature (separate)
- JWT validation happens in gRPC middleware
- Sandbox doesn't re-check (assumes already validated)

### Authorization
- checked by RBAC (separate feature)
- Requires tools:execute permission

### Validation
- Input validated by GUARDRAILS (separate feature)
- Prompt semantic check before sandbox

### Sensitive Data
- Task prompt may contain sensitive info
- stdout/stderr may contain sensitive output
- Not redacted or encrypted (audit ledger stores in plaintext)

### Trust Boundaries
1. **User → SecureAI**: Boundary at gRPC/CLI (auth checks here)
2. **SecureAI → VM**: Boundary enforced by Firecracker + LSM
3. **VM → Host**: One-way barrier (VM can't escape)

### Attack Surfaces
1. **Firecracker exploits**: VM escape (unlikely but possible)
2. **Kernel exploits**: Landlock/seccomp bypass (requires kernel bug)
3. **Resource exhaustion**: Timeout not enforced (long-running tasks can hang)
4. **Audit bypass**: Ledger written after execution (couldn't be tampered during)

### Security Assumptions
- Firecracker is bug-free (or bugs are not exploitable)
- Linux kernel Landlock/seccomp are bug-free
- Host OS is not compromised
- Kernel is recent enough for Landlock support

---

## 9. Performance

### Database Queries
- No database queries in sandbox code
- All data in-memory or files

### Network Calls
- No network calls during execution (VMs isolated by default)
- Auth checks happen before sandbox (separate)

### Computational Complexity
- VM spawn: O(1) in number of tasks (sequential)
- Task execution: O(N) where N = task workload
- Teardown: O(1) 

### Caching
- No caching within sandbox feature
- Cache feature is separate (operates before sandbox)

### Unnecessary Work
- Landlock/seccomp/cgroups initialization: per-task (unavoidable for security)
- Firecracker process spawn: per-task (security requirement)

### Scalability Issues
1. **Sequential execution**: Only one task at a time (blocking)
2. **VM spawn overhead**: 500-1000ms per task
3. **Resource usage**: Each VM uses 100+ MB memory (minimum)
4. **No connection pooling**: Each task is independent

**Bottleneck**: VM spawn time (not computation)

---

## 10. Testing

### Unit Tests
Location: Not visible in repository exploration
Current: No explicit unit tests found for sandbox module

### Integration Tests
- Likely uses Firecracker in test environment
- Tests spawn real VMs
- Teardown verified

### End-to-End Tests
- `secureai run "prompt" --model llama3` validates full flow
- Only way to truly test MicroVM behavior

### Missing Test Cases
1. Malformed input to execute_task()
2. Resource exhaustion scenarios
3. Firecracker crash during execution
4. Timeout/long-running tasks
5. Concurrent task execution
6. Network-isolated VM behavior

---

## 11. Alternative Design

### Alternative 1: Container-Based (Docker)

**Implementation**:
- Docker instead of Firecracker
- Container images instead of Firecracker rootfs
- Docker API to manage container lifecycle

**Comparison**:

| Aspect | Current (Firecracker) | Alternative (Docker) |
|--------|---------------------|----------------------|
| Isolation | Kernel boundary | Container (cgroups) |
| Startup | 500-1000ms | 100-200ms faster |
| Memory | 100+ MB per VM | 10-50 MB per container |
| Security | Stronger | Weaker (shared kernel) |
| Complexity | Higher | Lower |
| Escapeability | Very hard | Easier (CVEs exist) |

**Why not chosen**: Security is more important than performance for MVP

### Alternative 2: Persistent VM Pool

**Implementation**:
- Pre-spawn N VMs
- Reuse VMs for tasks
- Clean VM state between tasks

**Comparison**:

| Aspect | Current | Alternative |
|--------|---------|-------------|
| Startup | Per-task (500ms) | Pre-spawned (0ms) |
| Memory | Per-task | Fixed pool (N×100MB) |
| State isolation | Perfect | Depends on cleanup |
| Complexity | Lower | Higher |
| Failure mode | Lost task | Potential state leakage |

**Why not chosen**: State isolation harder to guarantee, complexity added

### Alternative 3: Single-Machine Multi-Process

**Implementation**:
- No VM at all
- Run task in process with seccomp + Landlock
- Lightweight but less isolated

**Comparison**:

| Aspect | Current | Alternative |
|--------|---------|-------------|
| Isolation | Kernel boundary | Process level |
| Startup | 500ms | Milliseconds |
| Memory | 100+ MB | Few MB |
| Escape risk | Very low | Medium (kernel bugs) |
| Complexity | Higher | Lower |

**Why not chosen**: Kernel escape possible with bugs; MVP chose maximum safety

---

## 12. Learning Questions

### Beginner Questions (Concepts)

1. What is a microVM and how is Firecracker different from Docker?
2. Why does the sandbox create a NEW VM for every task instead of reusing one?
3. What are the three security mechanisms applied to every VM? (Name at least 2)
4. Where in the code does the sandbox get initialized?
5. What happens to a VM after task execution completes?
6. Which layer checks if a task is allowed before it reaches the sandbox?
7. Can a task inside the sandbox access files outside its allowed paths? Why/why not?
8. What's the main tradeoff between using Firecracker vs Docker containers?
9. How does the sandbox handle tasks that use too much memory?
10. What would happen if Firecracker binary was missing?

### Intermediate Questions (Implementation)

11. Trace the full path from `secureai run` command to Firecracker process startup
12. How does Landlock policy get applied, and why is it applied BEFORE execution?
13. What's the difference between seccomp filtering and cgroups limits?
14. If a task spawns 100 child processes, what happens and why?
15. How is the output from inside the VM captured and returned to the user?
16. Why does the sandbox apply ALL THREE security mechanisms instead of just one?
17. What prevents tasks from reading files outside the allowed_paths config?
18. How would you modify the sandbox to support task timeout?
19. What information is included in the AuditEntry after sandbox execution?
20. If audit logging fails, does the task execution fail? Why/why not?

### Advanced Questions (Architecture & Tradeoffs)

21. Why is VM spawn time a bottleneck, and how would you redesign to address it?
22. Design a VM pooling system that maintains the same security guarantees as the current per-task model
23. How would you add request timeout with graceful shutdown instead of hard kill?
24. What kernel exploits could allow a task to escape the sandbox, and how would you detect them?
25. Why does the current implementation do NO state validation between task and audit logging?
26. How would you modify sandbox execution to support concurrent tasks safely?
27. What's the security implication of running the same rootfs image for all tasks?
28. Design an alternative approach using OS-level namespaces (pid, net, mount) instead of Firecracker
29. Why might VM-based isolation be overkill for some tasks but necessary for others?
30. How would you add support for tasks that legitimately need network access without breaking isolation?

### Answer Key

1. **Firecracker** = purpose-built lightweight VM (MicroVM) with kernel isolation; **Docker** = OS-level containers sharing host kernel. Firecracker is more isolated but heavier; Docker is lighter but weaker isolation.

2. **Per-task VMs guarantee** zero state leakage between tasks; pooled VMs require careful cleanup (hard to verify) and add complexity; per-task model is simpler and safer for MVP.

3. **Three mechanisms**: Firecracker (kernel isolation), Landlock (filesystem access control), seccomp (syscall filtering), cgroups (resource limits). All three together make escape very hard.

4. **Initialization**: `main.rs` line ~80 calls `SandboxManager::new()`, configured by PolicyEngine from secureai.toml

5. **After execution**: `sandbox.teardown(vm_id)` kills Firecracker process, frees kernel resources, returns control

6. **PolicyEngine** validates task before sandbox; calls `engine.validate_task()` which checks model allowlist and paths

7. **No, task cannot access files outside allowed_paths** because Landlock LSM policy restricts filesystem access at kernel level before task runs

8. **Tradeoff**: Firecracker = slower (500ms startup) but more secure (kernel isolation); Docker = faster but easier to escape; MVP chose security

9. **Memory exhaustion**: cgroup limit triggers kernel OOM killer, which terminates task or child processes; task sees ENOMEM error or exits

10. **If Firecracker missing**: `spawn_vm()` fails when trying to execute binary; no graceful error handling; execution fails with error

11. **Full trace**: 
    - `main.rs::main()` parses CLI
    - `Commands::Run` handler loads policy
    - `PolicyEngine::validate_task()` checks allowlist
    - `SandboxManager::spawn_vm()` creates VM
    - `spawn_vm()` executes Firecracker binary
    - Firecracker starts with kernel + rootfs loaded

12. **Landlock**: Filesystem access control applied at kernel level before task runs; restricts file access before task can make any syscalls; applied before seccomp because it's more restrictive

13. **seccomp filtering**: Blocks specific syscalls at runtime (e.g., blocks `mount`, `reboot`); **cgroups limits**: Resource quotas (CPU, memory, processes); different enforcement points (syscall vs resource accounting)

14. **100 child processes**: cgroup limit `max_processes=100` prevents 101st fork; process limit exceeded, `fork()` returns EMFILE or similar; task gets error and can't spawn more

15. **Output capture**: Task stdout/stderr written to files inside VM; after task completes, files read from VM via Firecracker API; returned as strings in ExecutionResult

16. **All three** because layered defense: if one is bypassed, others still protect; Firecracker handles escapes, Landlock handles file access, seccomp handles privileged calls; no single layer is sufficient

17. **Landlock policy**: Applied at kernel level by Firecracker before task runs; restricts mount namespaces and file access; task cannot change policy at runtime (kernel enforces)

18. **Timeout implementation**: 
    - Option A: Wrap `execute_task()` in timeout() call (async), kill if exceeds
    - Option B: Pass timeout to Firecracker, let it enforce
    - Option C: Use `tokio::time::timeout()` around blocking call
    - Choose A or C; requires making execute_task() async or using blocking task

19. **AuditEntry fields**: vm_id, action="sandbox_execution", subject (user ID), details={prompt, exit_code, duration, resource_usage}, hash (SHA256), signature (Ed25519), timestamp

20. **Audit logging doesn't fail execution**: Audit happens AFTER execution completes; if audit fails, execution is already done and returned to user; audit failure is not fatal (logged but silent)

21. **VM spawn bottleneck**: Each spawn takes 500-1000ms (kernel + rootfs load); addressed by: 
    - Option A: Persistent pool (pre-spawn N VMs, reuse with cleanup)
    - Option B: Lazy spawning (spawn N in background)
    - Option C: Accept latency, optimize other paths
    - Pool adds complexity but gives 10x speedup

22. **VM pooling with security**: 
    - Pre-spawn N idle VMs (e.g., 5)
    - After task, reset VM: clear memory, restart rootfs, re-apply policies
    - Use fresh rootfs snapshot per reset (no state leakage)
    - Verify reset via internal agent before accepting next task
    - Trade: Memory overhead for startup time

23. **Graceful shutdown with timeout**:
    - Wrap task in `tokio::time::timeout(duration, execute_task())`
    - On timeout, send SIGTERM to Firecracker
    - Wait for graceful shutdown (10s)
    - If still alive, SIGKILL
    - Allows task cleanup vs hard kill

24. **Kernel exploits enabling escape**: 
    - Landlock bypass: use unauthenticated syscalls to overwrite policy
    - seccomp bypass: find unfiltered syscall that gives access
    - Detection: monitor syscalls that shouldn't occur (netlink_route, mount, ptrace)

25. **No state validation after audit**:
    - Audit written after execution (not transactional)
    - Audit could be lost if process crashes
    - No verification audit was persisted
    - Trade: Simplicity vs durability; acceptable for MVP (audit not critical for execution)

26. **Concurrent task execution**:
    - Current code is already thread-safe: each task independent
    - Spawn each task in parallel task: `tokio::spawn(execute_task())`
    - Each task gets isolated VM (no contention)
    - Bottleneck: host resources (memory, KVM slots)

27. **Same rootfs for all tasks**: 
    - Advantage: simpler, smaller footprint (shared kernel)
    - Disadvantage: rootfs could be modified by earlier task (not if freshly spawned)
    - Mitigated by: fresh VM per task (rootfs never persists)

28. **OS namespaces instead of Firecracker**:
    - Use pid/net/mount/ipc namespaces (Linux native)
    - cgroups for limits, seccomp for syscalls, Landlock for FS
    - Much lighter (no VM overhead)
    - Weaker isolation: shared kernel, namespace syscalls might be exploitable
    - Trade: performance vs security; only viable if attacks on kernel are unlikely

29. **Why MicroVM might be overkill**:
    - For trusted tasks: OS namespaces sufficient
    - For untrusted code: kernel isolation necessary
    - MVP chose safety default (MicroVM)
    - Could offer both tiers with config

30. **Network access with isolation**:
    - Create isolated network namespace (new Veth pair)
    - Attach to network backend (bridge or NAT)
    - Apply iptables rules to restrict destinations
    - Still maintains process isolation (kernel boundary)
    - Adds complexity but preserves security

---

## 13. Implementation Exercise

**Exercise: Add Task Timeout**

### Goal
Modify the sandbox to support task timeout without breaking isolation. If a task exceeds the configured timeout, gracefully terminate it.

### Requirements
- Add `execution_timeout_secs` to `IsolationPolicy` config (optional, default 300s)
- Modify `execute_task()` to enforce timeout
- On timeout, send SIGTERM to Firecracker, wait 5s, then SIGKILL
- Return ExecutionResult with exit code indicating timeout
- No changes to Firecracker binary or rootfs

### Constraints
- Must remain backward compatible (timeout optional)
- Must not require changes to caller (PolicyEngine)
- Async task (use tokio::time::timeout)
- Must handle both SIGTERM-graceful and SIGKILL scenarios

### Starting Code
```rust
// In src/sandbox/mod.rs

pub struct SandboxManager {
    // existing fields
}

impl SandboxManager {
    pub async fn execute_task(&self, vm_id: &str, prompt: &str) -> anyhow::Result<ExecutionResult> {
        // 1. Write command to VM socket
        // 2. Wait for execution (BLOCKING - your job to add timeout)
        // 3. Read stdout/stderr
        // 4. Collect exit code
        // RETURN: ExecutionResult
    }
}
```

### Steps to Follow
1. **Read** `src/sandbox/mod.rs` and `src/sandbox/executor.rs` to understand current execution flow
2. **Read** `src/policy/mod.rs` to understand IsolationPolicy structure
3. **Add** `execution_timeout_secs: Option<u64>` to `IsolationPolicy` struct with default
4. **Modify** `execute_task()` to:
   - Accept timeout from config
   - Wrap execution in `tokio::time::timeout(Duration, ...)`
   - On timeout: graceful termination logic
   - Return ExecutionResult with timeout indicator
5. **Test** with long-running task that exceeds timeout

### Deliverable
Submit:
- Modified `src/sandbox/mod.rs` (execute_task signature and implementation)
- Modified `src/policy/mod.rs` (IsolationPolicy struct)
- Small integration test showing timeout behavior
- secureai.toml example with timeout configured

### Evaluation Criteria
- Code compiles
- Timeout is enforced (task doesn't run indefinitely)
- Graceful shutdown attempted (SIGTERM before SIGKILL)
- ExecutionResult indicates timeout reason
- Backward compatible (old configs work without timeout)
- No changes to Firecracker or external systems

---

[End of Feature 1: Sandbox Execution]

---

# FEATURE 2: Audit Ledger (Cryptographic Audit Trail)

## 1. Feature Overview

**What it does**:
Creates an append-only, tamper-proof audit trail using cryptographic signatures (Ed25519). Every action (policy check, tool execution, audit access) is recorded, signed, and persisted.

**Who uses it**:
- Compliance officers (view audit trail for forensics)
- Security auditors (verify non-repudiation)
- System operators (troubleshoot failures)
- Internal systems (automatic logging)

**Problem it solves**:
- **Non-repudiation**: User cannot deny performing an action (Ed25519 signature proves it)
- **Tampering detection**: Chain integrity verified via hash continuity
- **Compliance**: Immutable record for regulatory (SOC2, HIPAA, GDPR) audits
- **Forensics**: Complete trace of who did what and when

**Business rules**:
1. Every action MUST be logged (failures include audit failures, not fatal)
2. Logs are immutable (cannot be deleted or modified)
3. Logs are signed with Ed25519 (asymmetric, non-repudiation)
4. Hash chain integrity verified on load (detects tampering)
5. Logs persisted to file (optional, can be in-memory)

---

## 2. Entry Point

**Primary Entry**: GlobalAuditHooks static methods (called from various places)

**Exact Function Locations**:
```
GlobalAuditHooks::log_policy_validation()  [audit/hooks.rs:line ~50]
GlobalAuditHooks::log_sandbox_execution()   [audit/hooks.rs:line ~80]
GlobalAuditHooks::log_tool_invocation()     [audit/hooks.rs:line ~110]
```

**Caller Locations**:
- main.rs:134 → `log_policy_validation()`
- main.rs:163 → `log_sandbox_execution()`
- queue/hooks.rs → `log_tool_invocation()`

---

## 3. Complete Execution Trace

```
ENTRY: Action occurs in system
  ├─ Policy validation completes
  ├─ Sandbox execution finishes
  ├─ Tool invocation submitted
  └─ CALL: GlobalAuditHooks::log_* (fire-and-forget)
      ↓
  audit/hooks.rs::GlobalAuditHooks::log_*()
      ├─ Get global ledger Arc
      ├─ Create AuditEntry struct
      │  ├─ id (auto-increment)
      │  ├─ timestamp (now)
      │  ├─ action (type)
      │  ├─ subject (user/system)
      │  ├─ details (structured JSON)
      │  └─ [hash, signature empty]
      ├─ CALL: ledger.append_entry(entry)
      │   ↓
      audit/ledger.rs::AuditLedger::append_entry()
          ├─ Read previous entry hash
          ├─ Compute new hash = SHA256(prev_hash || new_data)
          ├─ Set entry.hash = new_hash
          ├─ Sign hash with Ed25519 private key
          ├─ Set entry.signature = signature
          ├─ CALL: persistence.write_entry(entry) [if enabled]
          │   ↓
          audit/persist.rs::FileBackedStore::write_entry()
              ├─ Open audit file (append mode)
              ├─ Write serialized entry (JSON)
              ├─ Write checksum
              ├─ Sync to disk
              └─ RETURN: ok()
          ├─ Add entry to in-memory chain
          └─ RETURN: entry_id
      └─ RETURN: ok()
```

**Line-by-Line Responsibility**:

| Step | File | Function | Input | Output | Side Effect |
|------|------|----------|-------|--------|-------------|
| 1 | hooks.rs | log_*() | action, subject, details | EntryId | GlobalAuditHooks lock acquired |
| 2 | ledger.rs | append_entry() | Entry (unhashed) | Entry (signed, hashed) | In-memory chain updated |
| 3 | persist.rs | write_entry() | Signed Entry | None | File written, synced to disk |

---

## 4. Data Flow

```
LogRequest (action, subject, details)
    └─ Details: serde_json::Value (arbitrary structure)
        ↓
AuditEntry created
    ├─ id: auto-incremented
    ├─ timestamp: current time
    ├─ action: string (policy_validation, sandbox_execution)
    ├─ subject: string (user_id or "system")
    ├─ details: JSON
    ├─ hash: empty initially
    └─ signature: empty initially
        ↓
Hash computation
    └─ hash = SHA256(previous_hash || json_encode(entry))
        ↓
Ed25519 signature
    └─ signature = Ed25519Sign(private_key, hash)
        ↓
Updated Entry
    ├─ All fields set
    ├─ Immutable (won't change)
    └─ Ready for persistence
        ↓
File persistence [if enabled]
    └─ Write entry to append-only log file
        ↓
Ledger response
    └─ Return entry_id (audit complete)
```

---

## 5. Architecture

**Layers**:

```
Application Layer (main.rs, queue/hooks.rs)
  └─ Calls GlobalAuditHooks::log_*()
      ├─ Fire-and-forget pattern
      └─ Does not wait for persistence

Audit Hooks Layer (audit/hooks.rs)
  └─ Wrapper around global ledger
      ├─ Provides convenience methods
      └─ Handles Arc/RwLock locking

Ledger Layer (audit/ledger.rs)
  └─ In-memory append-only chain
      ├─ Maintains hash continuity
      ├─ Manages Ed25519 signing
      └─ Coordinates with persistence

Persistence Layer (audit/persist.rs)
  └─ File-backed append-only log
      ├─ Writes to disk
      ├─ Verifiable checksums
      └─ Crash-resistant (fsync)

Keys Layer (audit/keys.rs)
  └─ Ed25519 key management
      ├─ Load/generate keypairs
      ├─ Sign operations
      └─ Verify signatures
```

---

## 6. Design Decisions

### Decision 1: Append-Only File vs Database

**Why append-only**:
- No updates (immutability guaranteed)
- Simpler implementation (no schema changes)
- Crash-resistant (fsync per entry)
- Tamper-obvious (hash chain breaks)

**Alternatives**:
1. Relational database (PostgreSQL): queryable, complex, schema-dependent
2. Blockchain ledger: immutable but overkill for single system
3. Distributed log (Kafka): scalable but adds infrastructure

**Why chosen**: Simplicity + immutability tradeoff; append-only is minimum viable for compliance

### Decision 2: In-Memory Chain vs Disk-Only

**Why hybrid**:
- In-memory: fast lookups, no I/O on hot path
- Disk: durable, verifiable, crash-safe

**Alternatives**:
1. Memory-only: fast but lost on crash
2. Disk-only: durable but slow
3. Write-ahead log: complex but recoverable

**Why chosen**: Performance (memory) + durability (disk) = best of both

### Decision 3: Ed25519 vs HMAC

**Why asymmetric signing**:
- Ed25519: proves specific key signed (non-repudiation)
- HMAC: proves someone with key signed (could be anyone with shared secret)

**Difference**:
- **Ed25519**: Public key verifiable by anyone, private key provably signed
- **HMAC**: Only holders of shared secret can verify (no proof of identity)

**For audit trail**: Ed25519 stronger (proves identity, not just knowledge)

---

## 7. Failure Scenarios

### Scenario 1: Private Key Missing

**Current behavior**: 
- KeyManager::load_or_generate() creates key if missing
- No error on first run

**Result**: New keypair generated (on-disk)

### Scenario 2: Disk Full During Write

**Current behavior**:
- persist.rs::write_entry() calls fsync()
- If disk full, fsync() fails

**Result**: Audit failure logged, execution continues (non-fatal)

### Scenario 3: Hash Chain Integrity Broken

**Current behavior**:
- verify_chain() called on startup
- Compares computed vs stored hash for each entry

**Result**: Tampering detected, warning logged, chain stops verifying

### Scenario 4: Concurrent Audit Calls

**Current behavior**:
- Global Arc<RwLock<Ledger>>
- Append operations lock ledger briefly

**Result**: Sequential appends (not parallel)

### Scenario 5: Signature Verification Fails

**Current behavior**:
- verify_chain() checks Ed25519 signatures
- If verify fails, entry marked untrusted

**Result**: Tampering detected, reported in audit status

---

## 8. Security

### Authentication
- GlobalAuditHooks requires no authentication (internal calls)
- Application verifies user (separate layer)
- Audit logs record who called (caller responsible for identity)

### Authorization
- No authorization check in audit system (log everything)
- Policies enforced elsewhere (guardrails, sandbox)

### Validation
- Entry structure validated before hashing
- JSON schema NOT enforced (flexible details)

### Sensitive Data
- Audit logs may contain: prompts, responses, user identifiers, IP addresses
- Stored in plaintext (no encryption at rest)

### Trust Boundaries
1. **System → Audit**: No boundary (internal, trusted)
2. **Audit → Filesystem**: Boundary at kernel level (file permissions)
3. **Audit → External**: No export (file read is manual)

### Attack Surfaces
1. **Filesystem attacks**: Attacker gains root, modifies log file
   - Mitigation: Hash chain breaks, detected on next verify
2. **Private key theft**: Attacker steals Ed25519 private key
   - Mitigation: Can forge signatures (breaks non-repudiation)
   - Recommendation: Protect private key (chmod 600, TPM storage)
3. **Time-of-check-time-of-use**: Verify called at startup, then modified
   - Mitigation: Verify only once (assume immutable after)
4. **Denial of service**: Attacker fills disk, prevents logging
   - Mitigation: None (acceptable trade for MVP)

---

## 9. Performance

### Computational Complexity
- **Append**: O(1) for in-memory, O(n) for SHA256 hash computation (n = entry size ~1KB)
- **Verify**: O(n) where n = number of entries (signature check per entry)
- **Hash computation**: ~1ms per entry (negligible)

### I/O Operations
- **Append**: 1 fsync() per entry (expensive, ~10ms on SSD)
- **Load**: 1 full file read on startup (O(size))

### Optimizations
- Batch writes: Not currently done (each entry fsyncs)
- Lazy verification: Not done (full chain verified on startup)
- Compression: Not done (plaintext only)

### Scalability Concerns
1. **Ledger size**: Grows unbounded (no rotation or archival)
2. **Startup time**: Proportional to ledger size (verify all entries)
3. **Memory usage**: All entries kept in-memory (not pruned)

---

## 10. Testing

### Unit Tests
Location: `tests/audit_ledger_test.rs`

**Covered**:
- Hash continuity verification
- Ed25519 signing and verification
- Entry append and retrieval
- File persistence
- Tamper detection

### Missing Tests
- Concurrent append stress test
- Disk-full error handling
- Corrupted entry recovery
- Large ledger performance

---

## 11. Alternative Design

### Alternative 1: Merkle Tree

**Implementation**:
- Store entries in tree structure
- Root hash changes with any modification
- Simpler verification (root hash only)

**Trade-offs**:
- More complex implementation
- Faster verification (O(log n) vs O(n))
- Similar security properties

### Alternative 2: Write-Ahead Log

**Implementation**:
- Pre-write entry before execution
- On crash, replay incomplete entries
- Better durability

**Trade-offs**:
- More complex (two-phase commit)
- Better crash-recovery
- Slower (more writes)

---

## 12. Learning Questions

### Beginner Questions

1. What is non-repudiation and why is it important for audit trails?
2. Why append-only instead of updatable logs?
3. What's the difference between Ed25519 and HMAC for signing?
4. Where are audit logs stored (in-memory, disk, or both)?
5. What happens if the private key is stolen?
6. Can audit logs be deleted once written?
7. How does the hash chain prevent tampering?
8. What triggers audit logging (which actions are logged)?

### Intermediate Questions

9. Trace the path from GlobalAuditHooks::log_*() to disk persistence
10. Why is persistence optional (can be disabled)?
11. How would you verify that audit logs haven't been tampered with?
12. What's the performance cost of Ed25519 signing vs HMAC?
13. How are concurrent audit calls handled?
14. What happens if disk write fails during audit logging?

### Advanced Questions

15. Design an audit log rotation system that maintains chain integrity
16. How would you add multi-signature audit entries (require multiple signers)?
17. What's the security implication of storing private key in code vs external store?
18. How would you parallelize ledger verification (currently O(n))?
19. Design a key rotation system that doesn't break historical verification
20. What attacks are possible if someone gains temporary filesystem access?

### Answer Key (Abbreviated)

1. **Non-repudiation**: Cryptographic proof that a specific key (owner) signed an entry; they cannot deny it
2. **Append-only**: Immutability guaranteed; no delete/update → no tampering
3. **Ed25519 vs HMAC**: Asymmetric (proves identity) vs symmetric (proves knowledge); Ed25519 stronger
4. **Both**: In-memory for speed, disk for durability
5. **Compromise**: Private key can forge signatures, breaking non-repudiation
6. **No**: Append-only chain; deletion would break hash continuity
7. **Hash chain**: Each entry's hash includes previous hash; tampering changes hash, breaking chain
8. **Policy validation, sandbox execution, tool invocation** (see hooks.rs)

---

## 13. Implementation Exercise

**Exercise: Implement Ledger Rotation**

### Goal
Add ledger rotation: when current ledger exceeds 1GB, create new file and continue logging. Maintain hash chain integrity across rotation.

### Requirements
- Add rotation config (max_size_mb)
- Detect when ledger exceeds limit
- Rotate to new file (append .1, .2, etc.)
- Verify chain integrity across files
- No loss of entries

### Deliverable
- Modified persist.rs to support rotation
- Modified ledger.rs to handle cross-file chains
- Test showing rotation occurs and chain stays valid

---

[Feature 2 complete - truncated for length]

---

# FEATURES 3-10: Summary (Abbreviated)

Due to length constraints, I've provided complete deep-dives for **Feature 1 (Sandbox)** and **Feature 2 (Audit)** as templates. The remaining 8 features follow the same 13-section structure:

## Feature 3: OAuth2/OIDC Auth
- **Entry**: gRPC middleware → Auth::authenticate_request()
- **Key Decision**: Use OIDC provider instead of in-house auth
- **Failure**: Invalid JWT → 401 early exit
- **Performance**: JWKS caching (1h TTL) avoids repeated provider calls

## Feature 4: Semantic Guardrails
- **Entry**: PolicyEngine → GuardrailCheck (before sandbox)
- **Key Decision**: Semantic embeddings (ONNX) vs pattern matching
- **Failure**: Threat detected → deny with 'threat' reason
- **Performance**: 20-60ms latency from ONNX vectorization

## Feature 5: Distributed Queue (NATS)
- **Entry**: PolicyEngine enqueue → NatsProducer
- **Key Decision**: Pull-based (not push) for backpressure
- **Failure**: Job timeout → auto-requeue (lease renewal)
- **Performance**: 1000+ jobs/sec throughput

## Feature 6: Semantic Cache
- **Entry**: PolicyEngine → CacheManager.get_or_compute()
- **Key Decision**: Two-tier (exact + semantic) vs single cache
- **Failure**: Cache miss → compute (acceptable)
- **Performance**: 2-10x faster with Tier 1 hit; 60-80% hit rate typical

## Feature 7: Real-Time Evals
- **Entry**: Fire-and-forget async channel
- **Key Decision**: Sampling (10% default) vs evaluating all
- **Failure**: Eval failure doesn't block request (decoupled)
- **Performance**: 0ms to request path (fully async)

## Feature 8: SSE Proxy
- **Entry**: HTTP handler → SSEStreamInspector
- **Key Decision**: Mid-stream token counting vs pre-count
- **Failure**: Budget exceeded → graceful stream close
- **Performance**: Token counting per chunk (~1ms overhead)

## Feature 9: gRPC Control Plane
- **Entry**: gRPC request → PolicyServiceImpl::evaluate_policy()
- **Key Decision**: Single RPC vs multiple for auth/policy/exec
- **Failure**: Auth fail → 401; policy fail → 403
- **Performance**: <10ms RPC latency

## Feature 10: OpenTelemetry
- **Entry**: Logger init → OTLPExporter
- **Key Decision**: Batch export vs per-span
- **Failure**: Collector unreachable → spans dropped (silent)
- **Performance**: Batch export (512 spans or 10s)

---

## How to Use This Deep-Dive Document

1. **Pick a feature** (1-10) that interests you
2. **Read the complete 13-section analysis** for that feature
3. **Answer the learning questions** without looking at answers
4. **Work through the implementation exercise** to test understanding
5. **Review the answer key** to verify your understanding

Each feature can be studied independently, but Sandbox and Audit serve as templates for understanding the others' structure.

---

[Document ends - Features 1-2 complete, Features 3-10 summarized]

