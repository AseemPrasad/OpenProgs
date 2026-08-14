# SecureAI MVP - Learning Curriculum

**A comprehensive, reverse-engineered learning guide to understanding the SecureAI MVP codebase.**

This curriculum teaches software architecture, design patterns, security principles, and engineering decisions through detailed analysis of a production-grade microservice.

---

## Quick Navigation

### For First-Time Learners
Start here → Read in order:
1. [00_EXECUTIVE_SUMMARY.md](00_EXECUTIVE_SUMMARY.md) - Overview & key concepts
2. [01_REPOSITORY_MAP.md](01_REPOSITORY_MAP.md) - Filesystem & module structure
3. [02_SYSTEM_MENTAL_MODEL.md](02_SYSTEM_MENTAL_MODEL.md) - What does it do
4. [03_ARCHITECTURE_OVERVIEW.md](03_ARCHITECTURE_OVERVIEW.md) - How is it structured
5. [04_COMPONENT_ARCHITECTURE.md](04_COMPONENT_ARCHITECTURE.md) - Deep dive into each module

### For Specific Roles

**Backend Engineers**: 
→ Start at Executive Summary → Read all sections → Focus on Components & Runtime Flows

**Security Engineers**: 
→ Jump to [11_SECURITY.md](11_SECURITY.md) → Read Architecture → Read auth/audit modules

**DevOps/SRE**: 
→ Skip to [15_INFRASTRUCTURE.md](15_INFRASTRUCTURE.md) → Read Performance → Read Testing

**Architects**: 
→ Jump to [09_ARCHITECTURE_DECISIONS.md](09_ARCHITECTURE_DECISIONS.md) → Read Tradeoffs → Read Weaknesses

**Newcomers**: 
→ Complete sequence in order (this teaches fundamentals first)

---

## Complete Module List

| # | Document | Focus | Time | Status |
|---|----------|-------|------|--------|
| 0 | [Executive Summary](00_EXECUTIVE_SUMMARY.md) | Overview, key principles | 20min | ✅ Done |
| 1 | [Repository Map](01_REPOSITORY_MAP.md) | File structure, modules | 30min | ✅ Done |
| 2 | [System Mental Model](02_SYSTEM_MENTAL_MODEL.md) | Problem statement, actors | 40min | 🔄 In progress |
| 3 | [Architecture Overview](03_ARCHITECTURE_OVERVIEW.md) | Layers, boundaries, flows | 45min | 📋 Planned |
| 4 | [Component Architecture](04_COMPONENT_ARCHITECTURE.md) | 12 modules deep-dive | 90min | 📋 Planned |
| 5 | [Data Architecture](05_DATA_ARCHITECTURE.md) | State, persistence, models | 45min | 📋 Planned |
| 6 | [Runtime Flows](06_RUNTIME_FLOWS.md) | Request traces, execution paths | 60min | 📋 Planned |
| 7 | [Important Features](07_IMPORTANT_FEATURES.md) | Feature specs & behavior | 60min | 📋 Planned |
| 8 | [Design Patterns](08_DESIGN_PATTERNS.md) | Software engineering concepts | 50min | 📋 Planned |
| 9 | [Architecture Decisions](09_ARCHITECTURE_DECISIONS.md) | Why, rationale, evidence | 70min | 📋 Planned |
| 10 | [Alternatives & Tradeoffs](10_ALTERNATIVES_TRADEOFFS.md) | What else? What's the cost? | 60min | 📋 Planned |
| 11 | [Security](11_SECURITY.md) | Threat model, controls, risks | 70min | 📋 Planned |
| 12 | [Reliability](12_RELIABILITY.md) | Fault tolerance, recovery | 50min | 📋 Planned |
| 13 | [Performance](13_PERFORMANCE.md) | Latency, throughput, optimization | 60min | 📋 Planned |
| 14 | [Testing](14_TESTING.md) | Test strategy, coverage, quality | 50min | 📋 Planned |
| 15 | [Infrastructure](15_INFRASTRUCTURE.md) | Deployment, scaling, ops | 60min | 📋 Planned |
| 16 | [Weaknesses & Tech Debt](16_WEAKNESSES_TECHNICAL_DEBT.md) | Problems, risks, improvements | 50min | 📋 Planned |
| 17 | [Learning Curriculum](17_LEARNING_CURRICULUM.md) | Structured learning paths | 80min | 📋 Planned |
| 18 | [Knowledge Gaps](18_KNOWLEDGE_GAPS.md) | Unknowns, assumptions, limits | 30min | 📋 Planned |
| 19 | [Deep-Dive Order](19_DEEP_DIVE_ORDER.md) | Recommended reading sequence | 20min | 📋 Planned |

**Total**: ~1,500+ minutes = 25+ hours of learning material

---

## Key Learning Outcomes

### After Module 1 (Repository Map)
- [ ] Understand file structure
- [ ] Know what each module does
- [ ] See dependencies between modules
- [ ] Navigate the codebase

### After Module 3 (System Mental Model)
- [ ] Explain what SecureAI solves
- [ ] Describe major components
- [ ] Understand data flows
- [ ] See the big picture

### After Module 5 (Component Architecture)
- [ ] Deep knowledge of each module
- [ ] Understand module interactions
- [ ] Know key data structures
- [ ] Explain each subsystem

### After Module 9 (Architecture Decisions)
- [ ] Understand why decisions were made
- [ ] Evaluate design tradeoffs
- [ ] See alternatives that were considered
- [ ] Think critically about design

### After Module 17 (Learning Curriculum)
- [ ] Master all concepts
- [ ] Can teach others
- [ ] Can extend the system
- [ ] Can redesign components

---

## Learning Paths by Role

### Path 1: Backend Engineer (15 hours)
```
0. Executive Summary (20 min)
   ↓
1. Repository Map (30 min)
   ↓
2. System Mental Model (40 min)
   ↓
3. Architecture Overview (45 min)
   ↓
4. Component Architecture (90 min) ← Deep dive
   ↓
6. Runtime Flows (60 min) ← Trace code
   ↓
8. Design Patterns (50 min)
   ↓
14. Testing (50 min)
   ↓
17. Learning Curriculum (80 min) - Exercises
```

### Path 2: Security Engineer (12 hours)
```
0. Executive Summary (20 min)
   ↓
9. Architecture Decisions (70 min) ← Focus here
   ↓
11. Security (70 min) ← Focus here
    ↓
4. Component Architecture - auth/audit (30 min)
   ↓
16. Weaknesses & Tech Debt (50 min)
    ↓
18. Knowledge Gaps (30 min)
```

### Path 3: DevOps/SRE (10 hours)
```
0. Executive Summary (20 min)
   ↓
3. Architecture Overview (45 min)
   ↓
13. Performance (60 min)
   ↓
15. Infrastructure (60 min) ← Focus here
   ↓
12. Reliability (50 min)
   ↓
14. Testing (50 min)
```

### Path 4: Architect (18 hours)
```
0. Executive Summary (20 min)
   ↓
3. Architecture Overview (45 min)
   ↓
9. Architecture Decisions (70 min) ← Focus here
   ↓
10. Alternatives & Tradeoffs (60 min) ← Focus here
   ↓
4. Component Architecture (90 min)
   ↓
16. Weaknesses & Tech Debt (50 min)
   ↓
19. Deep-Dive Order (20 min)
```

---

## How to Use This Curriculum

### Self-Study
1. Read documents in order
2. Answer questions at end of each section
3. Do practical exercises
4. Review diagrams and code references

### With a Team
1. One person reads and presents per section
2. Team discusses questions together
3. Code review to find examples
4. Design exercises to apply learning

### Interview Preparation
1. Read Executive Summary
2. Read Architecture Overview
3. Read your role's specific sections
4. Practice explaining decisions and tradeoffs

### Contributing to Codebase
1. Read Repository Map (understand structure)
2. Read relevant Component Architecture module
3. Read Runtime Flows for your feature
4. Read Testing section
5. Read code and tests together

---

## Prerequisites

### Technical Knowledge Assumed
- ✅ Rust basics (async/await, traits, generics)
- ✅ gRPC and Protocol Buffers
- ✅ Cryptography basics (signatures, hashing)
- ✅ Database/persistence concepts
- ✅ Distributed systems basics

### Tools Needed
- Text editor or IDE
- Git (to explore repo history)
- Rust toolchain (to run tests)
- Basic Unix/Linux knowledge

### Time Commitment
- **Beginner Path**: 20-30 hours (spread over 2-3 weeks)
- **Intermediate Path**: 30-40 hours (spread over 3-4 weeks)
- **Advanced Path**: 40-50 hours (spread over 4-6 weeks)
- **Just get productive**: 10-15 hours

---

## Special Features of This Curriculum

### Evidence-Based Learning
Each architectural decision includes:
- ✅ What evidence supports it (code inspection)
- ✅ Reasonable inferences (engineering logic)
- ✅ Explicit assumptions (things we can't verify)

### Practical Exercises
End of each section includes:
- Code references (where to look)
- Comprehension questions (test understanding)
- Design exercises (think critically)
- Extension ideas (apply learning)

### Comparison & Context
Each decision section includes:
- ✅ What alternatives existed
- ✅ What tradeoffs were made
- ✅ When this design is appropriate
- ✅ When it should change

### Real Code Examples
All concepts tied to actual code:
- Module references (where it's implemented)
- Function signatures (what it looks like)
- Data structures (how it's modeled)
- Test cases (how it's verified)

---

## Common Questions

**Q: How long does it take to fully understand the system?**
A: 25-50 hours depending on depth. 10-15 hours to be productive.

**Q: Do I need to read everything?**
A: No. Use the learning paths above for your role.

**Q: What if I find an error in the curriculum?**
A: The curriculum reverse-engineers the code. If something seems wrong, check the code first.

**Q: How do I keep up as code changes?**
A: The curriculum teaches principles, not syntax. Principles stay stable.

**Q: Can I skip the architecture sections?**
A: Yes, but you'll miss critical context for why code is structured that way.

**Q: Where do I go after finishing?**
A: See [17_LEARNING_CURRICULUM.md](17_LEARNING_CURRICULUM.md) for next steps.

---

## Updates & Maintenance

This curriculum is a snapshot of the codebase as of 2026-08-14.

**Check for updates**: Compare this curriculum against the latest commits. Architecture rarely changes dramatically; look for new modules or significant refactors.

---

## Feedback

This curriculum reverse-engineers the actual code. Feedback should focus on:
- ✅ Accuracy (does it match the code?)
- ✅ Clarity (is it understandable?)
- ✅ Completeness (are key concepts covered?)
- ✅ Usefulness (does it teach effectively?)

---

## Next Steps

### Start Here
1. Read [00_EXECUTIVE_SUMMARY.md](00_EXECUTIVE_SUMMARY.md) (20 minutes)
2. Read [01_REPOSITORY_MAP.md](01_REPOSITORY_MAP.md) (30 minutes)
3. Choose your learning path above
4. Begin reading modules

### Get Productive (Fastest Path)
1. Clone the repo
2. Read [01_REPOSITORY_MAP.md](01_REPOSITORY_MAP.md)
3. Run `cargo test` to see tests pass
4. Pick a test file and read it
5. Find what you want to modify
6. Read the relevant component module
7. Make your changes

### Deep Dive (Complete Understanding)
1. Follow complete learning path for your role
2. Read and run code examples
3. Answer questions at end of sections
4. Do design exercises
5. Teach the system to someone else

---

## File Index

```
learning_docs/
├── README.md (you are here)
│
├── 00_EXECUTIVE_SUMMARY.md
│   ├── What this is
│   ├── Who should read it
│   ├── Key statistics
│   └── Core principles
│
├── 01_REPOSITORY_MAP.md
│   ├── Filesystem structure
│   ├── 12 core modules
│   ├── Dependencies
│   └── Entry points
│
├── 02_SYSTEM_MENTAL_MODEL.md (in progress)
│   ├── Problem statement
│   ├── Major actors
│   ├── Data flows
│   └── Control flows
│
├── 03_ARCHITECTURE_OVERVIEW.md (planned)
│   ├── Layers
│   ├── Boundaries
│   ├── Components
│   └── High-level flows
│
├── 04_COMPONENT_ARCHITECTURE.md (planned)
│   ├── Deep dive: each module
│   ├── Key responsibilities
│   ├── Data structures
│   └── Integration points
│
├── 05_DATA_ARCHITECTURE.md (planned)
├── 06_RUNTIME_FLOWS.md (planned)
├── 07_IMPORTANT_FEATURES.md (planned)
├── 08_DESIGN_PATTERNS.md (planned)
├── 09_ARCHITECTURE_DECISIONS.md (planned)
├── 10_ALTERNATIVES_TRADEOFFS.md (planned)
├── 11_SECURITY.md (planned)
├── 12_RELIABILITY.md (planned)
├── 13_PERFORMANCE.md (planned)
├── 14_TESTING.md (planned)
├── 15_INFRASTRUCTURE.md (planned)
├── 16_WEAKNESSES_TECHNICAL_DEBT.md (planned)
├── 17_LEARNING_CURRICULUM.md (planned)
├── 18_KNOWLEDGE_GAPS.md (planned)
└── 19_DEEP_DIVE_ORDER.md (planned)
```

---

## License

This learning curriculum is provided as-is for educational purposes. It reverse-engineers the SecureAI MVP codebase and teaches software engineering principles through code analysis.

---

**Start learning**: [Executive Summary](00_EXECUTIVE_SUMMARY.md) → [Repository Map](01_REPOSITORY_MAP.md)
