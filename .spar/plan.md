# Feature: Real LanceDB Persistence for Cortex Long-Term Memory

## Context

Cerebrum's Cortex tier is designed as "long-term, persistent, vector-backed storage" but currently
stores everything in an in-memory `Arc<RwLock<Vec<LanceDBMemoryRecord>>>`. The LanceDB connection
is opened in `LanceDBCortex::new` and immediately dropped (`let _conn = …`). All data is lost on
process exit. This plan wires in real persistence using the same patterns as the sibling
`athenaeum-mcp` project.

See `.spar/brief.md` for the full decision record. Key decisions:

- **DB path:** CWD-relative `./data/cerebrum` via a new `Config` struct (mirrors athenaeum).
- **Connection:** cache the `lancedb::Connection`; re-open the `Table` per-operation (athenaeum pattern).
- **Upsert:** native `merge_insert(&["id"])` — single-transaction atomic upsert (LanceDB 0.30).
- **Ranking:** exact full-scan blend `0.7*similarity + 0.3*salience` for behavioural parity.
- **Schema guard:** `Embedder::dimension()` added to trait; asserted at construction time.
- **Constructor shape:** `LanceDBCortex::new(db_path: &Path, table_name: &str, dim: usize, embedder)`
  mirrors athenaeum's `Store::open`; `Config` consumed one level above in the orchestrator.
- **No `create_dir_all`:** mirror athenaeum — rely on LanceDB lazy directory creation.

---

## Phase 1: Widen Embedder trait with dimension()

Commit message: `feat(embedder): add dimension() to Embedder trait for schema safety`

### Step 1: Add dimension() to the Embedder trait

In `crates/cerebrum-core/src/traits.rs`, add a required method to the `Embedder` trait (after
`async fn embed`):

```rust
/// Return the dimensionality of vectors produced by this embedder.
fn dimension(&self) -> usize;
```

This is a synchronous, non-async fn. It must be a required method (no default), so any embedder
that does not implement it fails to compile, giving the schema guard its teeth.

### Step 2: Implement dimension() on MockEmbedder

In `crates/cerebrum-core/src/embedder.rs`, add to `impl Embedder for MockEmbedder`:

```rust
fn dimension(&self) -> usize {
    384
}
```

This matches the existing inherent `MockEmbedder::dimension()` at line 25.

### Step 3: Implement dimension() on FastEmbedEmbedder

In `crates/cerebrum-core/src/fastembed_embedder.rs`, add to `impl Embedder for FastEmbedEmbedder`:

```rust
fn dimension(&self) -> usize {
    384
}
```

This matches the existing inherent `FastEmbedEmbedder::embedding_dim()` at line 88.
The `nomic-embed-text` model produces 384-dimensional vectors (confirmed in `fastembed_embedder.rs:178`).

### Step 4: Fix any other impl Embedder blocks

Grep for `impl Embedder for` across the crate. Add `fn dimension(&self) -> usize { 384 }` to every
match. The `DefaultStore` in `traits.rs` tests implements `MemoryStore`, not `Embedder` — skip it.

---

## Phase 2: Introduce Config struct

Commit message: `feat(config): add Config with CWD-relative db_path`

### Step 1: Create crates/cerebrum-core/src/config.rs

Create the file with this exact content, mirroring athenaeum `crates/core/src/config.rs`:

```rust
//! Hardcoded configuration defaults for the single-user local build.
//!
//! `Config` holds all runtime parameters for the LanceDB storage path.
//! These are compile-time defaults — override any field by constructing the
//! struct directly (e.g. in tests, set `db_path` to a `tempdir()` path).

use std::path::PathBuf;

/// Configuration for `cerebrum-core` storage.
///
/// All fields have hardcoded defaults suitable for a single-user local
/// deployment where the MCP client controls the working directory.
/// Tests override `db_path` via `tempfile::tempdir()` to avoid
/// touching the production store.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the LanceDB database directory (CWD-relative by default).
    ///
    /// Resolved against the process working directory at startup.
    /// Set the MCP server's `cwd` to a durable project folder so
    /// `./data/cerebrum` lands somewhere persistent.
    pub db_path: PathBuf,
    /// Name of the LanceDB table that holds memories.
    pub table_name: String,
    /// Expected dimension of the embedding vectors.
    pub embedding_dim: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("./data/cerebrum"),
            table_name: "memories".to_string(),
            embedding_dim: 384,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_db_path_is_relative() {
        let config = Config::default();
        assert!(config.db_path.is_relative());
        assert_eq!(config.db_path, PathBuf::from("./data/cerebrum"));
    }

    #[test]
    fn default_table_name_is_memories() {
        assert_eq!(Config::default().table_name, "memories");
    }

    #[test]
    fn default_embedding_dim_is_384() {
        assert_eq!(Config::default().embedding_dim, 384);
    }
}
```

### Step 2: Register the module and re-export from lib.rs

In `crates/cerebrum-core/src/lib.rs`:
- Add `pub mod config;` after the existing `pub mod decay;` line.
- Add `pub use config::Config;` in the re-exports section alongside the other `pub use` lines.

---

## Phase 3: Rewrite LanceDBCortex with real persistence

Commit message: `feat(cortex): implement real LanceDB persistence for Cortex tier`

This phase rewrites `crates/cerebrum-core/src/lancedb_cortex.rs` in full. The file keeps the
existing `LanceDBMemoryRecord` struct (lines 15-32) and the `from_entry`/`to_entry`/`parse_scope_string`
helpers (lines 34-96) unchanged — they are the Arrow↔model seam and remain valid.
Everything from the struct definition downward is replaced.

### Step 1: Replace imports

Replace the current import block at the top of the file with:

```rust
use std::path::Path;
use std::sync::Arc;

use arrow_array::{
    array::ArrayRef,
    builder::{FixedSizeListBuilder, Float32Builder, StringBuilder},
    cast::AsArray,
    types::Float32Type,
    RecordBatch, RecordBatchIterator,
};
use arrow_schema::{DataType, Field, Fields, Schema};
use async_trait::async_trait;
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Connection, DistanceType, Table};
use serde::{Deserialize, Serialize};

use crate::embedder::Embedder;
use crate::error::{CerebrumError, Result};
use crate::models::{MemoryEntry, MemoryId, MemoryScope};
use crate::traits::MemoryStore;
```

Remove `use parking_lot::RwLock;` — it is no longer needed.

### Step 2: Add sql_quote helper

Add this free function immediately after the imports (before `LanceDBMemoryRecord`), mirroring
athenaeum `store.rs:33-35`:

```rust
/// Escape single quotes for safe insertion in a LanceDB SQL filter predicate.
///
/// Replaces every `'` with `''` and wraps the result in single quotes, making
/// the string safe to embed in `DELETE WHERE id = …` or `WHERE scope = …` filters.
fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
```

### Step 3: Add Arrow schema helper

Add this free function after `sql_quote` (before `LanceDBMemoryRecord`), mirroring athenaeum
`store.rs:58-73`. Columns map 1:1 from `LanceDBMemoryRecord`:

```rust
/// Build the Arrow schema for the `memories` table.
///
/// Column order must match the `RecordBatch` built in `store()`.
fn schema(dim: usize) -> Arc<Schema> {
    let vector_field = Field::new("item", DataType::Float32, true);
    let fields = vec![
        Field::new("id",                DataType::Utf8, false),
        Field::new("content",           DataType::Utf8, false),
        Field::new("salience",          DataType::Float32, false),
        Field::new("timestamp",         DataType::Utf8, false),
        Field::new("source_session_id", DataType::Utf8, true),   // nullable
        Field::new("scope",             DataType::Utf8, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(Arc::new(vector_field), dim as i32),
            false,
        ),
        Field::new("metadata_json",     DataType::Utf8, false),
    ];
    Arc::new(Schema::new(Fields::from(fields)))
}
```

### Step 4: Replace the LanceDBCortex struct

Replace the current struct (lines 102-116) with:

```rust
/// Persistent long-term memory storage backed by LanceDB (Cortex tier).
///
/// Stores memories in a vector database for efficient semantic search and
/// persistent storage across sessions. Supports salience-based ranking.
///
/// The LanceDB `Connection` is held for the lifetime of the store; the
/// `Table` handle is re-opened on each operation to avoid stale snapshots.
pub struct LanceDBCortex {
    /// Held LanceDB connection.
    conn: Connection,
    /// Table name for storing memories.
    table_name: String,
    /// Embedding dimension (384 for nomic-embed-text).
    embedding_dim: usize,
    /// Embedder for generating query embeddings.
    embedder: Arc<dyn Embedder>,
}
```

### Step 5: Replace the impl LanceDBCortex block

Replace the entire `impl LanceDBCortex` block with:

```rust
impl LanceDBCortex {
    /// Open (or create) the memories table at `db_path`.
    ///
    /// Mirrors athenaeum `Store::open`. Asserts that `embedder.dimension() == dim`
    /// at construction time to fail-fast before any schema-corrupting insert.
    ///
    /// # Arguments
    /// * `db_path`    – Path to the LanceDB directory (relative or absolute).
    /// * `table_name` – Name of the table within the database.
    /// * `dim`        – Expected embedding dimension; must match `embedder.dimension()`.
    /// * `embedder`   – Embedder used for query embedding during retrieval.
    pub async fn new(
        db_path: &Path,
        table_name: &str,
        dim: usize,
        embedder: Arc<dyn Embedder>,
    ) -> Result<Self> {
        // Fail-fast: embedder dimension must match schema dimension.
        let embedder_dim = embedder.dimension();
        if embedder_dim != dim {
            return Err(CerebrumError::Validation(format!(
                "Embedder dimension ({}) does not match schema dimension ({})",
                embedder_dim, dim
            )));
        }

        let path = db_path.to_str().ok_or_else(|| {
            CerebrumError::Database("non-UTF-8 db_path".to_string())
        })?;

        let conn = lancedb::connect(path)
            .execute()
            .await
            .map_err(|e| CerebrumError::Database(format!("Failed to connect to LanceDB: {}", e)))?;

        // Create the table if it does not yet exist.
        let existing = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| CerebrumError::Database(format!("Failed to list tables: {}", e)))?;

        if !existing.contains(&table_name.to_string()) {
            conn.create_empty_table(table_name, schema(dim))
                .execute()
                .await
                .map_err(|e| CerebrumError::Database(format!("Failed to create table: {}", e)))?;
        }

        Ok(Self {
            conn,
            table_name: table_name.to_string(),
            embedding_dim: dim,
            embedder,
        })
    }

    /// Open the table. Re-opened per operation to avoid stale snapshots.
    ///
    /// Mirrors athenaeum `Store::table()`.
    async fn table(&self) -> Result<Table> {
        self.conn
            .open_table(&self.table_name)
            .execute()
            .await
            .map_err(|e| CerebrumError::Database(format!("Failed to open table: {}", e)))
    }

    /// Calculate cosine similarity between two vectors.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }

    /// Build a single-row RecordBatch from a LanceDBMemoryRecord.
    fn record_to_batch(record: &LanceDBMemoryRecord, dim: usize) -> Result<RecordBatch> {
        let schema = schema(dim);

        let mut id_b           = StringBuilder::new();
        let mut content_b      = StringBuilder::new();
        let mut salience_b     = arrow_array::builder::Float32Builder::new();
        let mut timestamp_b    = StringBuilder::new();
        let mut session_b      = StringBuilder::new();
        let mut scope_b        = StringBuilder::new();
        let mut embedding_b    = FixedSizeListBuilder::new(Float32Builder::new(), dim as i32);
        let mut metadata_b     = StringBuilder::new();

        id_b.append_value(&record.id);
        content_b.append_value(&record.content);
        salience_b.append_value(record.salience);
        timestamp_b.append_value(&record.timestamp);
        match &record.source_session_id {
            Some(s) => session_b.append_value(s),
            None    => session_b.append_null(),
        }
        scope_b.append_value(&record.scope);
        for &v in &record.embedding {
            embedding_b.values().append_value(v);
        }
        embedding_b.append(true);
        metadata_b.append_value(&record.metadata_json);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_b.finish())        as ArrayRef,
                Arc::new(content_b.finish())   as ArrayRef,
                Arc::new(salience_b.finish())  as ArrayRef,
                Arc::new(timestamp_b.finish()) as ArrayRef,
                Arc::new(session_b.finish())   as ArrayRef,
                Arc::new(scope_b.finish())     as ArrayRef,
                Arc::new(embedding_b.finish()) as ArrayRef,
                Arc::new(metadata_b.finish())  as ArrayRef,
            ],
        )
        .map_err(|e| CerebrumError::Database(format!("Failed to build RecordBatch: {}", e)))?;

        Ok(batch)
    }

    /// Decode a RecordBatch into a Vec of LanceDBMemoryRecord.
    fn batch_to_records(batch: &RecordBatch) -> Result<Vec<LanceDBMemoryRecord>> {
        let n = batch.num_rows();
        if n == 0 {
            return Ok(vec![]);
        }

        let id_col       = batch.column_by_name("id").ok_or_else(|| CerebrumError::Database("missing 'id' column".into()))?.as_string::<i32>();
        let content_col  = batch.column_by_name("content").ok_or_else(|| CerebrumError::Database("missing 'content' column".into()))?.as_string::<i32>();
        let salience_col = batch.column_by_name("salience").ok_or_else(|| CerebrumError::Database("missing 'salience' column".into()))?.as_primitive::<Float32Type>();
        let ts_col       = batch.column_by_name("timestamp").ok_or_else(|| CerebrumError::Database("missing 'timestamp' column".into()))?.as_string::<i32>();
        let session_col  = batch.column_by_name("source_session_id").ok_or_else(|| CerebrumError::Database("missing 'source_session_id' column".into()))?.as_string::<i32>();
        let scope_col    = batch.column_by_name("scope").ok_or_else(|| CerebrumError::Database("missing 'scope' column".into()))?.as_string::<i32>();
        let emb_col      = batch.column_by_name("embedding").ok_or_else(|| CerebrumError::Database("missing 'embedding' column".into()))?.as_fixed_size_list();
        let meta_col     = batch.column_by_name("metadata_json").ok_or_else(|| CerebrumError::Database("missing 'metadata_json' column".into()))?.as_string::<i32>();

        let mut records = Vec::with_capacity(n);
        for i in 0..n {
            let emb_values = emb_col.value(i);
            let emb_f32 = emb_values.as_primitive::<Float32Type>();
            let embedding: Vec<f32> = (0..emb_f32.len()).map(|j| emb_f32.value(j)).collect();

            records.push(LanceDBMemoryRecord {
                id:                id_col.value(i).to_string(),
                content:           content_col.value(i).to_string(),
                salience:          salience_col.value(i),
                timestamp:         ts_col.value(i).to_string(),
                source_session_id: if session_col.is_null(i) { None } else { Some(session_col.value(i).to_string()) },
                scope:             scope_col.value(i).to_string(),
                embedding,
                metadata_json:     meta_col.value(i).to_string(),
            });
        }
        Ok(records)
    }

    /// Search memories by salience (highest first).
    pub async fn search_by_salience(&self, limit: usize) -> Result<Vec<MemoryEntry>> {
        let table = self.table().await?;
        let row_count = table.count_rows(None).await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        if row_count == 0 {
            return Ok(vec![]);
        }

        let stream = table.query().execute().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        let batches: Vec<RecordBatch> = stream.try_collect().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;

        let mut records: Vec<LanceDBMemoryRecord> = batches.iter()
            .flat_map(|b| Self::batch_to_records(b).unwrap_or_default())
            .collect();

        records.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));

        records.iter()
            .take(limit)
            .map(|r| r.to_entry())
            .collect()
    }
}
```

### Step 6: Replace the MemoryStore impl block

Replace the entire `#[async_trait] impl MemoryStore for LanceDBCortex` block with the following.
All five methods in the trait (`store`, `retrieve`, `retrieve_by_scope`, `delete`) are implemented,
plus explicit overrides of `list`, `len`, and `is_empty` — the default implementations in the trait
would embed the literal `"*"` string and are incorrect for a real store.

```rust
#[async_trait]
impl MemoryStore for LanceDBCortex {
    /// Store a memory entry using an atomic upsert keyed on `id`.
    ///
    /// Uses LanceDB `merge_insert` so re-storing the same `MemoryId` updates
    /// the existing row rather than duplicating it, with no crash window.
    async fn store(&self, entry: MemoryEntry) -> Result<()> {
        let record = LanceDBMemoryRecord::from_entry(&entry)?;
        let schema = schema(self.embedding_dim);
        let batch  = Self::record_to_batch(&record, self.embedding_dim)?;

        let reader = RecordBatchIterator::new(
            vec![Ok(batch)],
            schema,
        );

        let table = self.table().await?;
        let mut mi = table.merge_insert(&["id"]);
        mi.when_matched_update_all(None).when_not_matched_insert_all();
        mi.execute(Box::new(reader))
            .await
            .map_err(|e| CerebrumError::Database(format!("merge_insert failed: {}", e)))?;

        Ok(())
    }

    /// Retrieve memories by semantic similarity, blended with salience.
    ///
    /// Performs an exact full-scan so that the blend `0.7*similarity + 0.3*salience`
    /// is computed over every row — no memory can be dropped by a vector pre-filter.
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let query_embedding = self.embedder.embed(query).await?;

        let table = self.table().await?;
        let row_count = table.count_rows(None).await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        if row_count == 0 {
            return Ok(vec![]);
        }

        let stream = table.query().execute().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        let batches: Vec<RecordBatch> = stream.try_collect().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;

        let mut scored: Vec<(LanceDBMemoryRecord, f32)> = batches.iter()
            .flat_map(|b| Self::batch_to_records(b).unwrap_or_default())
            .map(|record| {
                let sim   = Self::cosine_similarity(&query_embedding, &record.embedding);
                let score = sim * 0.7 + record.salience * 0.3;
                (record, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter()
            .take(limit)
            .map(|(r, _)| r.to_entry())
            .collect()
    }

    /// Retrieve memories filtered by scope, then blended-score ranked.
    ///
    /// Pushes a coarse SQL predicate to LanceDB (reducing rows fetched),
    /// then applies the precise `MemoryScope::matches` logic in Rust to
    /// handle the bidirectional Global-matches-all semantic.
    async fn retrieve_by_scope(
        &self,
        query: &str,
        scope: &MemoryScope,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>> {
        let query_embedding = self.embedder.embed(query).await?;

        let table = self.table().await?;
        let row_count = table.count_rows(None).await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        if row_count == 0 {
            return Ok(vec![]);
        }

        // Coarse SQL pushdown: global matches all, specific scopes match themselves + global.
        let stream = match scope {
            MemoryScope::Global => {
                // Global scope matches everything — no filter needed.
                table.query().execute().await
                    .map_err(|e| CerebrumError::Database(e.to_string()))?
            }
            _ => {
                let predicate = format!(
                    "scope = 'global' OR scope = {}",
                    sql_quote(&scope.as_str())
                );
                table.query().only_if(predicate).execute().await
                    .map_err(|e| CerebrumError::Database(e.to_string()))?
            }
        };

        let batches: Vec<RecordBatch> = stream.try_collect().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;

        let mut scored: Vec<(LanceDBMemoryRecord, f32)> = batches.iter()
            .flat_map(|b| Self::batch_to_records(b).unwrap_or_default())
            .filter_map(|record| {
                // Precise scope match in Rust (handles bidirectional Global logic).
                let entry = record.to_entry().ok()?;
                if !scope.matches(&entry.scope) {
                    return None;
                }
                let sim   = Self::cosine_similarity(&query_embedding, &record.embedding);
                let score = sim * 0.7 + record.salience * 0.3;
                Some((record, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter()
            .take(limit)
            .map(|(r, _)| r.to_entry())
            .collect()
    }

    /// Delete a memory by ID.
    async fn delete(&self, id: &MemoryId) -> Result<()> {
        let predicate = format!("id = {}", sql_quote(&id.to_string()));
        self.table().await?
            .delete(&predicate)
            .await
            .map_err(|e| CerebrumError::Database(format!("delete failed: {}", e)))?;
        Ok(())
    }

    /// List all memories in the store.
    ///
    /// Explicit override — the trait default would embed the literal `"*"` string.
    async fn list(&self) -> Result<Vec<MemoryEntry>> {
        let table = self.table().await?;
        let row_count = table.count_rows(None).await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        if row_count == 0 {
            return Ok(vec![]);
        }

        let stream = table.query().execute().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;
        let batches: Vec<RecordBatch> = stream.try_collect().await
            .map_err(|e| CerebrumError::Database(e.to_string()))?;

        batches.iter()
            .flat_map(|b| Self::batch_to_records(b).unwrap_or_default())
            .map(|r| r.to_entry())
            .collect()
    }

    /// Get the number of memories in the store.
    ///
    /// Explicit override — the trait default would call list() which embeds `"*"`.
    async fn len(&self) -> Result<usize> {
        self.table().await?
            .count_rows(None)
            .await
            .map_err(|e| CerebrumError::Database(e.to_string()))
    }

    /// Check if the store is empty.
    ///
    /// Explicit override — the trait default would call len() via list() via retrieve("*"...).
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }
}
```

---

## Phase 4: Wire Config through orchestrator and main

Commit message: `feat(orchestrator): accept decomposed primitives; use Config in main`

### Step 1: Update MemoryOrchestrator::new signature

In `crates/cerebrum-core/src/orchestrator.rs`, change `pub async fn new` (line 26) from:
```rust
pub async fn new(cortex_db_path: &str, embedder: Arc<dyn Embedder>) -> Result<Self>
```
to:
```rust
pub async fn new(db_path: &Path, table_name: &str, dim: usize, embedder: Arc<dyn Embedder>) -> Result<Self>
```

Add `use std::path::Path;` to the imports. Update the body to call:
```rust
LanceDBCortex::new(db_path, table_name, dim, embedder.clone()).await?
```

Remove the `with_lancedb_cortex` constructor (lines 38-53) — it is identical to `new` and now redundant.

### Step 2: Update main.rs to use Config

In `crates/cerebrum/src/main.rs`, replace line 19:
```rust
let orchestrator = Arc::new(MemoryOrchestrator::new("/tmp/cerebrum_cortex", embedder).await?);
```
with:
```rust
let config = cerebrum_core::Config::default();
let orchestrator = Arc::new(
    MemoryOrchestrator::new(&config.db_path, &config.table_name, config.embedding_dim, embedder)
        .await?,
);
```

### Step 3: Migrate orchestrator.rs test call sites

In `crates/cerebrum-core/src/orchestrator.rs`, all test functions that call
`MemoryOrchestrator::new("/tmp/test_*", embedder)` (~20 occurrences) must be updated.

For each test function:
1. Add `let dir = tempfile::tempdir().unwrap();` before the orchestrator construction.
2. Replace the `new("/tmp/test_*", embedder)` call with `new(dir.path(), "memories", 384, embedder)`.
3. Keep the `dir` binding alive to the end of the test (LanceDB requires the directory to exist).

Add `use tempfile;` to the test module's use declarations if not already present.

### Step 4: Migrate mcp_server.rs test call sites

Same as Step 3, applied to `crates/cerebrum/src/mcp_server.rs`. Every
`MemoryOrchestrator::new("/tmp/test_*", embedder)` call (~15 occurrences) becomes
`MemoryOrchestrator::new(dir.path(), "memories", 384, embedder)` with a `tempdir()`.

Add `tempfile = "3"` to `[dev-dependencies]` in `crates/cerebrum/Cargo.toml`.

---

## Phase 5: Tests for real persistence

Commit message: `test(cortex): add persistence, idempotency, dimension-guard, and empty-table tests`

All new tests go in the `#[cfg(test)] mod tests` block at the bottom of
`crates/cerebrum-core/src/lancedb_cortex.rs`.

### Step 1: Migrate existing tests from ":memory:" to tempdir

Every existing test in the block that calls `LanceDBCortex::new(":memory:", embedder)` must be
updated to:
```rust
let dir = tempfile::tempdir().unwrap();
let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder.clone())
    .await
    .unwrap();
```
Keep `dir` alive for the test duration.

### Step 2: Add the reopen-persistence test

This is the test that proves the gap is closed — data survives dropping and re-opening the store:

```rust
#[tokio::test]
async fn test_cortex_persists_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let embedder = Arc::new(MockEmbedder::new());
    let id = MemoryId::new();

    {
        let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder.clone())
            .await
            .unwrap();
        let entry = MemoryEntry::builder(id, "persistent memory".to_string())
            .embedding(vec![0.1; 384])
            .tier(MemoryTier::Cortex)
            .build();
        cortex.store(entry).await.unwrap();
    }
    // LanceDBCortex dropped here — connection closed.

    {
        let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder.clone())
            .await
            .unwrap();
        let results = cortex.list().await.unwrap();
        assert_eq!(results.len(), 1, "memory must survive process restart");
        assert_eq!(results[0].id, id);
        assert_eq!(results[0].content, "persistent memory");
    }
}
```

### Step 3: Add dimension-mismatch test

```rust
#[tokio::test]
async fn test_cortex_dimension_mismatch_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let embedder = Arc::new(MockEmbedder::new()); // reports dimension 384
    // Request dim=768 — must be rejected before any LanceDB call.
    let result = LanceDBCortex::new(dir.path(), "memories", 768, embedder).await;
    assert!(result.is_err(), "mismatched dimension must error at construction");
}
```

### Step 4: Add empty-table guard test

```rust
#[tokio::test]
async fn test_cortex_search_empty_table_returns_empty_vec() {
    let dir = tempfile::tempdir().unwrap();
    let embedder = Arc::new(MockEmbedder::new());
    let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder)
        .await
        .unwrap();
    // All read operations on a fresh store must succeed with empty results.
    assert_eq!(cortex.retrieve("anything", 10).await.unwrap(), vec![]);
    assert_eq!(cortex.list().await.unwrap(), vec![]);
    assert_eq!(cortex.len().await.unwrap(), 0);
    assert!(cortex.is_empty().await.unwrap());
}
```

### Step 5: Add upsert-idempotency test

```rust
#[tokio::test]
async fn test_cortex_store_upserts_on_same_id() {
    let dir = tempfile::tempdir().unwrap();
    let embedder = Arc::new(MockEmbedder::new());
    let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder)
        .await
        .unwrap();
    let id = MemoryId::new();

    cortex.store(
        MemoryEntry::builder(id, "version 1".to_string())
            .embedding(vec![0.1; 384])
            .tier(MemoryTier::Cortex)
            .build(),
    ).await.unwrap();

    cortex.store(
        MemoryEntry::builder(id, "version 2".to_string())
            .embedding(vec![0.2; 384])
            .tier(MemoryTier::Cortex)
            .build(),
    ).await.unwrap();

    let entries = cortex.list().await.unwrap();
    assert_eq!(entries.len(), 1, "upsert must not duplicate rows");
    assert_eq!(entries[0].content, "version 2");
}
```

### Step 6: Add list/len must-not-use-retrieve test

This guards against someone removing the explicit overrides, causing the defaults to silently re-engage:

```rust
#[tokio::test]
async fn test_cortex_list_and_len_work_without_embedding_query() {
    // MockEmbedder rejects empty strings. If list() or len() internally called
    // retrieve("*", usize::MAX), this test would fail. It passing proves the
    // explicit overrides are in place and the defaults are not used.
    let dir = tempfile::tempdir().unwrap();
    let embedder = Arc::new(MockEmbedder::new());
    let cortex = LanceDBCortex::new(dir.path(), "memories", 384, embedder)
        .await
        .unwrap();
    // These must succeed without embedding anything.
    let _ = cortex.list().await.expect("list() must not call embed()");
    let _ = cortex.len().await.expect("len() must not call embed()");
    let _ = cortex.is_empty().await.expect("is_empty() must not call embed()");
}
```

---

## Phase 6: Cleanup

Commit message: `chore: remove parking_lot if unused; add tempfile to cerebrum dev-deps`

### Step 1: Check parking_lot usage across all modules

Grep for `parking_lot` across `crates/cerebrum-core/src/`. The `synapse.rs` module uses
`parking_lot::RwLock` for its in-memory `HashMap`. If it does:
- Keep `parking_lot = "0.12"` in `crates/cerebrum-core/Cargo.toml`.
- No change needed.

If nothing else uses it after Phase 3:
- Remove `parking_lot = "0.12"` from `[dependencies]` in `crates/cerebrum-core/Cargo.toml`.

### Step 2: Add tempfile to cerebrum binary dev-deps

In `crates/cerebrum/Cargo.toml`, add under `[dev-dependencies]`:
```toml
tempfile = "3"
```

This is required for the migrated tests in `mcp_server.rs` (Phase 4 Step 4).

### Step 3: Final verification

Run the full test suite and typecheck:
```
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All pre-existing tests must still pass (migrated, not deleted). The five new tests in Phase 5
must pass, including `test_cortex_persists_across_reopen` which is the definitive proof that
long-term memory now actually persists.
