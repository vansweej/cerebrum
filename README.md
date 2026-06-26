# Cerebrum

A two-tier agent memory subsystem implemented as a single Model Context Protocol (MCP) server.

## Features

### Core Memory System
- **Synapse Tier:** Fast, in-memory short-term memory with semantic search
- **Cortex Tier:** Persistent long-term memory with vector embeddings
- **Blended Recall:** Unified search across both memory tiers
- **Automatic Promotion:** Memories promoted from Synapse to Cortex based on salience

### Phase 6: Production Hardening & LanceDB Integration

#### Real Ollama Integration
- **Semantic Embeddings:** Real embeddings via Ollama HTTP API (nomic-embed-text model)
- **384-Dimensional Vectors:** Optimized for semantic similarity search
- **Automatic Fallback:** Graceful degradation when Ollama is unavailable
- **Configurable Endpoint:** Support for custom Ollama server locations

#### LanceDB Cortex Backend
- Persistent vector database for long-term memory storage
- Configurable backend support (in-memory Synapse or LanceDB Cortex)
- Efficient semantic search with vector embeddings
- Scalable storage for large memory collections

#### Observability & Resilience
- **Circuit Breaker Pattern:** Automatic failure detection and recovery
- **Operation Metrics:** Latency tracking, success rate monitoring, error counting
- **Structured Logging:** Comprehensive tracing with `tracing` crate
- **Graceful Degradation:** System continues operating when Ollama is unavailable

#### Embedding Migration Tooling
- **Reembed Strategy:** Re-embed all memories with new model (most accurate)
- **Preserve Strategy:** Keep old embeddings, add new ones alongside (preserves history)
- **Hybrid Strategy:** Re-embed high-salience memories, preserve low-salience ones (balanced)
- Batch processing for efficient migrations
- Dry-run mode for testing migrations

#### Error Handling & Resilience
- **Circuit Breaker Pattern:** Automatic failure detection and recovery
- **Exponential Backoff Retry:** Configurable retry logic with jitter
- **Comprehensive Error Types:** Detailed error information for debugging
- Graceful degradation under failure conditions

## Quick Start

### Prerequisites

1. **Ollama Server** (for real semantic embeddings)
   ```bash
   # Install Ollama from https://ollama.ai
   # Start the Ollama server
   ollama serve
   
   # In another terminal, pull the nomic-embed-text model
   ollama pull nomic-embed-text
   ```

2. **Rust & Nix** (for development)
   ```bash
   # Install Nix from https://nixos.org/download.html
   ```

### Running Cerebrum

```bash
nix develop . --command cargo run --bin cerebrum
```

## Usage Examples

### Basic Memory Operations

```rust
use cerebrum_core::orchestrator::MemoryOrchestrator;
use cerebrum_core::embedder::MockEmbedder;
use cerebrum_core::models::MemoryScope;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let embedder = Arc::new(MockEmbedder::new());
    let orchestrator = MemoryOrchestrator::new(":memory:", embedder).await?;

    // Store a memory
    let content = "Important information about the user".to_string();
    orchestrator.memorize(&content, MemoryScope::Global).await?;

    // Recall memories
    let results = orchestrator.recall("information".to_string(), 10).await?;
    println!("Found {} memories", results.len());

    Ok(())
}
```

### Using Real Ollama Embeddings

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use cerebrum_core::orchestrator::MemoryOrchestrator;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create embedder with real Ollama backend
    let embedder = Arc::new(FastEmbedEmbedder::new());
    
    // Verify Ollama is available
    if !embedder.is_available().await {
        eprintln!("Warning: Ollama server not available at http://localhost:11434");
        eprintln!("Please start Ollama: ollama serve");
        return Err("Ollama not available".into());
    }

    let orchestrator = MemoryOrchestrator::new("/tmp/cortex", embedder).await?;

    // Use orchestrator with real semantic embeddings
    let results = orchestrator.recall("query".to_string(), 10).await?;
    
    Ok(())
}
```

### Monitoring Metrics

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;

let embedder = FastEmbedEmbedder::new();
let metrics = embedder.metrics();

// Check operation metrics
println!("Total operations: {}", metrics.total_operations());
println!("Success rate: {:.1}%", metrics.success_rate());
println!("Average latency: {:.2}ms", metrics.average_time_ms());
```

### Circuit Breaker Status

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;

let embedder = FastEmbedEmbedder::new();
let cb = embedder.circuit_breaker();

// Check if requests are allowed
match cb.allow_request() {
    Ok(()) => println!("Circuit breaker is CLOSED - requests allowed"),
    Err(_) => println!("Circuit breaker is OPEN - requests denied"),
}
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

## Troubleshooting

### Ollama Connection Issues

**Problem:** "Cannot connect to Ollama at http://localhost:11434"

**Solution:**
```bash
# 1. Verify Ollama is running
curl http://localhost:11434/api/tags

# 2. If not running, start it
ollama serve

# 3. Verify nomic-embed-text model is available
ollama list

# 4. If not available, pull it
ollama pull nomic-embed-text
```

### Circuit Breaker Open

**Problem:** Circuit breaker is open and rejecting requests

**Explanation:** The circuit breaker opens after 5 consecutive failures to protect the system from cascading failures.

**Solution:**
- Check Ollama server status
- Wait 60 seconds for the circuit breaker to transition to HalfOpen state
- Once Ollama recovers, the circuit breaker will automatically close

### Slow Embeddings

**Problem:** Embedding operations are slow

**Explanation:** First embedding request may be slow as Ollama loads the model into memory.

**Solution:**
- Subsequent requests will be faster (model stays in memory)
- Monitor metrics to track average latency
- Consider increasing Ollama's memory allocation if available

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

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run Phase 4 coverage tests
cargo test --test phase4_coverage_tests

# Run with output
cargo test -- --nocapture
```

## Code Quality Requirements

- **Coverage Gate:** All code must maintain ≥90% test coverage (configured in `tarpaulin.toml`, enforced by `cargo tarpaulin`)
- **Formatting:** Code must be formatted with `cargo fmt`
- **Linting:** All clippy warnings must be fixed (run `cargo clippy -- -D warnings`)
- **Tests:** All 282+ tests must pass before committing

## Documentation

- [Architecture](docs/architecture.md) - System design and memory tier documentation
- [Migration Guide](docs/MIGRATION_GUIDE.md) - How to migrate between backends and embedders
- [Observability Guide](docs/OBSERVABILITY_GUIDE.md) - Metrics and logging setup
- [Ollama Integration Guide](docs/OLLAMA_INTEGRATION.md) - Real semantic embeddings setup

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design and memory tier documentation.

## License

MIT
