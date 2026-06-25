# Cerebrum Roadmap

This roadmap covers work **beyond Phase 5**. Phases 1-5 deliver a fully-featured two-tier memory system with advanced features (promotion, decay, summarization, scope filtering). Phase 6 focuses on production hardening and persistence.

---

## Phase 6: Production Hardening & LanceDB Integration

**Status:** Planned (Ready to implement)

Phase 6 transitions Cerebrum from a prototype with in-memory storage to a production-ready system with persistent vector database storage.

### Deliverables

- **LanceDB Integration** — Replace in-memory Cortex with persistent vector database
- **FastEmbed Integration** — Real embeddings with BGE-small model (384-dim)
- **Embedding Migration** — Tooling to handle embedding model changes
- **Observability** — Structured logging, metrics, and tracing
- **Error Handling & Resilience** — Retry logic, circuit breaker, graceful degradation
- **Orchestrator Updates** — Support configurable backends
- **Integration Tests** — 20 comprehensive tests for Phase 6 features
- **Documentation** — Production deployment guide, LanceDB setup, observability guide

### Architecture Decisions

See `docs/adr-phase-6.md` for detailed Architecture Decision Records:
- **ADR-001:** LanceDB for Persistent Cortex Storage
- **ADR-002:** FastEmbed for Real Embeddings
- **ADR-003:** Embedding Migration Strategy
- **ADR-004:** Error Handling and Resilience

### Implementation Plan

See `docs/phase-6-plan.md` for detailed 8-step implementation plan:
1. LanceDB Integration Foundation
2. FastEmbed Integration
3. Embedding Migration Tooling
4. Observability & Logging
5. Error Handling & Resilience
6. Orchestrator Updates
7. Integration Tests
8. Documentation & Release

### Success Criteria

- ✅ LanceDB Cortex fully functional and tested
- ✅ FastEmbed embeddings working with BGE-small model
- ✅ Embedding migration tooling available and tested
- ✅ Comprehensive observability in place
- ✅ Error handling and resilience patterns implemented
- ✅ All 80 new tests passing
- ✅ 90%+ code coverage maintained
- ✅ Zero clippy warnings
- ✅ Production deployment guide complete
- ✅ Project at 100% completion (5 of 5 phases)

---

## Phase 7+: Advanced Features & Scaling

**Status:** Future (Post-Phase 6)

After Phase 6 completes production hardening, future phases may include:

### Intelligence Layer

- **Multi-model embedding support** — Support multiple embedding models simultaneously
- **Distributed deployment** — Multiple servers with shared Cortex
- **Advanced analytics** — Memory usage patterns, agent insights
- **Agent-specific optimization** — Per-agent memory tuning
- **Conflict resolution** — Handle contradictory memories
- **Semantic deduplication** — Identify and merge similar memories

### Operational Features

- **Docker containerization** — Container images for deployment
- **Kubernetes manifests** — Helm charts for orchestration
- **Monitoring and alerting** — Prometheus metrics, alerting rules
- **Performance tuning** — Benchmarking, optimization
- **Security hardening** — Authentication, encryption, audit logging

### Research Directions

- **Temporal reasoning** — Time-aware memory retrieval
- **Causal inference** — Understanding memory relationships
- **Active learning** — System-initiated clarification questions
- **Memory compression** — Lossless compression of long-term memories
- **Cross-agent learning** — Shared insights between agents

---

## Guiding Principles (All Phases)

1. **The agent must not see the seams.** Tiering stays an implementation detail; the tool surface should remain stable.
2. **Every durable memory carries provenance.** Automatic decisions must be auditable and reversible.
3. **Backward compatibility.** New phases must not break existing MCP tools or agent integrations.
4. **Production-ready quality.** All code must meet 90%+ coverage, zero clippy warnings, comprehensive tests.
5. **Clear documentation.** Every phase includes architecture docs, ADRs, and deployment guides.
