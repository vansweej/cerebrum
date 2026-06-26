# Spar Decision Brief

## Feature
Real persistent long-term (Cortex) memory for cerebrum-mcp, backed by an on-disk LanceDB store, replacing the current in-memory `Vec` that loses all data on process exit.

## Key decisions made
- **DB path:** CWD-relative `./data/cerebrum` via a new `Config` struct (mirrors athenaeum `config.rs:44`); resolves against the MCP-launch CWD. Replaces the hard-coded `/tmp/cerebrum_cortex` at `crates/cerebrum/src/main.rs:19`.
- **Directory creation:** mirror athenaeum — no `create_dir_all`; rely on LanceDB lazy creation.
- **Connection handling:** cache the `Connection` only; re-open the table per-operation (athenaeum `crates/core/src/store.rs:119-125`). Corrects an earlier mistake that proposed caching the `Table` handle (staleness risk).
- **Upsert/`store()`:** use LanceDB 0.30 native `merge_insert(&["id"])` + `when_matched_update_all`/`when_not_matched_insert_all` — single-transaction atomic upsert (lancedb-0.30.0 `src/table.rs:1062-1142`). Supersedes the earlier add-then-delete plan.
- **Ranking:** ship `exact` mode first — full-scan blend `0.7*similarity + 0.3*salience` for parity with `crates/cerebrum-core/src/lancedb_cortex.rs:207`. `approx` (vector ANN + re-rank) deferred.
- **Embedder dimension guard:** add `fn dimension(&self)` to the `Embedder` trait (`crates/cerebrum-core/src/traits.rs:7-10`); assert it equals `config.embedding_dim` in `LanceDBCortex::new` to fail-fast instead of corrupting inserts.
- **Trait method overrides:** `LanceDBCortex` MUST override `list()`/`len()`/`is_empty()` — the trait defaults (`traits.rs:33-49`) embed the literal `"*"` and use `usize::MAX`.
- **Reuse:** existing `LanceDBMemoryRecord` + `from_entry`/`to_entry`/`parse_scope_string` (`lancedb_cortex.rs:14-96`) kept unchanged as the Arrow<->model seam.
- **Deps:** none added — `lancedb 0.30`, `arrow-array/schema 58`, `futures` already in `crates/cerebrum-core/Cargo.toml:14,18-20`.
- **Future direction (noted, not scheduled):** after two working concrete stores exist, extract the shared vector-store mechanics (connect/table/search/merge/quote) into a crate shared by cerebrum + athenaeum, generic over a `Record` trait.

## Open questions
- Does `lancedb::connect` reliably create the nested `./data/` parent on a fresh checkout, or only the leaf? (Athenaeum works with `./data/athenaeum`, suggesting yes, but unconfirmed for an empty parent.)
- Which embedder is the production target (`MockEmbedder` vs `fastembed_embedder`), and is its output dim 384? Determines the `embedding_dim` default and the dimension-guard value.
- `merge_insert` doc warns rows "may appear to be reordered" (`table.rs:1082-1084`) — confirm nothing in retrieval depends on insertion order (it should not, since results are ranked).
- Scope filter pushdown: exact predicate form for `MemoryScope::matches` semantics (`crates/cerebrum-core/src/models.rs:68-76`), especially the Global-matches-all case.

## Rejected alternatives
- **Caching the `Table` handle** — rejected; risks stale snapshots missing committed writes (athenaeum re-opens per call).
- **add-then-delete-old upsert** — superseded by native atomic `merge_insert`.
- **delete-then-add (athenaeum's current `upsert_doc`)** — not crash-atomic; rejected for cerebrum.
- **Defensive `create_dir_all`** — rejected to mirror athenaeum and keep the storage layer convergent for the future shared crate.
- **`approx` vector-ANN ranking as v1** — deferred; can silently drop high-salience/low-similarity memories without a full scan.
- **Early extraction to a shared crate** — rejected as premature; extract after two working copies reveal the true seam.

## Risks identified
1. **Embedder/schema dimension mismatch** (high) — swapping embedders without matching `embedding_dim` corrupts inserts. Mitigated by the `dimension()` guard.
2. **Default-trait `list()`/`len()` trap** (high) — forgetting to override embeds `"*"` and silently returns wrong data. Mitigated by explicit overrides + a targeted test.
3. **CWD fragility** (medium) — if the MCP client does not set `cwd`, `./data/cerebrum` lands somewhere unexpected/unwritable. Shared with athenaeum.
4. **Schema as new compatibility surface** (medium) — changing columns/dim breaks existing on-disk tables; `migration.rs` covers embedding, not schema, migration. No schema-version field in v1.
5. **Ranking cost** (low/medium) — `exact` full-scan blend is O(n); fine at small N, revisit with an index.
6. **Untested path-resolution** (low) — tests use absolute `tempdir()` paths, bypassing the relative-resolution logic. Mitigated by a scoped-CWD unit test.

## Recommended next steps
1. Add `Embedder::dimension()` and the startup assertion (smallest change, unblocks the schema).
2. Rework `LanceDBCortex` struct to hold `Connection`; add `schema(dim)` + per-call `table()`.
3. Implement `store` via `merge_insert`, `retrieve`/`retrieve_by_scope` via scan+blend, `delete`/`len`/`list` against the table; override the three default trait methods.
4. Add `Config` + wire `main.rs`/orchestrator; migrate test call-sites to `tempdir()`.
5. Add the reopen-persistence test (store -> drop -> reopen -> assert present) — the test that would have caught the original gap.
