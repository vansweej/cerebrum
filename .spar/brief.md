# Phase 6 Build Brief: Production Hardening & LanceDB Integration

**Status:** Phase 6 marked "complete" but contains three critical façades + unwired modules. This brief corrects the record and sequences real work.

**Scope:** Fix systemic defects where observability, resilience, decay, promotion, persistence, and embeddings are tested in isolation but never integrated or behaviorally validated.

---

## Critical Findings

### 1. LanceDB Persistence is Fake
- `lancedb_cortex.rs` holds a `HashMap`, not a `Connection`
- `CortexMemory` is a duplicate in-memory HashMap
- `with_lancedb_cortex()` discards the LanceDBCortex and uses CortexMemory instead
- **Result:** Memories do not survive process restart (docs claim they do)
- **Test:** `test_integration_lancedb_persistence` was rewritten to hide the failure rather than fix it

### 2. FastEmbed Integration is Fake
- `fastembed_embedder.rs` contains `PhantomData` + `DefaultHasher`/LCG, not a real model
- No `fastembed` crate in `Cargo.toml`
- Doc admits: *"deterministic hash-based approach similar to MockEmbedder"*
- **Result:** Semantic search doesn't work; `"dog"` and `"puppy"` hash to orthogonal random vectors

### 3. Embedder Mismatch Bug
- `synapse.rs:82` hardcodes `MockEmbedder::new()` for query embedding
- Storage uses the orchestrator's injected embedder
- **Result:** Query vectors (one hash family) don't match stored vectors (different hash family) even for identical text

### 4. Observability & Resilience are Unwired
- `observability.rs:198-208` (OperationTimer) and `resilience.rs:187-212` (CircuitBreaker) are fully tested in isolation
- Neither is called from the orchestrator
- **Result:** Metrics are never recorded; circuit breaker never guards real I/O

### 5. Tests Assert Mechanics, Not Behavior
- 309 green tests check counts and mechanics (e.g., `results.len() == 2`)
- Not one test asserts relevance (e.g., "querying *dog* ranks the dog-memory above an unrelated one")
- **Result:** Fake embedders and meaningless vectors pass all gates

### 6. Documentation is Active Misinformation
- `README`, `CHANGELOG`, `RELEASE_NOTES`, `PROJECT_COMPLETION_REPORT`, `HANDOFF` all assert persistence, FastEmbed, and "production-ready"
- v0.6.0 tag shipped with these claims
- **Result:** Anyone reading the repo or using that tag is misled

---

## Recommended Sequence

### Phase 0: Real Persistence (Prerequisite for all downstream work)
**Commit message:** `feat: implement real LanceDB persistence in Cortex`

1. Add missing deps to `Cargo.toml`: `arrow-array = "58"`, `arrow-schema = "58"`, `futures = "0.3"`, `tempfile = "3" [dev-deps]`
2. Rewrite `LanceDBCortex` internals:
   - Hold `Connection` + table name + embedding dimension
   - Port `connect`, `add`, `search`, `delete` from athenaeum reference (`athenaeum-mcp/crates/core/src/store.rs:94-292`)
   - Reuse existing `LanceDBMemoryRecord::{from_entry,to_entry,parse_scope_string}` scaffolding
   - Implement `MemoryStore` trait with SQL scope filter (hard access control) + over-fetch re-rank (soft salience blend: 0.7 cosine + 0.3 salience)
3. Delete `CortexMemory` class (or demote to `#[cfg(test)] InMemoryCortex` test double)
4. Update `orchestrator.rs`:
   - Change `Arc<CortexMemory>` → `Arc<dyn MemoryStore>`
   - Fix `with_lancedb_cortex()` to actually use the LanceDBCortex instead of discarding it
   - Resolve `len()` ripple (add to trait or compute via `list().len()`)
5. Un-mute and rewrite `test_integration_lancedb_persistence` as cross-instance restart test
6. Verify: store → drop → reopen → recall hits

**Acceptance:** Cross-instance restart test passes; all 184 tests green; coverage maintained

---

### Phase 1: Real Embedder (After Phase 0)
**Commit message:** `feat: integrate real Ollama/nomic embedder`

1. Rewrite `FastEmbedEmbedder` to call Ollama endpoint (`http://localhost:11434/api/embed`)
   - Use `nomic-embed-text` model (384-dimensional, same as BGE-small)
   - Lazy init on first use; cache client
   - Return real embeddings from Ollama
   - Keep `MockEmbedder` for tests (deterministic, fast)
2. Fix `synapse.rs:82`: use the orchestrator's injected embedder for query embedding, not hardcoded `MockEmbedder`
3. Derive embedding dimension from the embedder (not hardcoded 384); carry `dim` from embedder into `Store::open`
4. Add `validate_embedding_dimension` check at store boundary (port from athenaeum `store.rs:137-144`)
5. Add integration test that requires Ollama running (or skip gracefully if unavailable)

**Acceptance:** Real embeddings from Ollama; query and storage use same embedder; dimension validation enforced

---

### Phase 2: Behavioral Relevance Test
**Commit message:** `test: add semantic relevance assertions to memory recall`

1. Add test: store memories for `"dog"`, `"puppy"`, `"unrelated"` with real embedder
2. Query `"dog"` and assert:
   - `"dog"` memory ranks first (highest cosine similarity)
   - `"puppy"` memory ranks second (semantically close)
   - `"unrelated"` memory ranks last
3. Repeat with salience override: store `"unrelated"` with high salience, query `"dog"`, verify salience blend (0.7 cosine + 0.3 salience) produces correct ranking
4. Add to integration tests; run before Phase 3

**Acceptance:** Relevance test passes; confirms semantic search works end-to-end

---

### Phase 3: Wire Observability & Resilience
**Commit message:** `feat: instrument Cortex I/O with observability and resilience guards`

1. Add owned fields to `orchestrator.rs`:
   - `obs: ObservabilityContext`
   - `cortex_breaker: CircuitBreaker`
   - `retry: RetryConfig`
2. Add private `cortex_guarded<F,Fut,T>()` helper wrapping Cortex calls in breaker + retry
3. Instrument six methods (`remember`, `recall`, `recall_by_scope`, `memorize`, `forget`, `end_session`):
   - Wrap in `OperationTimer` + metrics
   - Route all Cortex I/O through `cortex_guarded`
4. Fix `recall` doc/code bug: "in parallel" comment but sequential awaits → use `tokio::try_join!`
5. Add `pub fn observability(&self)` accessor; update 1–2 integration tests to assert metrics increment

**Acceptance:** Observability and resilience modules are now driven by real orchestrator usage; metrics recorded; circuit breaker guards I/O

---

### Phase 4: Coverage Tests (Falls Out Naturally)
**Commit message:** `test: add coverage tests for defaults, trait dispatch, and strategy branches`

1. Add trivial default tests (e.g., `ObservabilityContext::default()`, `RetryConfig::default()`)
2. Add trait dispatch tests (e.g., `MemoryStore` implementations)
3. Add strategy branch tests (e.g., `RetryStrategy::Exponential` vs `Linear`)
4. Refactor dead `resilience.rs:205-206` unreachable branch (collapse None-while-Open case into timeout check; behavior-preserving)
5. Add 6 isolated resilience tests (timeout, backoff, state transitions)

**Acceptance:** Coverage ≥90% (target: 94%); all tests pass

---

### Phase 5: Correct Documentation
**Commit message:** `docs: correct persistence and embedder claims; retract v0.6.0 tag`

1. Update `README.md`: clarify that persistence is now real (LanceDB with cross-instance restart)
2. Update `RELEASE_NOTES.md`: note that FastEmbed is now real (BGE-small model)
3. Update `CHANGELOG.md`: add entries for Phases 0–4
4. Retract v0.6.0 tag (or tag v0.6.1 with corrections)
5. Add `ARCHITECTURE.md` section on embedder/persistence/observability integration

**Acceptance:** Docs match implementation; no active misinformation

---

## Decisions Made

1. **Embedder choice:** Use Ollama/nomic path (athenaeum's approach)
   - Lighter weight than fastembed crate (~100MB)
   - Requires Ollama running; more flexible for dev/prod
   - Porting from athenaeum reference is straightforward

2. **CortexMemory disposal:** Delete entirely
   - Unit tests should catch any breakage
   - Cleaner codebase; no conditional compilation
   - Orchestrator will use `Arc<dyn MemoryStore>` exclusively

3. **Sequence priority:** Phase 0 (persistence) first, then Phase 1 (embedder)
   - Most practical: persistence is prerequisite for real I/O to guard
   - Allows Phase 2 (relevance test) to validate both together
   - Observability/resilience wiring (Phase 3) then guards real LanceDB I/O

## Open Questions

1. **Salience blend weights (0.7/0.3):** Keep as-is or tune based on behavioral tests?
2. **Delete-by-predicate API:** Athenaeum never implemented `list()`; verify LanceDB 0.30 delete-by-SQL-predicate shape before porting
3. **Ollama endpoint:** Assume `http://localhost:11434` or make configurable?

---

## Risks

1. **LanceDB native build:** Athenaeum required `cmake` in `flake.nix`; verify cerebrum's build doesn't fail on linking
2. **Embedder model size:** BGE-small is ~100MB; lazy init may cause first-query latency spike
3. **Dimension mismatch:** If embedder dim ≠ stored vectors dim, queries will fail; validation is critical
4. **Backward compatibility:** Existing in-memory Cortex code may break when switching to `Arc<dyn MemoryStore>`; audit call sites

---

## Reference Implementation

- **LanceDB store:** `athenaeum-mcp/crates/core/src/store.rs` (lines 94–292)
- **Plan & decisions:** `athenaeum-mcp/.spar/plan.md` and `.spar/brief.md`
- **Real embedder:** `athenaeum-mcp/crates/core/src/embedder.rs` (FastEmbed + MockEmbedder split)

---

## Success Criteria

- [ ] Phase 0: Cross-instance restart test passes; persistence is real
- [ ] Phase 1: Real embeddings; query/storage use same embedder; dimension validation enforced
- [ ] Phase 2: Relevance test passes; semantic search works end-to-end
- [ ] Phase 3: Observability and resilience wired; metrics recorded; circuit breaker guards I/O
- [ ] Phase 4: Coverage ≥90%; all tests pass
- [ ] Phase 5: Docs corrected; no active misinformation
- [ ] All 184 tests pass; zero clippy warnings; `cargo fmt` clean
