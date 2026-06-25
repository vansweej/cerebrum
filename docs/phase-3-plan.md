# Phase 3: Implement Memory Tiers and Orchestrator

## Commit Message
```
feat: Phase 3 - Implement Synapse and Cortex memory tiers with orchestrator

- Implement Synapse tier: in-memory HashMap-based short-term storage
- Implement Cortex tier: LanceDB-backed persistent long-term storage
- Implement Orchestrator: blended search, promotion logic, and tool routing
- Add comprehensive integration tests for tier interactions
- Update architecture documentation with tier implementation details
- Achieve ≥90% code coverage across all tiers
```

## Overview
Phase 3 focuses on implementing the two memory tiers (Synapse and Cortex) and the Orchestrator that coordinates them. This phase brings the core memory system to life.

## Architecture

### Synapse Tier (Short-term Memory)
- **Storage:** In-memory HashMap keyed by MemoryId
- **Scope:** Per-session/interaction context
- **Lifecycle:** Cleared when session ends or manually purged
- **Operations:**
  - `store(entry)` — Add/update memory in HashMap
  - `retrieve(query, limit)` — Semantic search using embeddings
  - `delete(id)` — Remove memory by ID
  - `clear()` — Clear all memories (session end)
  - `list()` — List all memories (for debugging/inspection)

### Cortex Tier (Long-term Memory)
- **Storage:** LanceDB vector database
- **Scope:** Cross-session/global persistence
- **Lifecycle:** Durable; survives server restarts
- **Schema:**
  - `id: String` — MemoryId as string
  - `content: String` — Memory text
  - `embedding: Vec<f32>` — 384-dimensional vector
  - `salience: f32` — Importance score
  - `timestamp: i64` — Unix timestamp
  - `metadata: String` — JSON-serialized metadata
  - `source_session_id: Option<String>` — Session origin
- **Operations:**
  - `store(entry)` — Upsert memory into LanceDB
  - `retrieve(query, limit)` — Vector similarity search
  - `delete(id)` — Remove memory by ID
  - `search_by_salience(limit)` — Retrieve top memories by salience

### Orchestrator
- **Responsibility:** Coordinate Synapse and Cortex, expose unified tool interface
- **Operations:**
  - `remember(content, metadata)` — Store in Synapse
  - `recall(query, limit)` — Blended search across both tiers
  - `memorize(id)` — Promote memory from Synapse to Cortex
  - `forget(id)` — Delete from both tiers
  - `end_session()` — Clear Synapse, optionally promote high-salience memories

## Steps

### Step 1: Create Synapse Tier Implementation

**File:** `crates/cerebrum-core/src/synapse.rs` (new file)

Implement `SynapseMemory` struct:
```rust
pub struct SynapseMemory {
    memories: Arc<RwLock<HashMap<MemoryId, MemoryEntry>>>,
}

impl SynapseMemory {
    pub fn new() -> Self { ... }
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> { ... }
    pub async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> { ... }
    pub async fn delete(&self, id: &MemoryId) -> Result<()> { ... }
    pub async fn clear(&self) -> Result<()> { ... }
    pub async fn list(&self) -> Result<Vec<MemoryEntry>> { ... }
}
```

Implement `MemoryStore` trait for `SynapseMemory`.

**Key Features:**
- Thread-safe using `Arc<RwLock<HashMap>>`
- Semantic search using embeddings (cosine similarity)
- Salience-based ranking

### Step 2: Create Cortex Tier Implementation

**File:** `crates/cerebrum-core/src/cortex.rs` (new file)

Add `lancedb` dependency to `Cargo.toml`.

Implement `CortexMemory` struct:
```rust
pub struct CortexMemory {
    db: lancedb::db::Database,
    table: lancedb::db::Table,
}

impl CortexMemory {
    pub async fn new(db_path: &str) -> Result<Self> { ... }
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> { ... }
    pub async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> { ... }
    pub async fn delete(&self, id: &MemoryId) -> Result<()> { ... }
    pub async fn search_by_salience(&self, limit: usize) -> Result<Vec<MemoryEntry>> { ... }
}
```

Implement `MemoryStore` trait for `CortexMemory`.

**Key Features:**
- LanceDB vector database for persistent storage
- Vector similarity search using embeddings
- Salience-based ranking
- Metadata storage as JSON

### Step 3: Create Orchestrator

**File:** `crates/cerebrum-core/src/orchestrator.rs` (new file)

Implement `MemoryOrchestrator` struct:
```rust
pub struct MemoryOrchestrator {
    synapse: Arc<SynapseMemory>,
    cortex: Arc<CortexMemory>,
    embedder: Arc<dyn Embedder>,
}

impl MemoryOrchestrator {
    pub async fn new(cortex_db_path: &str, embedder: Arc<dyn Embedder>) -> Result<Self> { ... }
    
    // Tool implementations
    pub async fn remember(&self, content: String, metadata: HashMap<String, String>) -> Result<MemoryId> { ... }
    pub async fn recall(&self, query: String, limit: usize) -> Result<Vec<MemoryEntry>> { ... }
    pub async fn memorize(&self, id: MemoryId) -> Result<()> { ... }
    pub async fn forget(&self, id: MemoryId) -> Result<()> { ... }
    pub async fn end_session(&self, auto_promote_threshold: f32) -> Result<()> { ... }
}
```

**Key Features:**
- Blended search: query both tiers, merge and rank results
- Automatic embedding generation for stored memories
- Promotion logic: move high-salience memories from Synapse to Cortex
- Session lifecycle management

### Step 4: Update Module Exports

**File:** `crates/cerebrum-core/src/lib.rs`

Export new modules:
```rust
pub mod cortex;
pub mod orchestrator;
pub mod synapse;

pub use cortex::CortexMemory;
pub use orchestrator::MemoryOrchestrator;
pub use synapse::SynapseMemory;
```

### Step 5: Write Integration Tests

**File:** `crates/cerebrum-core/tests/tier_integration_tests.rs` (new file)

Test coverage:
- Synapse store/retrieve/delete operations
- Cortex store/retrieve/delete operations
- Blended recall across both tiers
- Promotion from Synapse to Cortex
- Forget operation across both tiers
- Session lifecycle (end_session with auto-promotion)
- Salience-based ranking
- Embedding generation and storage
- Concurrent access to Synapse (thread safety)

Target: ≥20 integration tests

### Step 6: Update Architecture Documentation

**File:** `docs/architecture.md`

Add new sections:
- **Synapse Tier Implementation** — In-memory storage details, thread safety, search algorithm
- **Cortex Tier Implementation** — LanceDB schema, vector search, persistence
- **Orchestrator** — Blended search algorithm, promotion logic, tool routing
- **Data Flow Diagrams** — Store, retrieve, promote, forget workflows

### Step 7: Verify Coverage and Quality

Run:
```bash
nix develop . --command cargo fmt
nix develop . --command cargo clippy -- -D warnings
nix develop . --command cargo test
nix develop . --command cargo tarpaulin
```

Ensure:
- All tests pass
- Coverage ≥90%
- No clippy warnings
- Code is formatted

## Acceptance Criteria

- [x] `SynapseMemory` implemented with HashMap-based storage
- [x] `CortexMemory` implemented with LanceDB-backed storage
- [x] `MemoryOrchestrator` implemented with blended search and promotion logic
- [x] All three components implement `MemoryStore` trait
- [x] Automatic embedding generation for stored memories
- [x] Comprehensive integration tests (≥20 tests)
- [x] All code formatted, linted, and tested
- [x] Coverage ≥90%
- [x] Architecture documentation updated
- [x] Commit pushed to `phase-3-memory-tiers` branch

## Dependencies Added

- `lancedb` — Embedded vector database for Cortex tier
- `tokio` — Already present, used for async operations
- `parking_lot` — For efficient RwLock (optional optimization)

## Notes

- Synapse uses `Arc<RwLock<HashMap>>` for thread-safe concurrent access
- Cortex uses LanceDB's built-in concurrency handling
- Embeddings are generated on-demand during store operations
- Blended recall merges results from both tiers and ranks by relevance + salience
- Promotion logic can be customized via `auto_promote_threshold` parameter
- Session lifecycle is managed by the Orchestrator (Synapse cleared on session end)

## Future Enhancements (Phase 5)

- Automatic promotion based on access frequency/recency
- Decay/forgetting of stale memories in Cortex
- Summarization on promotion (distill verbose memories)
- Identity & scope model (per-agent, per-user, global)
- Real embedding strategy hardening (pluggable backends)
