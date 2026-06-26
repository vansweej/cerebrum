# Migration Guide

This guide covers migrating between different backends and embedders in Cerebrum.

## Embedder Migration: MockEmbedder to Ollama

### Overview

Cerebrum supports two embedders:
- **MockEmbedder:** Hash-based, deterministic, suitable for development
- **FastEmbedEmbedder:** Real Ollama integration, semantic embeddings, suitable for production

### Prerequisites

Before migrating to Ollama embeddings, ensure:
1. Ollama is installed and running: `ollama serve`
2. nomic-embed-text model is available: `ollama pull nomic-embed-text`
3. Ollama is accessible at http://localhost:11434 (or configure custom endpoint)

### Step 1: Update Your Code

**Before (MockEmbedder):**
```rust
use cerebrum_core::embedder::MockEmbedder;
use cerebrum_core::orchestrator::MemoryOrchestrator;
use std::sync::Arc;

let embedder = Arc::new(MockEmbedder::new());
let orchestrator = MemoryOrchestrator::new(":memory:", embedder).await?;
```

**After (FastEmbedEmbedder):**
```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use cerebrum_core::orchestrator::MemoryOrchestrator;
use std::sync::Arc;

let embedder = Arc::new(FastEmbedEmbedder::new());

// Verify Ollama is available
if !embedder.is_available().await {
    eprintln!("Ollama not available, falling back to MockEmbedder");
    let embedder = Arc::new(MockEmbedder::new());
}

let orchestrator = MemoryOrchestrator::new(":memory:", embedder).await?;
```

### Step 2: Migrate Existing Embeddings

Use the migration tooling to re-embed existing memories:

```rust
use cerebrum_core::migration::{MigrationConfig, MigrationManager, MigrationStrategy};
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use cerebrum_core::embedder::MockEmbedder;
use std::sync::Arc;

// Create old orchestrator with MockEmbedder
let old_embedder = Arc::new(MockEmbedder::new());
let old_orchestrator = MemoryOrchestrator::new(":memory:", old_embedder).await?;

// Create new orchestrator with FastEmbedEmbedder
let new_embedder = Arc::new(FastEmbedEmbedder::new());
let new_orchestrator = MemoryOrchestrator::new(":memory:", new_embedder).await?;

// Use Hybrid strategy: re-embed high-salience memories, preserve low-salience
let config = MigrationConfig::new(MigrationStrategy::Hybrid, new_embedder.clone())
    .with_batch_size(50)
    .with_hybrid_threshold(0.7);  // Re-embed memories with salience >= 0.7

let manager = MigrationManager::new();
let result = manager.execute(&old_orchestrator.cortex(), &config).await?;

println!("Migration success rate: {:.2}%", result.success_rate());
println!("Memories migrated: {}", result.migrated_count());
```

### Step 3: Verify Migration

```rust
// Verify embeddings are now semantic
let test_query = "important information";
let results = new_orchestrator.recall(test_query.to_string(), 10).await?;

println!("Found {} semantically similar memories", results.len());

// Check metrics
let metrics = new_embedder.metrics();
println!("Embedding success rate: {:.1}%", metrics.success_rate());
println!("Average embedding latency: {:.2}ms", metrics.average_time_ms());
```

### Step 4: Monitor Circuit Breaker

```rust
// Monitor circuit breaker during migration
let cb = new_embedder.circuit_breaker();

match cb.allow_request() {
    Ok(()) => println!("Circuit breaker: CLOSED (healthy)"),
    Err(_) => {
        eprintln!("Circuit breaker: OPEN (Ollama may be unavailable)");
        eprintln!("Check Ollama status: curl http://localhost:11434/api/tags");
    }
}
```

## Backend Migration: In-Memory to LanceDB

### Overview

Cerebrum supports two Cortex backends:
- **In-Memory (CortexMemory):** Fast, suitable for development and testing
- **LanceDB:** Persistent, suitable for production deployments

### Step 1: Update Your Code

**Before (In-Memory):**
```rust
use cerebrum_core::orchestrator::MemoryOrchestrator;
use cerebrum_core::embedder::MockEmbedder;
use std::sync::Arc;

let embedder = Arc::new(MockEmbedder::new());
let orchestrator = MemoryOrchestrator::new("/tmp/cortex", embedder).await?;
```

**After (LanceDB):**
```rust
use cerebrum_core::orchestrator::MemoryOrchestrator;
use cerebrum_core::embedder::MockEmbedder;
use std::sync::Arc;

let embedder = Arc::new(MockEmbedder::new());
let orchestrator = MemoryOrchestrator::new("/tmp/lancedb", embedder).await?;
```

### Step 2: Migrate Existing Data

If you have existing memories in the in-memory backend, use the migration tooling:

```rust
use cerebrum_core::migration::{MigrationConfig, MigrationManager, MigrationStrategy};

// Create source and destination orchestrators
let source = MemoryOrchestrator::new("/tmp/cortex_old", embedder.clone()).await?;
let dest = MemoryOrchestrator::new("/tmp/lancedb_new", embedder.clone()).await?;

// Retrieve all memories from source
let all_memories = source.cortex().list().await?;

// Store in destination
for memory in all_memories {
    dest.cortex().store(memory).await?;
}
```

### Step 3: Verify Migration

```rust
// Verify data integrity
let source_count = source.cortex().len().await?;
let dest_count = dest.cortex().len().await?;

assert_eq!(source_count, dest_count, "Memory counts should match");

// Test recall functionality
let results = dest.recall("test query".to_string(), 10).await?;
println!("Found {} memories", results.len());
```

## Embedding Model Migration

### Overview

Cerebrum supports three migration strategies when changing embedding models:

1. **Reembed:** Re-embed all memories with the new model
   - Most accurate, preserves semantic relationships
   - Slowest, requires re-processing all memories
   - Best for: Critical applications where accuracy is paramount

2. **Preserve:** Keep old embeddings, add new ones alongside
   - Preserves history and allows rollback
   - Uses more storage (2x embeddings)
   - Best for: Gradual rollouts, A/B testing

3. **Hybrid:** Re-embed high-salience memories, preserve low-salience ones
   - Balanced approach between accuracy and performance
   - Configurable threshold for salience
   - Best for: Most production scenarios

### Step 1: Choose Migration Strategy

```rust
use cerebrum_core::migration::{MigrationConfig, MigrationStrategy};

// Reembed all memories
let config = MigrationConfig::new(MigrationStrategy::Reembed, new_embedder.clone());

// Or preserve old embeddings
let config = MigrationConfig::new(MigrationStrategy::Preserve, new_embedder.clone());

// Or hybrid with 0.7 salience threshold
let config = MigrationConfig::new(MigrationStrategy::Hybrid, new_embedder.clone())
    .with_hybrid_threshold(0.7);
```

### Step 2: Configure Migration Parameters

```rust
let config = MigrationConfig::new(MigrationStrategy::Hybrid, new_embedder)
    .with_batch_size(100)           // Process 100 memories at a time
    .with_hybrid_threshold(0.5)     // Re-embed memories with salience > 0.5
    .with_dry_run(true);            // Test without making changes
```

### Step 3: Execute Migration

```rust
use cerebrum_core::migration::MigrationManager;

let manager = MigrationManager::new();
let result = manager.execute(&cortex, &config).await?;

println!("Migration completed:");
println!("  Total memories: {}", result.total_memories);
println!("  Successful: {}", result.successful_operations);
println!("  Failed: {}", result.failed_operations);
println!("  Success rate: {:.2}%", result.success_rate());
```

### Step 4: Verify Results

```rust
// Test recall with new embeddings
let query = "test query";
let results = orchestrator.recall(query.to_string(), 10).await?;

// Verify results are semantically similar
for result in results {
    println!("Found: {} (salience: {:.2})", result.content, result.salience);
}
```

## Best Practices

### 1. Test Migrations in Staging First

Always test migrations in a staging environment before production:

```rust
// Staging: Test with dry-run
let config = MigrationConfig::new(strategy, new_embedder.clone())
    .with_dry_run(true);

let result = manager.execute(&staging_cortex, &config).await?;
println!("Dry-run results: {}", result.success_rate());

// Production: Execute for real
let config = MigrationConfig::new(strategy, new_embedder.clone())
    .with_dry_run(false);

let result = manager.execute(&prod_cortex, &config).await?;
```

### 2. Use Batch Processing for Large Datasets

```rust
// For large datasets, use smaller batches to avoid memory issues
let config = MigrationConfig::new(strategy, new_embedder)
    .with_batch_size(50);  // Process 50 at a time instead of default 100
```

### 3. Monitor Migration Progress

```rust
// Use observability context to track migration
let context = ObservabilityContext::new();

// Migrations automatically record metrics
context.log_summary();
```

### 4. Preserve Old Embeddings During Rollout

```rust
// Use Preserve strategy during gradual rollout
let config = MigrationConfig::new(MigrationStrategy::Preserve, new_embedder)
    .with_batch_size(100);

// If new embedder has issues, you can still use old embeddings
```

### 5. Use Hybrid Strategy for Production

```rust
// Hybrid strategy balances accuracy and performance
let config = MigrationConfig::new(MigrationStrategy::Hybrid, new_embedder)
    .with_hybrid_threshold(0.6)  // Re-embed important memories
    .with_batch_size(100);
```

## Troubleshooting

### Migration Fails with "Unavailable" Error

This indicates a transient failure. The system will automatically retry with exponential backoff:

```rust
use cerebrum_core::resilience::RetryConfig;
use std::time::Duration;

let retry_config = RetryConfig::new()
    .with_max_retries(5)
    .with_initial_backoff_ms(100);

// Retry logic is built into migration manager
```

### Low Success Rate

If success rate is below 90%, investigate:

1. Check embedder availability
2. Verify database connectivity
3. Review error logs with observability context
4. Consider using smaller batch sizes

```rust
let context = ObservabilityContext::new();
context.log_summary();  // Shows detailed metrics
```

### Memory Leaks During Migration

Use batch processing to avoid loading all memories at once:

```rust
let config = MigrationConfig::new(strategy, new_embedder)
    .with_batch_size(50);  // Smaller batches
```

## Rollback Procedures

### If Using Preserve Strategy

```rust
// Old embeddings are preserved, so you can revert to old embedder
let old_embedder = Arc::new(OldEmbedder::new());
let orchestrator = MemoryOrchestrator::new("/tmp/cortex", old_embedder).await?;

// Recall will use old embeddings
let results = orchestrator.recall("query".to_string(), 10).await?;
```

### If Using Reembed Strategy

```rust
// Create backup before migration
let backup = cortex.list().await?;

// If migration fails, restore from backup
for memory in backup {
    cortex.store(memory).await?;
}
```

## Performance Considerations

| Strategy | Speed | Accuracy | Storage | Best For |
|----------|-------|----------|---------|----------|
| Reembed | Slow | Highest | Normal | Critical applications |
| Preserve | Fast | Medium | 2x | Gradual rollouts |
| Hybrid | Medium | High | Normal | Most production |

## See Also

- [Observability Guide](OBSERVABILITY_GUIDE.md) - Monitor migrations
- [Architecture](architecture.md) - System design
