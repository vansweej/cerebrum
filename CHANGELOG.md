# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-06-25

### Phase 6: Production Hardening & LanceDB Integration

#### Added

**LanceDB Cortex Backend**
- New `LanceDBCortex` implementation for persistent vector database storage
- `MemoryOrchestrator::with_lancedb_cortex()` builder method for LanceDB backend
- Support for configurable Cortex backends (in-memory or LanceDB)
- Efficient semantic search with vector embeddings
- Scalable storage for large memory collections

**FastEmbed Integration**
- `FastEmbedEmbedder` for hash-based embedding generation
- Consistent, reproducible embeddings across calls
- Support for custom embedder implementations
- Configurable embedding dimensions

**Embedding Migration Tooling**
- `MigrationConfig` for configuring migration parameters
- `MigrationManager` for orchestrating migrations
- Three migration strategies:
  - `Reembed`: Re-embed all memories with new model
  - `Preserve`: Keep old embeddings, add new ones alongside
  - `Hybrid`: Re-embed high-salience memories, preserve low-salience ones
- Batch processing for efficient migrations
- Dry-run mode for testing migrations without making changes
- `MigrationResult` for tracking migration outcomes

**Observability & Structured Logging**
- `ObservabilityContext` for comprehensive metrics collection
- `OperationMetrics` for tracking operation success rates and timing
- `OperationTimer` for measuring operation duration
- Integration with `tracing` crate for structured logging
- OpenTelemetry compatible instrumentation
- Per-operation metrics tracking (remember, recall, memorize, forget, promote, decay)

**Error Handling & Resilience**
- `CircuitBreaker` pattern implementation with three states (Closed, Open, HalfOpen)
- `CircuitBreakerConfig` for configurable failure thresholds and timeouts
- `RetryConfig` with exponential backoff and jitter
- `CerebrumError::Unavailable` variant for transient failures
- Graceful degradation under failure conditions

**Orchestrator Enhancements**
- `MemoryOrchestrator::embedder()` accessor method
- `MemoryOrchestrator::synapse()` accessor method
- `MemoryOrchestrator::cortex()` accessor method
- Support for trait-based Cortex backends

**Testing**
- 10 new orchestrator unit tests for LanceDB backend
- 20 comprehensive integration tests covering:
  - LanceDB persistence and data storage
  - Embedder consistency
  - Migration workflows with all strategies
  - Error recovery with circuit breaker
  - Retry with exponential backoff
  - Observability context functionality
  - End-to-end workflows
  - Concurrent operations
  - Memory decay
  - Blended search across tiers
  - Metadata preservation
  - Circuit breaker state transitions
  - And more...

#### Changed

- `MemoryStore` trait now includes `list()` method with default implementation
- Orchestrator now supports configurable backends via trait objects
- Enhanced error handling with more specific error types

#### Documentation

- Updated README.md with Phase 6 features and usage examples
- Added Migration Guide (docs/MIGRATION_GUIDE.md)
- Added Observability Guide (docs/OBSERVABILITY_GUIDE.md)

#### Metrics

- Total tests: 309 (up from 188)
- Code coverage: 87.47%
- Zero clippy warnings
- All tests passing

### Previous Phases

#### Phase 1-5: Core Memory System

- Two-tier memory architecture (Synapse + Cortex)
- Semantic search with embeddings
- Automatic memory promotion
- Memory decay and salience tracking
- Multiple promotion strategies
- Comprehensive test coverage

## [0.1.0] - Initial Release

### Added

- Initial two-tier memory system
- Synapse tier for short-term memory
- Cortex tier for long-term memory
- Basic memory operations (remember, recall, memorize, forget)
- Semantic search functionality
- Memory promotion logic
- Memory decay mechanisms
