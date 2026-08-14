# Learning Curriculum: SecureAI MVP
## Executive Summary

**Document Purpose**: A reverse-engineered learning guide to understand the SecureAI MVP repository through the lens of software engineering principles, architecture decisions, and design patterns.

---

## What This Curriculum Teaches

This curriculum teaches you to read, understand, and extend a production-grade security-focused microservice through 20 progressive learning modules.

You will learn:
- **What** SecureAI does (problem domain)
- **Why** it's structured this way (architectural reasoning)
- **How** code flows at runtime (execution paths)
- **Which** patterns and principles it demonstrates (software engineering)
- **When** certain design decisions are appropriate (context-dependent thinking)
- **How** to identify tradeoffs and alternatives (critical analysis)

---

## System Overview

**SecureAI MVP** is an enterprise-grade AI task execution platform that sandboxes AI agent workloads and enforces multi-layered security controls.

### The Problem It Solves

Running untrusted or semi-trusted AI agent code safely in production requires:
1. **Isolation**: Executing in containerized, resource-limited environments
2. **Control**: Enforcing policies before, during, and after execution
3. **Verification**: Semantic threat detection on prompts/commands
4. **Audit**: Immutable record of all actions for compliance
5. **Performance**: Fast response times despite added security layers
6. **Observability**: Complete tracing for debugging and monitoring
7. **Multi-tenancy**: Safe isolation between different customers/organizations

### Core Design Philosophy

**Security by Default**: No request is trusted until authenticated → authorized → guardrail-checked → policy-validated.

**Non-Breaking Evolution**: All features opt-in (disabled by default), new features added without affecting existing code.

**Fail-Secure**: Invalid JWT → block (don't skip auth). Missing permission → block (don't grant). Threat detected → block (don't allow).

---

## Key Statistics

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Lines of Code** | ~3,000 | Core implementation |
| **Total Lines of Tests** | ~2,000+ | 150+ unit tests |
| **Number of Features** | 10 | Each independently optional |
| **Number of Modules** | 12 | policy, auth, guardrails, audit, queue, cache, evals, proxy, router, sandbox, telemetry, api |
| **External Dependencies** | 30+ | Curated for security + performance |
| **Configuration Options** | 50+ | All optional, sensible defaults |
| **Request Latency** | 100-200ms (cached) | ~500-1200ms (no cache) |
| **Throughput** | 1000+ requests/sec | Limited by policy checks |

---

## Architectural Layers

```
┌─────────────────────────────────────────┐
│ API Layer (gRPC)                        │
│ Entry point: PolicyService              │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Security Layer (Auth + RBAC)            │
│ Validates: JWT, roles, permissions     │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Threat Detection (Semantic Guardrails)  │
│ Validates: Prompt safety via ONNX       │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Policy & Cache Layer                    │
│ Enforces: Policy rules, caches results  │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Execution Layer (Sandbox + Queue)       │
│ Executes: In isolated VMs, async queues │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Persistence Layer (Audit + Telemetry)   │
│ Records: Immutable audit trail, traces  │
└─────────────────────────────────────────┘
```

---

## Ten Core Features (Implementations)

| # | Feature | Module | Purpose | Complexity |
|---|---------|--------|---------|-----------|
| 1 | MicroVM Sandboxing | sandbox/ | Isolated execution | High |
| 2 | Audit Ledger | audit/ | Non-repudiation | Medium |
| 3 | OAuth2/OIDC Auth | auth/ | Enterprise identity | Medium |
| 4 | Semantic Guardrails | guardrails/ | Threat detection | High |
| 5 | Task Queue | queue/ | Async execution | High |
| 6 | Semantic Cache | cache/ | Performance | High |
| 7 | Evals & Drift | evals/ | QA + monitoring | Medium |
| 8 | SSE Proxy | proxy/ | Streaming + budgeting | Medium |
| 9 | gRPC Control Plane | api/ | Policy management | Low |
| 10 | Distributed Tracing | telemetry/ | Observability | Low |

---

## Who Should Read This

| Role | Focus Areas | Time Investment |
|------|-----------|-----------------|
| **Backend Engineer** | All sections | Full curriculum (30-40 hrs) |
| **Security Engineer** | Auth, Audit, Guardrails, Architecture Decisions | 15-20 hrs |
| **DevOps/SRE** | Infrastructure, Deployment, Monitoring, Performance | 10-15 hrs |
| **Architect** | Architecture Overview, Design Decisions, Tradeoffs | 20-30 hrs |
| **Newcomer** | Executive Summary → Learning Curriculum → Deep Dives | 40-50 hrs |

---

## How This Curriculum Is Organized

```
Learning Docs/
├── 00_EXECUTIVE_SUMMARY.md (you are here)
├── 01_REPOSITORY_MAP.md (filesystem + modules)
├── 02_SYSTEM_MENTAL_MODEL.md (what does it do)
├── 03_ARCHITECTURE_OVERVIEW.md (layered structure)
├── 04_COMPONENT_ARCHITECTURE.md (each module deep-dive)
├── 05_DATA_ARCHITECTURE.md (state + persistence)
├── 06_RUNTIME_FLOWS.md (request traces)
├── 07_IMPORTANT_FEATURES.md (feature specifications)
├── 08_DESIGN_PATTERNS.md (software engineering concepts)
├── 09_ARCHITECTURE_DECISIONS.md (why + tradeoffs)
├── 10_ALTERNATIVES_TRADEOFFS.md (what else could've been)
├── 11_SECURITY.md (threat model + controls)
├── 12_RELIABILITY.md (fault tolerance + recovery)
├── 13_PERFORMANCE.md (latency + throughput)
├── 14_TESTING.md (test strategy + coverage)
├── 15_INFRASTRUCTURE.md (deployment + scaling)
├── 16_WEAKNESSES_TECHNICAL_DEBT.md (problems + risks)
├── 17_LEARNING_CURRICULUM.md (structured learning path)
├── 18_KNOWLEDGE_GAPS.md (unknowns + assumptions)
└── 19_DEEP_DIVE_ORDER.md (recommended reading order)
```

---

## Key Learning Outcomes by Level

### Beginner
- [ ] Understand what SecureAI does and why it exists
- [ ] Identify the 10 major features and their responsibilities
- [ ] Trace a request from entry to response
- [ ] Understand the role of each layer in the architecture

### Intermediate
- [ ] Explain how each component works (auth, cache, queue, etc.)
- [ ] Identify design patterns used (singleton, adapter, factory)
- [ ] Understand configuration and deployment options
- [ ] Trace error paths and recovery mechanisms

### Advanced
- [ ] Explain why specific architectural decisions were made
- [ ] Identify tradeoffs and when design would need to change
- [ ] Predict performance implications of configuration changes
- [ ] Understand security model and threat mitigation

### Architecture
- [ ] Design extensions to the system
- [ ] Evaluate alternative designs
- [ ] Identify scalability limits and solutions
- [ ] Make informed tradeoff decisions

### Senior-Level
- [ ] Teach the system to others
- [ ] Anticipate failure modes and design around them
- [ ] Challenge assumptions and propose improvements
- [ ] Connect system design to business requirements

---

## How to Use This Curriculum

### Self-Study Path (Recommended)
1. Read Executive Summary (this document)
2. Read Repository Map
3. Read System Mental Model
4. Read Architecture Overview
5. Deep dive into components of interest
6. Read Architecture Decisions
7. Work through Learning Curriculum exercises

### Fast-Track Path (30 minutes)
1. Read Executive Summary
2. Read System Mental Model
3. Read Architecture Overview
4. Skim Runtime Flows
5. Read Learning Curriculum overview

### Interview Preparation Path
1. Read Executive Summary
2. Read Architecture Overview
3. Read Architecture Decisions
4. Read Design Patterns
5. Understand tradeoffs (Alternatives doc)

### Contribution Path
1. Read Executive Summary
2. Read Repository Map (understand structure)
3. Read Component Architecture (understand what you'll work on)
4. Read Testing (understand test expectations)
5. Study the specific module you'll modify

---

## Critical Context

### What This Repository IS
- ✅ A production-grade security-focused microservice
- ✅ A demonstration of enterprise software engineering patterns
- ✅ A multi-layered defense system
- ✅ An example of opt-in feature architecture
- ✅ A learning resource for advanced systems design

### What This Repository IS NOT
- ❌ A simple CRUD web service (complexity is intentional)
- ❌ A real-time web application (async but not reactive)
- ❌ A distributed system in production (could scale better)
- ❌ A bleeding-edge technology showcase (proven tech stack)
- ❌ A complete production system (still needs ops infrastructure)

---

## Core Principles Demonstrated

This repository teaches:

1. **Security-First Design**: Design for threats, not convenience
2. **Layered Defense**: No single point of failure for security
3. **Observability by Default**: Trace everything, understand behavior
4. **Performance Under Constraints**: Fast despite added security
5. **Non-Breaking Extensibility**: Add features without risk
6. **Async/Await Mastery**: High concurrency with async Rust
7. **Error Handling**: Fail-secure, not fail-open
8. **Testing Discipline**: 150+ tests for 3000 lines of code
9. **Configuration as Code**: Features toggled via TOML
10. **Reverse-Engineering Skills**: Learn from existing code

---

## What You'll Be Able to Do After Studying This

1. **Explain** the entire system to a new team member
2. **Identify** architectural patterns and their purposes
3. **Trace** a request from API call to database and back
4. **Understand** security controls and threat mitigation
5. **Evaluate** design decisions and their tradeoffs
6. **Predict** performance implications of changes
7. **Design** extensions without breaking existing code
8. **Test** new features with confidence
9. **Deploy** securely to production
10. **Teach** others the principles behind the design

---

## Navigation Tips

- **Breadcrumbs**: Each document links to related sections
- **References**: Code locations referenced with module:file:line format
- **Diagrams**: Architecture diagrams use Mermaid format
- **Questions**: End-of-section questions test your understanding
- **Exercises**: Practical exercises to reinforce learning
- **Deep Dives**: Links to detailed explanations

---

## Next Steps

1. **Start here**: Read [Repository Map](01_REPOSITORY_MAP.md)
2. **Understand the system**: Read [System Mental Model](02_SYSTEM_MENTAL_MODEL.md)
3. **See the big picture**: Read [Architecture Overview](03_ARCHITECTURE_OVERVIEW.md)
4. **Study deeply**: Proceed through remaining sections in order

---

**Estimated time to complete this curriculum: 30-50 hours**

**Estimated time to be productive with codebase: 10-15 hours**

**Estimated time to redesign a component: 5-10 hours**

---

[→ Next: Repository Map](01_REPOSITORY_MAP.md)
