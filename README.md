# Cerebrum

A two-tier agent memory subsystem implemented as a single Model Context Protocol (MCP) server.

## Features

### Core Memory System
- **Synapse Tier:** Fast, in-memory short-term memory with semantic search
- **Cortex Tier:** Persistent long-term memory with vector embeddings
- **Blended Recall:** Unified search across both memory tiers
- **Automatic Promotion:** Memories promoted from Synapse to Cortex based on salience

### Phase 6: Production Hardening & LanceDB Integration

#### LanceDB Cortex Backend
- Persistent vector database for long-term memory storage
- Configurable backend support (in-memory Synapse or LanceDB Cortex)
- Efficient semantic search with vector embeddings
- Scalable storage for large memory collections

#### FastEmbed Integration
- Hash-based embedding generation for consistent, reproducible embeddings
- Support for custom embedder implementations
- Configurable embedding dimensions

#### Embedding Migration Tooling
- **Reembed Strategy:** Re-embed all memories with new model (most accurate)
- **Preserve Strategy:** Keep old embeddings, add new ones alongside (preserves history)
- **Hybrid Strategy:** Re-embed high-salience memories, preserve low-salience ones (balanced)
- Batch processing for efficient migrations
- Dry-run mode for testing migrations

#### Observability & Structured Logging
- Comprehensive metrics collection for all memory operations
- Operation timing and success rate tracking
- Structured logging with `tracing` crate
- OpenTelemetry compatible instrumentation
- Per-operation metrics (remember, recall, memorize, forget, promote, decay)

#### Error Handling & Resilience
- **Circuit Breaker Pattern:** Automatic failure detection and recovery
- **Exponential Backoff Retry:** Configurable retry logic with jitter
- **Comprehensive Error Types:** Detailed error information for debugging
- Graceful degradation under failure conditions

## Quick Start

```bash
nix develop . --command cargo run --bin cerebrum
```

## Usage Examples

### Basic Memory Operations

```rust
use cerebrum_core::orchestrator::MemoryOrchestrator;
use cerebrum_core::embedder::MockEmbedder;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let embedder = Arc::new(MockEmbedder::new());
    let orchestrator = MemoryOrchestrator::new("/tmp/cortex", embedder).await?;

    // Store a memory
    let id = orchestrator.remember(
        "Important information".to_string(),
        HashMap::new()
    ).await?;

    // Recall memories
    let results = orchestrator.recall("information".to_string(), 10).await?;

    // Promote to long-term storage
    orchestrator.memorize(id).await?;

    Ok(())
}
```

### Using LanceDB Backend

```rust
let orchestrator = MemoryOrchestrator::with_lancedb_cortex(
    "/tmp/lancedb",
    embedder
).await?;
```

### Migration Workflows

```rust
use cerebrum_core::migration::{MigrationConfig, MigrationManager, MigrationStrategy};

let config = MigrationConfig::new(MigrationStrategy::Hybrid, embedder)
    .with_batch_size(100)
    .with_hybrid_threshold(0.5);

let manager = MigrationManager::new();
let result = manager.execute(&cortex, &config).await?;

println!("Migration success rate: {:.2}%", result.success_rate());
```

### Observability

```rust
use cerebrum_core::observability::ObservabilityContext;

let context = ObservabilityContext::new();

// Metrics are automatically collected during operations
context.log_summary();
```

### Error Recovery with Circuit Breaker

```rust
use cerebrum_core::resilience::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

let config = CircuitBreakerConfig::new()
    .with_failure_threshold(5)
    .with_timeout_ms(60000);

let breaker = CircuitBreaker::new(config);

// Circuit breaker automatically handles transient failures
if breaker.allow_request().is_ok() {
    // Perform operation
}
```

## Development

All commands should be run inside the Nix dev shell:

```bash
nix develop . --command cargo fmt
nix develop . --command cargo clippy -- -D warnings
nix develop . --command cargo test
nix develop . --command cargo tarpaulin
```

Or enter the dev shell once and run commands directly:

```bash
nix develop
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo tarpaulin
```

## Code Quality Requirements

- **Coverage Gate:** All code must maintain ≥90% test coverage (configured in `tarpaulin.toml`, enforced by `cargo tarpaulin`)
- **Formatting:** Code must be formatted with `cargo fmt`
- **Linting:** All clippy warnings must be fixed (run `cargo clippy -- -D warnings`)

## Documentation

- [Architecture](docs/architecture.md) - System design and memory tier documentation
- [Migration Guide](docs/MIGRATION_GUIDE.md) - How to migrate between backends and embedders
- [Observability Guide](docs/OBSERVABILITY_GUIDE.md) - Metrics and logging setup

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design and memory tier documentation.

## License

MIT
