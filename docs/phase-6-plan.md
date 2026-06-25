# Phase 6: Production Hardening & LanceDB Integration

## Overview

Phase 6 transitions Cerebrum from a prototype with in-memory storage to a production-ready system with persistent vector database storage. This phase implements LanceDB integration for the Cortex tier, adds real embedding support, and hardens the system for deployment.

## Goals

1. **Replace in-memory Cortex with LanceDB** — Persistent vector database for long-term memory
2. **Add real embedding support** — FastEmbed integration with BGE-small model (384-dim)
3. **Implement embedding migration** — Tooling to handle embedding model changes
4. **Add observability** — Logging, metrics, and tracing for production monitoring
5. **Harden error handling** — Comprehensive error recovery and graceful degradation
6. **Performance optimization** — Batch operations, connection pooling, caching

## Architecture

### Current State (Phase 5)

```
┌─────────────────────────────────────────┐
│         MCP Agent                       │
└────────────────┬────────────────────────┘
                 │
        ┌────────▼────────┐
        │  Cerebrum MCP   │
        │    Server       │
        └────────┬────────┘
                 │
        ┌────────▼────────────────┐
        │ MemoryOrchestrator      │
        │ - recall_by_scope()     │
        │ - remember()            │
        │ - recall()              │
        │ - memorize()            │
        │ - forget()              │
        └────────┬────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
   ┌────▼────┐      ┌────▼──────┐
   │ Synapse │      │  Cortex   │
   │ (Memory)│      │ (Memory)  │
   └─────────┘      └───────────┘
```

### Phase 6 Target State

```
┌─────────────────────────────────────────┐
│         MCP Agent                       │
└────────────────┬────────────────────────┘
                 │
        ┌────────▼────────┐
        │  Cerebrum MCP   │
        │    Server       │
        └────────┬────────┘
                 │
        ┌────────▼────────────────┐
        │ MemoryOrchestrator      │
        │ - recall_by_scope()     │
        │ - remember()            │
        │ - recall()              │
        │ - memorize()            │
        │ - forget()              │
        └────────┬────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
   ┌────▼────┐      ┌────▼──────────────┐
   │ Synapse │      │  Cortex           │
   │ (Memory)│      │ (LanceDB)         │
   └─────────┘      │ - Vector DB       │
                    │ - Persistence     │
                    │ - Batch ops       │
                    └───────────────────┘
                            │
                    ┌───────▼────────┐
                    │ FastEmbed      │
                    │ (BGE-small)    │
                    │ 384-dim        │
                    └────────────────┘
```

## Implementation Plan

### Phase 6 Step 1: LanceDB Integration Foundation

**Objective:** Add LanceDB as a dependency and create a new `LanceDBCortex` implementation.

**Deliverables:**
- Add `lancedb` crate to `Cargo.toml` with pinned version
- Create `crates/cerebrum-core/src/lancedb_cortex.rs` with `LanceDBCortex` struct
- Implement `MemoryStore` trait for LanceDB backend
- Add 10 unit tests for LanceDB operations
- Maintain backward compatibility with existing `CortexMemory` (in-memory fallback)

**Key Decisions:**
- Use LanceDB's Rust SDK for embedded vector database
- Store embeddings in LanceDB table with schema: `{id, content, embedding, salience, scope, timestamp, ...}`
- Implement connection pooling for concurrent access
- Use async/await throughout for non-blocking I/O

**Files to Create/Modify:**
- `Cargo.toml` — Add lancedb dependency
- `crates/cerebrum-core/src/lancedb_cortex.rs` — NEW
- `crates/cerebrum-core/src/lib.rs` — Export LanceDBCortex
- `crates/cerebrum-core/tests/lancedb_tests.rs` — NEW (10 tests)

**Commit Message:** `"feat(phase-6): add LanceDB integration foundation"`

---

### Phase 6 Step 2: FastEmbed Integration

**Objective:** Replace MockEmbedder with real FastEmbed embeddings (BGE-small, 384-dim).

**Deliverables:**
- Add `fastembed` crate to `Cargo.toml`
- Create `crates/cerebrum-core/src/fastembed_embedder.rs` with `FastEmbedEmbedder` struct
- Implement `Embedder` trait for FastEmbed backend
- Add 10 unit tests for embedding generation and consistency
- Keep `MockEmbedder` for testing (behind feature flag or test-only)

**Key Decisions:**
- Use BGE-small model (384-dim) as default
- Cache embeddings to avoid recomputation
- Handle model download and initialization gracefully
- Support offline mode with vendored model weights (future)

**Files to Create/Modify:**
- `Cargo.toml` — Add fastembed dependency
- `crates/cerebrum-core/src/fastembed_embedder.rs` — NEW
- `crates/cerebrum-core/src/lib.rs` — Export FastEmbedEmbedder
- `crates/cerebrum-core/src/embedder.rs` — Keep MockEmbedder for tests
- `crates/cerebrum-core/tests/fastembed_tests.rs` — NEW (10 tests)

**Commit Message:** `"feat(phase-6): add FastEmbed integration with BGE-small model"`

---

### Phase 6 Step 3: Embedding Migration Tooling

**Objective:** Provide tooling to handle embedding model changes without data loss.

**Deliverables:**
- Create `crates/cerebrum-core/src/migration.rs` with `EmbeddingMigration` trait
- Implement `ReembedMigration` strategy (re-embed all memories with new model)
- Implement `PreserveMigration` strategy (keep old embeddings, add new ones)
- Add 10 unit tests for migration strategies
- Create CLI tool `cerebrum-migrate` for running migrations

**Key Decisions:**
- Support multiple migration strategies for different use cases
- Preserve provenance: track which model generated each embedding
- Implement dry-run mode to preview changes
- Add rollback capability

**Files to Create/Modify:**
- `crates/cerebrum-core/src/migration.rs` — NEW
- `crates/cerebrum/src/bin/migrate.rs` — NEW (CLI tool)
- `crates/cerebrum-core/tests/migration_tests.rs` — NEW (10 tests)
- `docs/migration.md` — NEW (migration guide)

**Commit Message:** `"feat(phase-6): add embedding migration tooling"`

---

### Phase 6 Step 4: Observability & Logging

**Objective:** Add comprehensive logging, metrics, and tracing for production monitoring.

**Deliverables:**
- Add `tracing` and `tracing-subscriber` crates
- Add structured logging to all major operations (remember, recall, memorize, forget, promote, decay)
- Implement metrics collection (operation counts, latencies, error rates)
- Add 10 unit tests for logging and metrics
- Create observability guide in docs

**Key Decisions:**
- Use `tracing` for structured logging (compatible with OpenTelemetry)
- Log at INFO level for user-facing operations, DEBUG for internal details
- Collect metrics: operation latency, memory tier sizes, promotion/decay rates
- Support multiple output formats (JSON, text)

**Files to Create/Modify:**
- `Cargo.toml` — Add tracing dependencies
- `crates/cerebrum-core/src/observability.rs` — NEW
- `crates/cerebrum/src/main.rs` — Initialize tracing
- `crates/cerebrum-core/tests/observability_tests.rs` — NEW (10 tests)
- `docs/observability.md` — NEW

**Commit Message:** `"feat(phase-6): add observability and structured logging"`

---

### Phase 6 Step 5: Error Handling & Resilience

**Objective:** Harden error handling with comprehensive recovery strategies.

**Deliverables:**
- Expand `crates/cerebrum-core/src/error.rs` with detailed error types
- Implement retry logic with exponential backoff for transient failures
- Add circuit breaker pattern for LanceDB connection failures
- Implement graceful degradation (fall back to Synapse if Cortex unavailable)
- Add 10 unit tests for error scenarios

**Key Decisions:**
- Use `thiserror` crate for ergonomic error types
- Implement retry with jitter to avoid thundering herd
- Circuit breaker: fail fast after N consecutive failures, retry after timeout
- Graceful degradation: if Cortex unavailable, use Synapse-only mode

**Files to Create/Modify:**
- `Cargo.toml` — Add thiserror dependency
- `crates/cerebrum-core/src/error.rs` — Expand error types
- `crates/cerebrum-core/src/resilience.rs` — NEW (retry, circuit breaker)
- `crates/cerebrum-core/tests/resilience_tests.rs` — NEW (10 tests)

**Commit Message:** `"feat(phase-6): add comprehensive error handling and resilience patterns"`

---

### Phase 6 Step 6: Orchestrator Updates

**Objective:** Update MemoryOrchestrator to use LanceDB Cortex and FastEmbed.

**Deliverables:**
- Update `MemoryOrchestrator::new()` to accept configurable Cortex backend
- Add `with_lancedb_cortex()` builder method
- Add `with_fastembed()` builder method
- Update all tool handlers to use new backends
- Add 10 unit tests for orchestrator with new backends

**Key Decisions:**
- Support both in-memory and LanceDB backends via trait objects
- Make backend selection configurable at startup
- Maintain backward compatibility with existing code

**Files to Create/Modify:**
- `crates/cerebrum-core/src/orchestrator.rs` — Update builder pattern
- `crates/cerebrum/src/main.rs` — Configure backends at startup
- `crates/cerebrum-core/tests/orchestrator_phase6_tests.rs` — NEW (10 tests)

**Commit Message:** `"feat(phase-6): update orchestrator for LanceDB and FastEmbed backends"`

---

### Phase 6 Step 7: Integration Tests

**Objective:** Comprehensive integration tests for Phase 6 features.

**Deliverables:**
- Create `crates/cerebrum-core/tests/phase6_integration_tests.rs` with 20 tests covering:
  - LanceDB persistence across restarts
  - FastEmbed consistency
  - Embedding migration workflows
  - Error recovery and resilience
  - Observability and logging
  - End-to-end workflows with new backends
- All tests passing with 100% coverage

**Files to Create/Modify:**
- `crates/cerebrum-core/tests/phase6_integration_tests.rs` — NEW (20 tests)

**Commit Message:** `"feat(phase-6): add 20 comprehensive integration tests"`

---

### Phase 6 Step 8: Documentation & Release

**Objective:** Complete Phase 6 documentation and prepare for release.

**Deliverables:**
- Add Phase 6 section to `docs/architecture.md`
- Create `docs/lancedb-setup.md` — LanceDB configuration and deployment guide
- Create `docs/embedding-models.md` — Embedding model selection and migration
- Create `docs/observability.md` — Logging, metrics, and tracing guide
- Update `PROGRESS.md` to mark Phase 6 complete (100% project)
- Update `README.md` with production deployment instructions

**Files to Create/Modify:**
- `docs/architecture.md` — Add Phase 6 section
- `docs/lancedb-setup.md` — NEW
- `docs/embedding-models.md` — NEW
- `docs/observability.md` — NEW
- `PROGRESS.md` — Mark Phase 6 complete
- `README.md` — Add production deployment section

**Commit Message:** `"feat(phase-6): complete documentation and mark phase 6 100% complete"`

---

## Test Coverage

- **10 LanceDB tests** — Database operations, persistence, schema
- **10 FastEmbed tests** — Embedding generation, consistency, caching
- **10 migration tests** — Re-embedding, preservation, rollback
- **10 observability tests** — Logging, metrics, tracing
- **10 resilience tests** — Retry logic, circuit breaker, graceful degradation
- **10 orchestrator tests** — Backend configuration, tool handlers
- **20 integration tests** — End-to-end workflows, persistence, recovery
- **Total: 80 new tests** (268 total project tests)

## Quality Gates

- ✅ All tests passing (100% success rate)
- ✅ 90%+ code coverage (tarpaulin)
- ✅ Zero clippy warnings
- ✅ All code formatted with `cargo fmt`
- ✅ Backward compatible with Phase 5 MCP tools

## Timeline

- **Step 1 (LanceDB):** 2-3 hours
- **Step 2 (FastEmbed):** 2-3 hours
- **Step 3 (Migration):** 2-3 hours
- **Step 4 (Observability):** 2-3 hours
- **Step 5 (Resilience):** 2-3 hours
- **Step 6 (Orchestrator):** 1-2 hours
- **Step 7 (Integration Tests):** 2-3 hours
- **Step 8 (Documentation):** 1-2 hours
- **Total: 14-22 hours** (estimated)

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| LanceDB API changes | High | Pin version, add integration tests, monitor releases |
| FastEmbed model download failures | Medium | Implement offline mode, vendor weights in flake |
| Embedding dimension mismatch | High | Add migration tooling, validate on startup |
| Performance regression | Medium | Benchmark before/after, optimize hot paths |
| Data loss during migration | Critical | Implement dry-run, backup, rollback capabilities |
| Production outages | High | Circuit breaker, graceful degradation, observability |

## Success Criteria

1. ✅ LanceDB Cortex fully functional and tested
2. ✅ FastEmbed embeddings working with BGE-small model
3. ✅ Embedding migration tooling available and tested
4. ✅ Comprehensive observability in place
5. ✅ Error handling and resilience patterns implemented
6. ✅ All 80 new tests passing
7. ✅ 90%+ code coverage maintained
8. ✅ Zero clippy warnings
9. ✅ Production deployment guide complete
10. ✅ Project at 100% completion (5 of 5 phases)

## Next Steps After Phase 6

- **Phase 7 (Optional):** Advanced features
  - Multi-model embedding support
  - Distributed deployment (multiple servers)
  - Advanced analytics and insights
  - Agent-specific memory optimization
  - Conflict resolution and merging
  - Semantic deduplication

- **Production Deployment:**
  - Docker containerization
  - Kubernetes manifests
  - Monitoring and alerting setup
  - Performance tuning
  - Security hardening

---

**Status:** Ready for implementation
**Estimated Completion:** 1-2 weeks
**Project Completion After Phase 6:** 100% (5 of 5 phases)
