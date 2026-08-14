# Architecture Change Scenarios

**Practical exercises: Make architectural decisions for the SecureAI system**

Each scenario presents a real-world requirement. You must decide what changes, what stays the same, what risks emerge, and how to test.

---

## How to Use These Scenarios

1. **Read the requirement** (don't skip ahead to solution)
2. **Answer each question** in writing
   - Which components change?
   - Why those components?
   - What risks emerge?
   - How would you test?
   - What tradeoffs are you making?
3. **Then** read the ideal solution
4. **Compare** your approach against it
5. **Reflect** on what you missed

**Time per scenario**: 15-30 minutes

---

---

# LEVEL 1: Small Changes

## Scenario 1.1: Add Execution Timeout

**Requirement:**
Some users' tasks hang indefinitely. You need to add a timeout mechanism:
- If a task runs longer than X seconds, terminate it
- User sees "timeout" error, not "still running"
- Timeout is configurable per deployment

**Your Analysis:**

### Q1: Which components would change?

```
Components that need modification:
1. [component] - because:
2. [component] - because:
...

New components needed:
1. [component] - because:
...
```

### Q2: Why these components?

```
Why [component1]?

Why not [other]?
```

### Q3: What risks do you foresee?

```
Risk 1: [risk] - Mitigation:
Risk 2: [risk] - Mitigation:
Risk 3: [risk] - Mitigation:
```

### Q4: How would you test this change?

```
Unit tests:
- Test case:
- Test case:

Integration tests:
- Test case:
- Test case:

E2E tests:
- Test case:
```

### Q5: What architectural tradeoffs are you making?

```
What you gain:
- [benefit]
- [benefit]

What you lose:
- [cost]
- [cost]

Is this tradeoff worth it?
```

---

## Ideal Solution

**Components to change**:
1. `src/policy/mod.rs:IsolationPolicy` - Add `execution_timeout_secs: Option<u64>` field
2. `src/sandbox/mod.rs:SandboxManager::execute_task()` - Wrap with timeout
3. `secureai.toml` - Add `[sandbox]` timeout configuration
4. `src/audit/mod.rs` - Log timeout as separate event type

**Implementation approach**:
```rust
// In sandbox/mod.rs
pub async fn execute_task(&self, task: &Task) -> Result<ExecutionResult> {
    let timeout = self.config.execution_timeout_secs.unwrap_or(300);
    
    match tokio::time::timeout(
        Duration::from_secs(timeout),
        self.run_in_vm(task)
    ).await {
        Ok(result) => Ok(result),
        Err(_) => {
            // Log timeout
            GlobalAuditHooks::log_timeout(task.id, timeout).await;
            Err(ExecutionError::Timeout(timeout))
        }
    }
}
```

**Why this approach**:
- Minimal changes (timeout added at single point)
- Async-friendly (tokio::time::timeout integrates with async)
- Configurable (per-deployment setting)
- Auditable (timeout logged)

**Risks and mitigations**:
- Risk: VM still running after timeout → Mitigation: SIGTERM + SIGKILL on timeout
- Risk: Partial output returned → Mitigation: Discard partial output, return error only
- Risk: Timeout too short → Mitigation: Make configurable with reasonable default (300s)

**Testing**:
```
Unit tests:
- Test timeout triggers on long-running task
- Test result is error with timeout code
- Test timeout not triggered if fast

Integration tests:
- Test task terminated after timeout
- Test audit entry created with timeout event
- Test multiple concurrent timeouts

E2E tests:
- Submit long-running task, verify timeout
- Verify user sees timeout error
```

**Tradeoffs**:
- Gain: Prevents resource exhaustion, improves UX (users know what happened)
- Lose: Some valid long-running tasks will fail (user must configure higher timeout)
- Worth it: Yes (safety > some inconvenience)

---

## Scenario 1.2: Add Request ID Tracking

**Requirement:**
Operators need to correlate logs across systems. You need:
- Each request gets unique ID
- ID appears in all logs for that request
- ID passed to external services (audit, queue)
- ID used in error responses

**Your Analysis:**

### Q1: Which components would change?

```
Components that need modification:
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/api/grpc.rs` - gRPC interceptor to extract/generate request ID
2. `src/policy/mod.rs` - Pass request ID through PolicyEngine
3. `src/audit/mod.rs` - Include request ID in audit entry
4. All logging - Add request ID to logs

**Implementation approach**:
```rust
// In api/grpc.rs
pub struct RequestIdInterceptor;

impl tonic::service::Interceptor for RequestIdInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let request_id = request
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Store in context (tracing span)
        Ok(request)
    }
}
```

**Why this approach**:
- Interceptor pattern = minimal code changes
- Uses gRPC metadata (standard location)
- Integrates with OpenTelemetry tracing
- Optional in client (server generates if missing)

**Risks and mitigations**:
- Risk: Duplicate IDs if not random enough → Mitigation: Use UUID v4
- Risk: ID lost in async calls → Mitigation: Use structured logging/tracing
- Risk: Audit entry doesn't include ID → Mitigation: Modify audit schema

**Testing**:
```
Unit tests:
- Test request without ID gets generated
- Test request with ID is preserved
- Test ID format valid

Integration tests:
- Test ID appears in all audit entries for request
- Test ID in error response
- Test ID passed to queue

E2E tests:
- Trace full request lifecycle by ID
```

**Tradeoffs**:
- Gain: Operability (debugging easier), compliance (audit trail more useful)
- Lose: Tiny overhead (UUID generation, header parsing)
- Worth it: Absolutely (debugging cost is huge)

---

## Scenario 1.3: Add Task Cancellation

**Requirement:**
Users want to cancel a submitted task (before it runs):
- Endpoint to cancel task by ID
- Returns "cancelled" status
- Cancelled task never executes
- Audit logs the cancellation

**Your Analysis:**

### Q1: Which components would change?

```
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/queue/mod.rs` - Add cancel mechanism
2. `src/api/grpc.rs` - Add CancelTask RPC
3. `src/audit/mod.rs` - Log cancellation
4. Task state machine - Add "cancelled" state

**Implementation approach**:
```rust
// In queue/mod.rs
pub struct Task {
    pub id: String,
    pub status: TaskStatus,  // Queued, Running, Completed, Cancelled
}

// In api/grpc.rs
pub async fn cancel_task(&self, id: String) -> Result<()> {
    // Update task status to cancelled in queue
    self.queue.cancel_task(&id).await?;
    GlobalAuditHooks::log_task_cancelled(&id).await;
    Ok(())
}
```

**Why this approach**:
- Uses existing queue infrastructure
- State change is idempotent (cancelling twice OK)
- Audit trail captures intent
- Works with async processing (task might not exist if already executed)

**Risks and mitigations**:
- Risk: Race condition (cancel vs execution) → Mitigation: Task status atomic update
- Risk: Cancel succeeds but task executes anyway → Mitigation: Task checks status before execution
- Risk: Customer cancels, but already charged → Mitigation: Document that cancellation best-effort (executed task not refundable)

**Testing**:
```
Unit tests:
- Test cancel on queued task works
- Test cancel on non-existent task is safe
- Test cancel on completed task is no-op

Integration tests:
- Test queued task not executed after cancel
- Test audit entry created
- Test multiple cancels idempotent

E2E tests:
- Submit task, cancel, verify not executed
- Submit task, let it run, cancel (should fail), verify executed
```

**Tradeoffs**:
- Gain: Better UX (users can cancel), cost savings (no wasted compute)
- Lose: Complexity in state machine, race conditions to handle
- Worth it: Yes (UX improvement worth complexity)

---

# LEVEL 2: Feature Changes

## Scenario 2.1: Add Task Batching

**Requirement:**
Customers want to submit 1000 related tasks at once and get results in batch:
- One API call to submit multiple tasks
- Get back task IDs for all tasks
- Get back status/results for all tasks
- Tasks must execute in order (user guarantees independence otherwise)

**Your Analysis:**

### Q1: Which components would change?

```
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/api/grpc.rs` - Add BatchExecute RPC
2. `src/policy/mod.rs` - Validate each task in batch
3. `src/queue/mod.rs` - Enqueue all tasks atomically
4. Response format - Return batch of results

**Implementation approach**:
```rust
// In api/grpc.rs
pub async fn batch_execute(
    &self,
    request: BatchExecuteRequest,
) -> Result<BatchExecuteResponse> {
    let tasks = request.tasks;
    
    // Validate all tasks
    for task in &tasks {
        self.engine.validate_task(task).await?;
    }
    
    // Enqueue all tasks atomically
    let task_ids = Vec::new();
    for task in tasks {
        let id = self.queue.enqueue(task).await?;
        task_ids.push(id);
    }
    
    // Audit the batch
    GlobalAuditHooks::log_batch_submitted(task_ids.len()).await;
    
    Ok(BatchExecuteResponse { task_ids })
}
```

**Why this approach**:
- Minimal changes (just iterate and enqueue)
- Reuses existing validation
- Atomic from user perspective (all succeed or all fail)
- Queue handles concurrency (multiple workers process independently)

**Risks and mitigations**:
- Risk: One task invalid, entire batch rejected → Mitigation: Validate before enqueueing or document "all or nothing"
- Risk: Tasks execute out of order → Mitigation: Queue guarantees order per topic, or document no-guarantee
- Risk: Partial failures (task 1-500 succeed, 501-1000 fail) → Mitigation: Two-phase: validate all, then enqueue all

**Testing**:
```
Unit tests:
- Test 10 tasks batched
- Test invalid task in batch rejected
- Test 1000 tasks accepted

Integration tests:
- Test all tasks appear in queue
- Test all tasks eventually complete
- Test audit logs batch submission

E2E tests:
- Submit batch of 100 tasks
- Verify all complete
- Verify order maintained (if guaranteed)
```

**Tradeoffs**:
- Gain: Better UX for bulk operations, simpler user code
- Lose: API complexity (new endpoint), no partial success handling
- Worth it: Yes (bulk operations common)

---

## Scenario 2.2: Add Task Prioritization

**Requirement:**
Premium customers want their tasks processed first:
- Tasks have priority level (low, normal, high)
- High priority tasks executed before low priority
- Fair-share: At least some low-priority tasks always process
- Audit shows priority used

**Your Analysis:**

### Q1: Which components would change?

```
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/queue/mod.rs` - Multiple priority queues (or priority field in task)
2. `src/queue/consumer.rs` - Consumer pulls from high-priority first
3. `src/api/grpc.rs` - Accept priority in request
4. `src/policy/mod.rs` - Validate that user can set priority (RBAC)
5. `src/audit/mod.rs` - Log priority in audit entry

**Implementation approach**:
```rust
// In queue/mod.rs - NATS with subject-based routing
pub async fn enqueue_task(&self, task: Task) -> Result<String> {
    let subject = match task.priority {
        Priority::High => "tasks.high",
        Priority::Normal => "tasks.normal",
        Priority::Low => "tasks.low",
    };
    
    self.nats.publish(subject, serialize(&task)).await?;
    Ok(task.id)
}

// In queue/consumer.rs - Weighted fair scheduling
pub async fn get_next_task(&self) -> Result<Task> {
    // Try high-priority first (80% of time)
    // Try normal-priority (15% of time)
    // Try low-priority (5% of time)
    
    // This ensures low-priority not starved
}
```

**Why this approach**:
- Uses existing queue infrastructure (NATS)
- Separate subjects = easy to scale (different workers for high-priority)
- Weighted scheduling = fairness without guarantees (acceptable)
- RBAC integration = security control

**Risks and mitigations**:
- Risk: All customers set high-priority (defeats purpose) → Mitigation: Rate limiting on high-priority
- Risk: High-priority starvation of low-priority → Mitigation: Weighted scheduling (5% minimum)
- Risk: Non-premium users not allowed high-priority → Mitigation: RBAC checks in policy engine

**Testing**:
```
Unit tests:
- Test task with priority enqueued to correct subject
- Test RBAC prevents non-premium from high priority

Integration tests:
- Submit high + low priority tasks
- Verify high executes first
- Verify low eventually executes (not starved)

E2E tests:
- Weighted test: 100 high + 100 low
- Verify roughly 80:20 execution ratio
```

**Tradeoffs**:
- Gain: Premium feature (revenue), better latency for customers paying more
- Lose: Complexity (multiple queues), fairness questions (starvation potential)
- Worth it: Maybe (depends on revenue model)

---

# LEVEL 3: Infrastructure Changes

## Scenario 3.1: Add Result Caching (Distributed)

**Requirement:**
Deploying to multiple regions. Cache must be shared across regions so:
- If US user asks question, result cached in US region
- If EU user asks same question, gets result from cache (don't compute again)
- Cache invalidated periodically (1 week)
- Cost savings: 40% fewer computations

**Your Analysis:**

### Q1: Which components would change?

```
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/cache/mod.rs` - Add Redis backend option
2. `src/cache/exact.rs` - Use Redis instead of in-memory for Tier 1
3. `src/cache/semantic.rs` - Keep in-memory (too expensive to replicate)
4. `secureai.toml` - Add Redis connection string

**Implementation approach**:
```rust
// In cache/exact.rs
pub struct ExactCache {
    // Tier 1: Redis (shared across regions)
    redis: RedisClient,
    ttl_secs: u64,  // Default 604800 (1 week)
}

impl ExactCache {
    pub async fn get(&self, prompt: &str) -> Result<Option<CachedResponse>> {
        let key = format!("cache:exact:{}", hash(prompt));
        self.redis.get(&key).await
    }
    
    pub async fn set(&self, prompt: &str, response: &CachedResponse) -> Result<()> {
        let key = format!("cache:exact:{}", hash(prompt));
        self.redis.set_ex(&key, response, self.ttl_secs).await
    }
}

// Tier 2 (semantic) stays in-memory (too expensive to distribute)
```

**Why this approach**:
- Tier 1 (exact match) is most valuable (1ms vs 5000ms)
- Sharing Tier 1 across regions is safe (consistent hash = same result everywhere)
- Tier 2 (semantic) stays local (expensive to replicate vector DB)
- TTL handles eventual consistency (1 week acceptable staleness)

**Risks and mitigations**:
- Risk: Redis becomes SPOF → Mitigation: Redis cluster with replication
- Risk: Network latency (US→EU→US) → Mitigation: Regional Redis clusters (not single global)
- Risk: Stale cache (prompt answer changed) → Mitigation: TTL, manual invalidation API
- Risk: Cache poisoning (wrong answer cached) → Mitigation: Validation on cache write

**Testing**:
```
Unit tests:
- Test Redis get/set works
- Test TTL respected
- Test key format consistent

Integration tests:
- Test US cache hit doesn't require EU fetch
- Test multiple regions consistent
- Test cache invalidation

E2E tests:
- US: Submit prompt, answer cached
- EU: Submit same prompt, get cached answer
- Verify 40% reduction in full computations
```

**Tradeoffs**:
- Gain: 40% cost savings, faster responses across regions
- Lose: Operational complexity (Redis cluster), latency on cache miss (network call)
- Worth it: Yes if multi-region and cost matters

---

## Scenario 3.2: Replace NATS with PostgreSQL Queue

**Requirement:**
Simpler operations team wants fewer dependencies. Replace NATS with PostgreSQL:
- Queue stored in database table
- Consumers poll database
- Transactions guarantee exactly-once delivery
- Audit trail in same database

**Your Analysis:**

### Q1: Which components would change?

```
```

### Q2: Why these components?

```
```

### Q3: What risks do you foresee?

```
```

### Q4: How would you test this change?

```
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Components to change**:
1. `src/queue/mod.rs` - Replace NATS client with PostgreSQL
2. `src/queue/consumer.rs` - Poll database instead of NATS
3. `src/queue/config.rs` - Remove NATS config, add PostgreSQL
4. Database schema - Add queue table

**Database schema**:
```sql
CREATE TABLE task_queue (
    id BIGSERIAL PRIMARY KEY,
    task_id UUID NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR NOT NULL, -- queued, processing, completed, failed
    created_at TIMESTAMP NOT NULL,
    lease_until TIMESTAMP,  -- Lease for exactly-once
    completed_at TIMESTAMP
);

CREATE INDEX idx_queue_status ON task_queue(status) WHERE status = 'queued';
```

**Implementation approach**:
```rust
// In queue/mod.rs
pub struct PostgresProducer {
    pool: PgPool,
}

impl PostgresProducer {
    pub async fn enqueue_task(&self, task: Task) -> Result<String> {
        sqlx::query!(
            "INSERT INTO task_queue (task_id, payload, status, created_at)
             VALUES ($1, $2, 'queued', NOW())",
            task.id,
            serde_json::to_value(&task)?
        )
        .execute(&self.pool)
        .await?;
        
        Ok(task.id)
    }
}

// In queue/consumer.rs
pub async fn get_next_task(&self) -> Result<Option<Task>> {
    // Transaction: Get and lease in one go (exactly-once)
    let task = self.pool.transaction(|txn| {
        Box::pin(async move {
            sqlx::query_as!(
                Task,
                "UPDATE task_queue
                 SET status = 'processing', lease_until = NOW() + INTERVAL '5 minutes'
                 WHERE id = (
                     SELECT id FROM task_queue 
                     WHERE status = 'queued'
                     LIMIT 1
                     FOR UPDATE SKIP LOCKED
                 )
                 RETURNING *"
            )
            .fetch_optional(txn)
            .await
        })
    }).await?;
    
    Ok(task)
}
```

**Why this approach**:
- PostgreSQL transactions guarantee exactly-once (select-for-update pattern)
- Single database reduces dependencies
- Lease mechanism prevents duplicate processing (if consumer crashes, lease expires)
- Same database as audit = simpler backup/restore

**Risks and mitigations**:
- Risk: Database becomes bottleneck → Mitigation: Connection pooling, indexing
- Risk: Lease timeout too short (task crashes mid-execution) → Mitigation: Make configurable (default 5 min)
- Risk: Database outage = queue broken → Mitigation: Need database replication anyway
- Risk: Polling is inefficient (constant queries) → Mitigation: LISTEN/NOTIFY for events

**Testing**:
```
Unit tests:
- Test task enqueued correctly
- Test dequeue leases task
- Test completed task removed

Integration tests:
- Test exactly-once delivery (multiple consumers)
- Test lease expiry requeues task
- Test concurrent consumers don't get same task

E2E tests:
- Submit 100 tasks
- Multiple consumers process
- Verify all completed exactly once
```

**Tradeoffs**:
- Gain: Simpler (fewer systems), single source of truth, audit integrated
- Lose: Performance (database queries vs memory), scalability (database bottleneck), NATS benefits (built-in clustering)
- Worth it: Maybe (depends on traffic volume and team expertise)

---

# LEVEL 4: Scale

## Scenario 4.1: 10x Traffic (100 to 1000 req/sec)

**Requirement:**
Launch major marketing campaign. Traffic increases 10x (100 → 1000 req/sec). System must handle without doubling cost.

**Your Analysis:**

### Q1: Which components hit bottleneck first?

```
Rank by order bottleneck appears:
1. [component] - Because:
2. [component] - Because:
3. [component] - Because:
4. [component] - Because:
```

### Q2: For each bottleneck, what's your fix?

```
Bottleneck 1: [component]
- Current: [issue]
- Fix: [solution]
- Cost of fix:
- Tradeoff:

[Repeat for each]
```

### Q3: What risks emerge at 1000 req/sec?

```
Risk 1: [risk]
- Mitigation:

Risk 2: [risk]
- Mitigation:
```

### Q4: How would you test this change?

```
Load tests:
- Test case:

Capacity planning:
- Measurement:
```

### Q5: What architectural tradeoffs are you making?

```
```

---

## Ideal Solution

**Bottleneck analysis**:
1. **Sandbox VM spawn** (500ms × 1000 req = 500 seconds queue!) - First bottleneck
2. **Memory** (100MB per VM × 10 concurrent = 1GB, fine until CPU limits)
3. **Audit logging disk I/O** (1000 writes/sec to disk)
4. **CPU** (embedding computation, policy checks)

**Fixes** (in order):

```
1. VM Pooling (CRITICAL)
   Current: Spawn fresh VM per request (500ms startup)
   Fix: Pre-spawn 50 idle VMs, reuse with cleanup
   Cost: 5GB extra memory (50 VMs × 100MB), complexity
   Result: Startup 500ms → 50ms (10x improvement)

2. Cache efficiency
   Current: Semantic cache O(n) search
   Fix: Add FAISS approximate nearest neighbor search
   Cost: New dependency, complexity
   Result: Semantic search 60ms → 10ms

3. Audit batching
   Current: Fsync per entry (expensive at 1000/sec)
   Fix: Buffer 100 entries, batch fsync (10 per sec)
   Cost: Some delay in audit visibility (100ms)
   Result: Audit write 1ms → 0.1ms

4. Horizontal scaling
   Current: Single machine
   Fix: Run on 3 machines (each handles ~400 req/sec)
   Cost: Load balancer, shared state (cache, audit)
   Result: Capacity 100 → 1200 req/sec
```

**Why in this order**:
- VM spawn is sequential bottleneck (can't parallelize)
- Cache can be approximated (good enough)
- Audit batching doesn't affect correctness
- Horizontal scale is fallback (most expensive)

**Risks at 1000 req/sec**:
- Risk: VM pool empty (all VMs in use) → New requests wait in queue → Degraded response time
  - Mitigation: Size pool to handle peak (50-100 VMs)
- Risk: Cache memory explodes (10k prompts × 100MB = 1GB) → OOM killer
  - Mitigation: LRU eviction, max cache size
- Risk: Audit log disk full → Write failures
  - Mitigation: Monitor disk space, rotate logs, archive

**Testing at 1000 req/sec**:
```
Load tests:
- Apache JMeter: 1000 concurrent users, 100 req/sec sustained
- Monitor: Response time, memory, disk I/O, error rate

Capacity planning:
- Measure VM spawn time, cache hit rate, audit write latency
- Verify no OOM, no disk full, <50ms p99 latency

Chaos tests:
- Kill VM during execution (verify graceful recovery)
- Poison cache (verify semantic search doesn't crash)
- Rotate audit log (verify no data loss)
```

**Tradeoffs**:
- Gain: Support 10x traffic without 10x cost
- Lose: Complexity (VM pooling, cache tuning, audit batching)
- Worth it: Yes (revenue justifies complexity)

---

## Scenario 4.2: 100x Traffic (1000 to 100k req/sec)

**Requirement:**
Become market leader. Traffic increases 100x (1000 → 100k req/sec). System must handle OR you lose market.

**Your Analysis:**

### Q1: Which core architectural assumptions break at 100k req/sec?

```
Assumption 1: [assumption] - Breaks because:
Assumption 2: [assumption] - Breaks because:
Assumption 3: [assumption] - Breaks because:
```

### Q2: For each broken assumption, what's the fundamental fix?

```
Assumption 1: [assumption]
- Broken because: [why]
- Fundamental fix: [redesign]
- Cost:
- Complexity:

[Repeat]
```

### Q3: What architectural pattern emerges?

```
Pattern: [pattern]
- Why necessary: [reason]
- How it helps: [benefit]
```

### Q4: What can't scale to 100k?

```
1. [component] - Reason: [limit]
2. [component] - Reason: [limit]
```

### Q5: What would you redesign first?

```
Component: [component]
- Current design: [describe]
- Bottleneck: [specific issue]
- Redesign: [new approach]
- Why first: [justification]
```

---

## Ideal Solution

**Broken assumptions**:
1. **Single machine** → Need distributed system (100k req exceeds single machine capacity)
2. **File-based audit** → Need centralized audit service (O(n) append doesn't scale)
3. **In-memory cache** → Need distributed cache (1GB cache becomes 100GB)
4. **Sequential VM spawn** → Need VM pre-provisioning at scale
5. **Request-response latency** → Need async batch processing (100k concurrent impossible)

**Fundamental fixes**:

```
1. Distributed Processing
   Old: Single machine processes all requests
   New: Kubernetes cluster, each pod processes subset
   Why: Single machine bottleneck at ~10k req/sec

2. Audit Service
   Old: Each pod writes to local file
   New: Centralized audit service (gRPC) + replicated store
   Why: File I/O doesn't scale, no central truth at 100k

3. Distributed Cache
   Old: In-memory cache per pod (no sharing)
   New: Redis cluster, all pods hit same cache
   Why: Each pod has different cache, duplicates computation

4. Queue Service
   Old: Each pod connects to NATS
   New: NATS cluster (multiple broker nodes)
   Why: Single NATS node bottleneck ~10k msg/sec

5. Request Batching
   Old: Synchronous request-response
   New: Async batch processing (user gets job ID, polls for result)
   Why: 100k concurrent connections impossible
```

**Architectural pattern that emerges**:
- **Microservices**: Separate services for audit, cache, queue
- **Asynchronous processing**: Batch jobs instead of synchronous
- **Event-driven**: Services communicate via events (NATS, Kafka)
- **Stateless compute**: Each pod replaceable
- **Centralized state**: Audit, cache, config centralized

**What can't scale to 100k**:
1. Synchronous request-response (100k concurrent connections)
2. File-based audit logging (O(n) write operations)
3. In-memory caching per pod (no data sharing)
4. Polling for task results (exponential queries)

**What to redesign first**:
- **Component**: Request processing model
- **Current**: Synchronous (client waits for result)
- **Bottleneck**: 100k concurrent connections → TCP connection limits, memory explosion
- **Redesign**: Async batch (client submits batch, gets back job IDs, polls or subscribes for results)
- **Why first**: Unlocks everything else (polling is light load vs concurrent connections)

---

# LEVEL 5: Failure

## Scenario 5.1: Authentication Provider Outage (1 hour)

**Requirement:**
OAuth provider (Google, Okta) is down for 1 hour. How does SecureAI respond?

**Your Analysis:**

### Q1: What happens to requests during outage?

```
Scenario 1: New token validation
- What happens:
- User impact:

Scenario 2: Valid token (from morning)
- What happens:
- User impact:

Scenario 3: Expired token (but cached JWKS)
- What happens:
- User impact:
```

### Q2: Which component saves you?

```
Component: [component]
- How it helps: [mechanism]
- Why sufficient/insufficient: [analysis]
```

### Q3: What's your recovery strategy?

```
During outage:
- Allow [action]
- Deny [action]
- Reason: [justification]

After outage:
- How do you sync state?
- How do you know outage is over?
```

### Q4: How would you test this failure?

```
Chaos test:
- Setup:
- Failure:
- Expected:
```

### Q5: What architectural assumptions did this test?

```
Assumption 1: [assumption] - Validated/Broke?
Assumption 2: [assumption] - Validated/Broke?
```

---

## Ideal Solution

**What happens during outage**:

```
Scenario 1: New token validation
- Try to verify signature with JWKS from cache
- Cache TTL (1 hour) may not have expired yet
- If expired: Can't verify, request FAILS (401)
- If not expired: Request SUCCEEDS (cached JWKS still valid)

Scenario 2: Valid token (from morning)
- Token not expired (tokens valid for 24h typically)
- Verification uses cached JWKS
- If cache not expired: Request SUCCEEDS
- If cache expired: Request FAILS

Scenario 3: Expired token (but cached JWKS)
- Token expired (JWT has exp claim)
- Verification fails regardless of JWKS cache
- Request FAILS (401)
```

**Which component saves you**:
- **Component**: JWKS cache (src/auth/jwks.rs)
- **How it helps**: Caches public keys for 1 hour. If provider down but cache not expired, verification still works
- **Why sufficient/insufficient**: 
  - Sufficient: If outage <1 hour and users have tokens that aren't token-expired
  - Insufficient: If cache expired (1hr+), new requests fail. If token expired, fails anyway

**Recovery strategy**:

```
During outage (assume JWKS cache expired):
- Allow: Requests with valid cached tokens (if somehow cached)
- Allow: Requests with service account key (if implementing)
- Deny: New authentication requests (no way to verify)
- Reason: Fail-secure (better to deny than allow invalid)

After outage:
- JWKS cache refetches from provider (automatic on TTL expiry)
- No manual sync needed (next request triggers fetch)
- How you know outage over: First successful JWKS fetch
```

**Chaos test**:
```
Setup:
- System running normally
- Mock OAuth provider responds to JWKS requests
- Users have valid tokens

Failure:
- Mock provider returns 503 (unavailable)
- JWKS cache TTL set to 5 seconds (for quick test)

Expected:
- First 5 seconds: Requests succeed (cached JWKS)
- After 5 seconds: Requests fail (can't fetch JWKS)
- After provider recovers: Requests succeed again

Test implementation:
- Testcontainers with mock OAuth server
- Inject latency/failures
- Verify behavior matches expectation
```

**Architectural assumptions tested**:
1. **Assumption**: JWKS cache TTL (1 hour) sufficient to survive provider outage
   - **Result**: Only partially true (depends on cache age when outage starts)
   
2. **Assumption**: Cached JWKS keys never become invalid
   - **Result**: True (OAuth provider doesn't rotate keys during outage)
   
3. **Assumption**: Fail-secure (deny on error) acceptable
   - **Result**: True for security, but hurts availability (tradeoff accepted)

**Improvement**: Add fallback auth mechanism (e.g., API keys for service-to-service) that doesn't depend on OAuth provider

---

## Scenario 5.2: Database Full (Audit Logging)

**Requirement:**
Audit log disk is full (100% used). What happens? How do you recover?

**Your Analysis:**

### Q1: What happens to incoming requests?

```
Request 1: Normal task execution
- What happens: [describe flow]
- Does task execute? [yes/no]
- Does audit log? [yes/no]

Request 2: Audit write fails
- What happens to user:
- What happens to system:
```

### Q2: Is this a critical failure or graceful degradation?

```
Assessment: [critical/graceful]
- Justification: [reasoning]
```

### Q3: How would you detect this?

```
Detection mechanism:
- Monitor: [what to monitor]
- Alert: [what to alert on]
- Recovery: [manual process]
```

### Q4: How would you prevent this?

```
Prevention strategy:
- Monitoring: [automated checks]
- Capacity planning: [growth model]
- Rotation: [log retention]
```

### Q5: What architectural changes would make this safer?

```
Option 1: [approach]
- How safer: [mechanism]
- Cost: [complexity/perf]

Option 2: [approach]
- How safer: [mechanism]
- Cost: [complexity/perf]
```

---

## Ideal Solution

**What happens**:

```
Request 1: Normal task execution
- PolicyEngine validates task
- Sandbox executes task
- Audit logging attempts to write
- Disk full → File write fails
- Error logged (but where?)
- User: Sees success (task executed, audit failed silently)
- System: Audit entry lost (non-repudiation broken)

Request 2: Audit write fails
- Execution successful
- Audit write fails with "disk full"
- Current code: Error logged but non-fatal
- User: Sees success anyway
- System: Integrity compromised (action happened but not logged)
```

**Critical failure or graceful degradation**:
- **Assessment**: CRITICAL (non-repudiation broken, compliance violation)
- **Justification**: Audit trail is legal requirement. If audit fails, system should fail. Current code treats audit failure as non-fatal (wrong choice).

**Detection**:
```
Monitoring:
- Watch disk usage on audit partition (alert at 80%, critical at 95%)
- Monitor audit write failures (error rates)
- Check audit log file age (alerts if >X hours without new entry)

Alerting:
- Disk >90%: Warning
- Disk >99%: Critical (page on-call)
- Audit write failure rate >0.1%: Critical

Recovery (manual):
1. SSH to machine
2. Identify large files
3. Archive old audit logs (if rotation policy exists)
4. Expand disk partition OR add new disk
5. Verify audit writes work
6. Run verification to find lost entries (if any)
```

**Prevention**:
```
Monitoring (automated):
- Daily check: "Audit partition disk usage today"
- Alert if trajectory would hit 100% in <7 days
- Alert if audit file growth rate changed

Capacity planning:
- Model growth rate: requests/sec × audit entry size
- At 1000 req/sec × 100 bytes per entry = 100KB/sec = 8GB/day
- Retention: Keep 30 days audit (240GB)
- Plan disk: 300GB (240 + 20% buffer)
- Grow proactively (don't wait until 99%)

Rotation:
- Keep 30 days of audit (oldest deleted)
- Before deletion: Archive to S3 (immutable)
- Verification: Audit entries should form continuous chain (no gaps)
```

**Architectural changes**:

```
Option 1: Async audit with buffer
- Write audit entries to in-memory queue
- Background job flushes to disk
- If flush fails: Hold in memory (up to limit)
- If queue full: Fail the request (correct behavior)
- Benefit: Disk full doesn't immediately fail
- Cost: More memory, complexity

Option 2: Centralized audit service
- Audit service as separate system
- All pods write to service (not local file)
- Service handles disk management (rotation, capacity)
- Pods don't need audit disk access
- Benefit: Centralized, easier to monitor
- Cost: Network dependency for audit

Option 3: Audit database (PostgreSQL)
- Write audit to database (transaction)
- Database handles durability, rotation
- One source of truth (same DB as queue)
- Benefit: Transactional, queryable
- Cost: Database becomes critical, needs replication

Best: Option 2 + monitoring (central audit service is standard pattern)
```

---

# LEVEL 6: Architectural Redesign

## Scenario 6.1: Add Multi-Tenancy

**Requirement:**
Currently single-tenant (one customer, all data together). Add multi-tenancy:
- Different customers have different data
- Customers can't see each other's data
- Billing per customer
- Compliance: GDPR data residency per customer

**Your Analysis:**

### Q1: Which boundaries must change?

```
Boundary 1: [boundary]
- Currently: [how isolated]
- Must change to: [new isolation]
- Risk if not isolated: [consequence]

[Repeat for each boundary]
```

### Q2: Where does tenant context flow through system?

```
Entry point (API):
- Where extracted: [location]
- How passed: [mechanism]

[Trace through each major component]
```

### Q3: What's your data isolation strategy?

```
Strategy 1: [approach]
- How isolated: [mechanism]
- Blast radius if breached: [consequence]
- Operational complexity: [complexity]
- Cost: [overhead]

Strategy 2: [approach]
- How isolated: [mechanism]
- Blast radius if breached: [consequence]
- Operational complexity: [complexity]
- Cost: [overhead]

[Which do you choose? Why?]
```

### Q4: What breaks if you get tenant isolation wrong?

```
Scenario 1: Tenant A can read Tenant B's audit logs
- Why possible: [mechanism]
- Impact: [consequence]
- How detected: [detection method]

Scenario 2: Tenant A can use Tenant B's cached responses
- Why possible: [mechanism]
- Impact: [consequence]
- How prevented: [prevention]

[Other scenarios...]
```

### Q5: How would you test this?

```
Unit tests:
- Test case: [test]

Integration tests:
- Test case: [test]

E2E tests:
- Test case: [test]

Security tests:
- Test case: [test]
```

---

## Ideal Solution

**Boundaries that must change**:

```
Boundary 1: Authentication context
- Currently: One user (from JWT)
- Must change to: User + Tenant (extract from JWT or header)
- Risk: If tenant not validated, users can impersonate other tenant

Boundary 2: Audit logging
- Currently: Single global audit file
- Must change to: Audit file per tenant (or database with tenant filter)
- Risk: If audit not filtered, customers see each other's actions

Boundary 3: Caching
- Currently: Global cache (all prompts mixed)
- Must change to: Cache keyed by tenant + prompt
- Risk: If not separated, Tenant A's cached answer used for Tenant B

Boundary 4: Task queue
- Currently: Single queue (all tasks mixed)
- Must change to: Queue per tenant (or per-tenant filtering)
- Risk: If not separated, Tenant A's task processed under Tenant B's quota

Boundary 5: Policy/configuration
- Currently: Global policy
- Must change to: Policy per tenant (or tenant-specific overrides)
- Risk: If not overridable, can't customize per customer

Boundary 6: Audit trail (compliance)
- Currently: Single machine (no data residency control)
- Must change to: Audit per region (EU data stays in EU, US in US)
- Risk: GDPR violation if EU customer's data leaves EU
```

**Tenant context flow**:

```
Entry point (gRPC):
- Extract from: JWT claims (sub = user, aud = tenant)
- Pass via: Context metadata (OpenTelemetry span)
- Available to: All handlers, services

PolicyEngine:
- Receives tenant context
- Loads tenant-specific policy
- Validates against tenant's model list, path list

Sandbox:
- Receives tenant context
- Executes with tenant-specific limits
- Logs to tenant's audit trail

Cache:
- Key includes tenant ID
- format: "cache:{tenant_id}:{prompt_hash}"
- Tier 1 and Tier 2 both tenant-scoped

Audit logging:
- Includes tenant_id in every entry
- Written to tenant's audit store
- Queries filtered by tenant

Queue:
- Subject includes tenant: "tasks.{tenant_id}.{priority}"
- Consumer pulls from tenant's subject
- Quota checked per tenant
```

**Data isolation strategy**:

```
Strategy 1: Separate databases per tenant
- How isolated: Complete database isolation
- Blast radius: Single customer's data only
- Operational complexity: High (N databases to manage)
- Cost: High (N database instances)
- Best for: High-security, regulatory requirement

Strategy 2: Shared database, row-level security
- How isolated: Database RLS policies
- Blast radius: All customers if RLS misconfigured
- Operational complexity: Medium (policy management)
- Cost: Medium (one database, plus RLS overhead)
- Best for: Cost-conscious, trust DB RLS

Strategy 3: Single database, application-level filtering
- How isolated: Queries filtered by tenant
- Blast radius: All customers if filter missing
- Operational complexity: Low (filtering is easy)
- Cost: Low (single database)
- Best for: Small deployments, but risky

Choice for this system: Strategy 2 (shared database + RLS)
- Reason: Good balance of security, cost, operational complexity
- Add application-level filtering as defense-in-depth
```

**What breaks if isolation wrong**:

```
Scenario 1: Tenant A can read Tenant B's audit logs
- Why possible: Audit file not tenant-filtered, Tenant A queries database without WHERE tenant_id = A
- Impact: GDPR violation, data breach, loss of customer
- How detected: Log analysis (correlation of prompts across customers)
- Prevention: RLS on audit table, audit query filtering, test coverage

Scenario 2: Tenant A uses Tenant B's cached responses
- Why possible: Cache key doesn't include tenant
- Impact: Confidentiality breach, wrong answers to users
- How detected: A/B testing (same prompt, different tenant, different answer expected but got same)
- Prevention: Tenant in cache key, cache query filtering, test coverage

Scenario 3: Tenant A can escalate to admin using Tenant B's credentials
- Why possible: Token validation doesn't check tenant, JWT trusts without validation
- Impact: Complete compromise of Tenant B
- How detected: Admin action audit showing Tenant A taking action on Tenant B resource
- Prevention: Token validation checks tenant, RBAC includes tenant context, test coverage

Scenario 4: Tenant A's tasks run with Tenant B's quota
- Why possible: Queue consumer doesn't filter by tenant, quota checking doesn't validate tenant
- Impact: Tenant A DOS's Tenant B (uses all their quota)
- How detected: Tenant B complains "my quota used but I didn't run tasks"
- Prevention: Queue per tenant, quota stored per tenant, consumer validates tenant
```

**Testing**:

```
Unit tests:
- Test TenantContext extracted from JWT
- Test cache key includes tenant
- Test audit entry includes tenant
- Test queue consumer filters by tenant

Integration tests:
- Create Tenant A, submit task, verify in Tenant A's queue only
- Create Tenant B, query cache, cannot see Tenant A's cached responses
- Verify Tenant A cannot query Tenant B's audit logs (RLS blocks)
- Submit identical task as Tenant A and Tenant B, verify separate cache entries

E2E tests:
- Two browser windows, two customers
- Customer A submits task
- Customer B submits task
- Verify complete isolation (no cross-contamination)

Security tests:
- Customer A tries to read Customer B's audit table (query without WHERE)
- RLS policy blocks it
- Customer A tries to modify JWT to change tenant
- Validation fails (signature invalid)
- Customer A submits task with explicit tenant_id = B in request
- Validator ignores, uses JWT tenant instead
```

**Summary of changes**:
- Add tenant context extraction (gRPC middleware)
- Add tenant to every relevant data structure (cache key, audit entry, queue subject)
- Add tenant-based policy (policy per customer)
- Add tenant-based quota (billing per customer)
- Add data residency (audit storage per region)
- Add RLS on database (if using database)
- Extensive testing (security is critical)

---

## Scenario 6.2: Convert to Event-Driven Architecture

**Requirement:**
Current architecture is synchronous (client submits task, waits for result). Convert to event-driven:
- User submits task (returns immediately with ID)
- System processes task asynchronously
- User gets notified when done (webhook, SSE, polling)
- Multiple downstream systems listen to events (audit, analytics, billing)
- Decoupled services (changes to one don't break others)

**Your Analysis:**

### Q1: What's the event model?

```
Core events:
1. Event: [event name]
   - Triggered by: [what triggers]
   - Data: [what included]
   - Consumers: [who listens]

[Define all events]
```

### Q2: Which components publish events?

```
Component 1: [name]
- Events published:
  1. [event]
  2. [event]

[Repeat for each]
```

### Q3: Which components consume events?

```
Event 1: [event name]
- Consumers:
  1. [consumer] - Why:
  2. [consumer] - Why:

[Repeat for each]
```

### Q4: What's your event broker?

```
Option 1: [broker]
- How works: [mechanism]
- Pros: [advantage]
- Cons: [disadvantage]

Option 2: [broker]
- How works: [mechanism]
- Pros: [advantage]
- Cons: [disadvantage]

Choice: [which]
- Justification: [why]
```

### Q5: How do you guarantee consistency?

```
Consistency model:
- At-least-once: [yes/no]
- Exactly-once: [yes/no]
- Ordering: [yes/no]
- Tradeoff: [cost of consistency]

Example scenario where consistency matters:
[scenario]
```

---

## Ideal Solution

**Event model**:

```
Core events (define at least these):

1. TaskSubmitted
   - Triggered by: User submits task via API
   - Data: {task_id, tenant_id, prompt, model, timestamp}
   - Consumers: Queue (enqueue), Audit (log submission)
   - Ordering: Per tenant (guarantee order within tenant)

2. TaskStarted
   - Triggered by: Sandbox starts execution
   - Data: {task_id, tenant_id, timestamp, vm_id}
   - Consumers: Audit (log start), Analytics (track latency)
   - Ordering: Per task (only one start per task)

3. TaskCompleted
   - Triggered by: Sandbox finishes execution
   - Data: {task_id, tenant_id, result, duration, timestamp}
   - Consumers: Audit (log completion), Billing (charge customer), User (notify)
   - Ordering: Causal (after TaskStarted)

4. TaskFailed
   - Triggered by: Sandbox fails or times out
   - Data: {task_id, tenant_id, reason, timestamp}
   - Consumers: Audit (log failure), Analytics (track errors)
   - Ordering: Causal (after TaskStarted)

5. CacheHit
   - Triggered by: Cache returns result
   - Data: {task_id, tenant_id, cache_tier, timestamp}
   - Consumers: Analytics (hit rate tracking), Billing (no charge?)
   - Ordering: Per task (only one event per task)

6. ResourceExhausted
   - Triggered by: System hits quota/limit
   - Data: {tenant_id, resource, limit, current, timestamp}
   - Consumers: Audit (log limit exceeded), Notifications (alert customer)
   - Ordering: None (informational)
```

**Publishers** (which components emit events):

```
1. API Service (gRPC)
   - Publishes: TaskSubmitted
   - When: User submits task
   - How: Event published to broker after validation

2. Cache Manager
   - Publishes: CacheHit
   - When: Tier 1 or Tier 2 hit
   - How: Event published immediately on hit

3. Sandbox Manager
   - Publishes: TaskStarted, TaskCompleted, TaskFailed
   - When: Execution lifecycle events
   - How: Event published at each transition

4. Quota Manager (new)
   - Publishes: ResourceExhausted
   - When: Quota exceeded
   - How: Event published when quota check fails

5. Policy Engine (monitors)
   - Publishes: ValidationFailed
   - When: Policy validation fails
   - How: Event published before rejection
```

**Consumers** (which components listen to events):

```
1. Audit Service (listens to all)
   - Consumes: All events
   - Action: Log to audit trail
   - Why: Complete audit trail

2. Queue (listens to TaskSubmitted)
   - Consumes: TaskSubmitted
   - Action: Enqueue task
   - Why: Async processing

3. Analytics Service (listens to TaskStarted, TaskCompleted, TaskFailed, CacheHit)
   - Consumes: Performance events
   - Action: Track metrics (latency, error rate, cache hit rate)
   - Why: Monitoring and insights

4. Billing Service (listens to TaskCompleted, ResourceExhausted)
   - Consumes: Usage events
   - Action: Bill customer
   - Why: Usage-based billing

5. Notification Service (listens to TaskCompleted, TaskFailed, ResourceExhausted)
   - Consumes: User-facing events
   - Action: Send webhook/email/notification
   - Why: User experience

6. Sandbox Manager (listens to TaskSubmitted)
   - Consumes: TaskSubmitted
   - Action: [Pulled from queue, already consumed]
   - Why: Alternative to queue (at-least-once guarantee)
```

**Event broker choice**:

```
Option 1: NATS (current system)
- How works: Pub/sub with streaming persistence
- Pros: Lightweight, already deployed, fast, at-least-once
- Cons: Not true event sourcing, limited storage
- Use for: Normal operations

Option 2: Kafka
- How works: Event log with consumer groups
- Pros: Distributed, durable, consumer offset tracking, replay
- Cons: Heavy (ZooKeeper), complex, slow for low latency
- Use for: If need event replay/forensics

Option 3: Event Database (PostgreSQL outbox pattern)
- How works: Events stored in database, then published
- Pros: Transactional, auditable, replay possible
- Cons: Complexity (dual writes), database load
- Use for: If consistency critical

Choice: NATS (with event history preserved)
- Justification: Already deployed, good enough for current scale, minimal changes
- Future: If analytics needs replay, migrate to Kafka
```

**Consistency guarantees**:

```
Consistency model:
- At-least-once: YES
  - Events may be published multiple times
  - Consumers must be idempotent
  - Example: TaskCompleted published twice → billing charges twice (bad!)
  - Mitigation: Consumer tracks (event_id, version) in database to deduplicate

- Exactly-once: ATTEMPTED (hard)
  - Requires: Transactional publish + consume
  - Implementation: Outbox pattern (write event and state in same transaction)
  - Cost: Database overhead, complexity
  - Worth it: Only for critical operations (billing)

- Ordering: CAUSAL (per tenant)
  - TaskSubmitted → TaskStarted → TaskCompleted (preserved)
  - Ordering: Per task ID
  - How: NATS subjects by tenant (tasks.{tenant_id})
  - Cost: Low (built into NATS)

Example scenario where consistency matters:
- TaskSubmitted, TaskFailed, TaskCompleted arrive in wrong order
- If consumer processes TaskCompleted first, then TaskFailed
- User sees "completed" then "failed" (confusing, wrong)
- Solution: Consumer waits for TaskSubmitted before processing other events OR
          Orders events by ID and time before processing

Critical operations (exactly-once):
- Billing: Use database transaction (outbox pattern)
- Audit: Use database transaction (can't lose audit)
- Notifications: At-least-once is OK (duplicate notification acceptable)
```

**Summary of changes**:

1. Define event model (6+ events)
2. Introduce event broker (use NATS)
3. Add event publishing (at each state transition)
4. Add event consumers (audit, analytics, billing, notifications)
5. Add idempotency (consumer deduplication)
6. Add ordering guarantees (causal per tenant)
7. Change API response (return task ID immediately, not result)
8. Add notification system (webhook, polling, SSE)
9. Extensive testing (event ordering, lost messages, duplicates)

**Architecture shift**:
- From: Client waits for result (synchronous, blocking)
- To: Client gets ID, checks later (asynchronous, non-blocking)
- Benefits: Scalability (no blocking), resilience (failures don't block client)
- Costs: Complexity (event handling), latency (not instant), user experience (polling)

---

## Scenario 6.3: Split Into Microservices

**Requirement:**
Monolithic system getting too complex. Split into microservices:
- Sandbox Service (runs tasks)
- Policy Service (validates, makes decisions)
- Audit Service (records actions)
- Cache Service (stores/retrieves cached responses)
- Queue Service (task queueing, coordination)
- API Gateway (routes requests, auth)

**Your Analysis:**

### Q1: How do services communicate?

```
Communication pattern:
- Synchronous: [mechanism] (when used)
- Asynchronous: [mechanism] (when used)
- Data sharing: [mechanism]

Example flow:
User submits task → [service 1] → [service 2] → [response]
- Service 1 calls Service 2 how? [describe]
- Service 2 response goes where? [describe]
```

### Q2: What data is shared vs owned?

```
Service 1: [name]
- Owns: [data]
- Reads from others: [data]
- Writes to others: [data]

[Repeat for each service]
```

### Q3: What breaks when services fail?

```
Service 1: [name]
- If down, what breaks: [consequence]
- How to mitigate: [strategy]

[Repeat for each]
```

### Q4: How do you maintain consistency?

```
Consistency challenge 1: [challenge]
- Current solution: [how solved]
- Distributed solution: [how solved in microservices]

[Repeat]
```

### Q5: What's the deployment strategy?

```
Deployment:
- Docker container per service? [yes/no]
- Kubernetes orchestration? [yes/no]
- Service discovery? [mechanism]
- Load balancing? [mechanism]
- Rollback strategy? [strategy]
```

---

## Ideal Solution

**Service communication**:

```
Synchronous (gRPC):
- API Gateway → Policy Service (validate task)
- API Gateway → Cache Service (check cache)
- Sandbox Service → Audit Service (log action)
- Policy Service → Queue Service (enqueue)

Asynchronous (NATS events):
- Sandbox Service → Event bus (TaskCompleted)
- Event bus → Audit Service (log)
- Event bus → Analytics Service (track metrics)
- Event bus → Notification Service (notify user)

Data sharing:
- Cache Service: Own database (Redis)
- Audit Service: Own database (PostgreSQL)
- Queue Service: Own database (PostgreSQL or NATS)
- Policy Service: Stateless (loads config at startup)
- Sandbox Service: Stateless (no persistent state)
- API Gateway: Stateless (just routing)
```

**Example request flow**:
```
1. User submits task via API Gateway
2. API Gateway → Policy Service (validate)
3. Policy Service checks config (stateless, load once)
4. Policy Service → Cache Service (check Tier 1/2)
5. Cache miss:
   a. Policy Service → Queue Service (enqueue)
   b. Queue Service stores in own database
   c. Queue Service returns task_id
6. Sandbox Service (independent worker) pulls from Queue Service
7. Sandbox executes task
8. Sandbox → Audit Service (log)
9. Sandbox publishes TaskCompleted event
10. Analytics Service subscribes, tracks metrics
11. Notification Service subscribes, notifies user
12. User polls API Gateway for result OR gets webhook callback
```

**Service ownership**:

```
API Gateway:
- Owns: Request routing, auth
- Reads from: Policy Service (validation)
- Writes to: Queue Service (enqueue)

Policy Service:
- Owns: Policy logic, configuration
- Reads from: None (config in memory)
- Writes to: None (stateless)

Cache Service:
- Owns: Cached responses, Tier 1 + Tier 2
- Reads from: None (independent)
- Writes to: None (independent)

Queue Service:
- Owns: Task queue, task state
- Reads from: Policy Service (enqueue)
- Writes to: Sandbox Service (pull)

Sandbox Service:
- Owns: VM lifecycle, execution
- Reads from: Queue Service (pull task)
- Writes to: Audit Service (log), Event bus (publish)

Audit Service:
- Owns: Audit logs, compliance
- Reads from: Event bus (events)
- Writes to: Audit database (append)

Analytics Service:
- Owns: Metrics, analytics
- Reads from: Event bus (all events)
- Writes to: Metrics database

Notification Service:
- Owns: Webhooks, notifications
- Reads from: Event bus (events)
- Writes to: User webhooks
```

**Service failure impact**:

```
API Gateway down:
- Impact: No requests processed (critical)
- Mitigation: Load balancer (multiple instances), health checks

Policy Service down:
- Impact: New tasks can't be validated (critical)
- Mitigation: Stateless (easy to scale), cache policy in gateway

Cache Service down:
- Impact: Cache misses (but system works, just slow)
- Mitigation: Degrade gracefully (compute instead of fetching)

Queue Service down:
- Impact: New tasks can't be queued (critical)
- Mitigation: Persistence (database as backup), replication

Sandbox Service down (one instance):
- Impact: That instance's tasks hang (non-critical)
- Mitigation: Scale to multiple instances, auto-restart on failure

Audit Service down:
- Impact: Logging fails (critical for compliance)
- Mitigation: Persistence (database), replication

Analytics Service down:
- Impact: Metrics not collected (non-critical)
- Mitigation: Non-critical service, can restart without consequence

Notification Service down:
- Impact: Users don't get notified (degraded UX)
- Mitigation: Queue notifications, retry when service back up
```

**Consistency in distributed system**:

```
Challenge 1: Task state consistency
- Monolithic: Task status in single database
- Microservices: Queue Service owns task state, Audit Service owns audit entry
- Solution: Event sourcing (task status derived from events)
- Implementation: Sandbox publishes TaskStarted, TaskCompleted events
              All services subscribe and update their local view
              Event order guaranteed per task

Challenge 2: Cache consistency
- Monolithic: Single cache
- Microservices: Cache Service has own database
- Solution: Cache Service is source of truth
- Implementation: All services query Cache Service for cached responses
              Cache Service invalidates on policy change

Challenge 3: Audit trail consistency
- Monolithic: Single audit file
- Microservices: Audit Service has own database
- Solution: All services publish events, Audit Service records
- Implementation: Event sourcing (audit derived from events)
              At-least-once delivery (events may duplicate)
              Audit Service deduplicates (by event_id)
```

**Deployment strategy**:

```
Containerization: YES
- Each service in Docker container
- Config via environment variables
- Secrets via Kubernetes secrets

Orchestration: Kubernetes YES (at scale)
- Each service as Deployment
- StatefulSet for stateful services (Audit, Cache, Queue)
- Service discovery via Kubernetes DNS

Load balancing: Service mesh (Istio) or Kubernetes Services
- Distribute traffic across instances
- Retry failed requests
- Circuit break failing services

Rollback strategy:
- Blue-green deployment (old + new side-by-side, switch on success)
- Canary deployment (roll out to % of traffic, monitor, then all)
- Event sourcing enables recovery (replay events from certain point)
```

**Summary of changes**:

1. Extract each domain into separate service (6 services)
2. Define service boundaries (data ownership)
3. Implement gRPC for sync calls, events for async
4. Add service discovery and load balancing
5. Add resilience patterns (retries, circuit breakers, bulkheads)
6. Deploy with Kubernetes (container orchestration)
7. Implement distributed tracing (correlation across services)
8. Extensive testing (integration between services, failure scenarios)

**Tradeoffs**:
- Gain: Scalability (each service scales independently), autonomy (teams own services)
- Lose: Complexity (network calls, failures, eventual consistency), debugging (distributed tracing needed)
- Worth it: Yes for scale/team size, probably not for single team

---

# Self-Assessment Guide

## For Each Scenario

1. **Before reading solution**: Write your answer completely
2. **Compare to ideal solution**: What did you get right? Miss?
3. **Identify gaps**: What concepts did you not consider?
4. **Reflect**: Why did you miss this? How to remember next time?

## Scoring Your Answers

**Expert (9-10/10)**
- All components identified
- Reasoning sound
- Tradeoffs understood
- Risks considered
- Testing strategy comprehensive

**Advanced (7-8/10)**
- Most components identified
- Good reasoning
- Some tradeoffs understood
- Some risks identified
- Testing strategy mostly complete

**Competent (5-6/10)**
- Core components identified
- Reasoning present but gaps
- Tradeoffs partially understood
- Few risks identified
- Testing strategy basic

**Developing (3-4/10)**
- Some components identified
- Reasoning unclear
- Tradeoffs missing
- Risks missed
- Testing strategy incomplete

**Novice (<3/10)**
- Few components identified
- Reasoning absent
- No tradeoffs
- No risks
- No testing strategy

---

**Total scenarios**: 14 (3 Level 1, 2 Level 2, 2 Level 3, 2 Level 4, 2 Level 5, 3 Level 6)

**Time to complete all**: 7-10 hours

**Best approach**: Do 1 per day, let learnings settle between scenarios

---

**Last updated**: 2026-08-14

