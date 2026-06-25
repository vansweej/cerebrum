# ADR-001: LanceDB for Persistent Cortex Storage

## Status
Proposed

## Context

Cerebrum currently uses in-memory HashMap storage for the Cortex (long-term memory) tier. This approach:

**Limitations:**
- No persistence across server restarts
- Memory usage grows unbounded
- No efficient vector search at scale
- Cannot support production deployments

**Requirements:**
- Persistent storage for long-term memories
- Efficient semantic search using embeddings
- Scalability to millions of memories
- Production-ready reliability
- Minimal operational overhead

**Constraints:**
- Single-server deployment (no distributed database)
- Embedded database preferred (no external services)
- Rust ecosystem (async/await support)
- 384-dimensional embeddings (BGE-small model)

## Decision

Use **LanceDB** as the persistent vector database backend for Cortex.

### Why LanceDB?

**Pros:**
- ✅ Embedded vector database (no external service)
- ✅ Rust SDK with async support
- ✅ Optimized for semantic search with embeddings
- ✅ ACID transactions and durability
- ✅ Minimal operational overhead
- ✅ Active development and community support
- ✅ Compatible with 384-dim embeddings

**Cons:**
- ⚠️ Newer project (less battle-tested than Postgres+pgvector)
- ⚠️ Single-node only (no built-in replication)
- ⚠️ Smaller ecosystem than traditional databases

### Alternatives Considered

1. **PostgreSQL + pgvector**
   - Pros: Battle-tested, rich ecosystem, replication support
   - Cons: Requires external service, operational overhead, overkill for single-server
   - Decision: Rejected (too heavy for embedded use case)

2. **Qdrant**
   - Pros: Specialized vector database, excellent performance
   - Cons: Requires external service, operational overhead
   - Decision: Rejected (not embedded)

3. **Milvus**
   - Pros: Scalable, distributed
   - Cons: Requires external service, complex deployment
   - Decision: Rejected (not embedded)

4. **In-memory HashMap (current)**
   - Pros: Simple, no dependencies
   - Cons: No persistence, unbounded memory growth, poor scalability
   - Decision: Rejected (not production-ready)

## Rationale

LanceDB provides the best balance of:
- **Simplicity:** Embedded database, no external services
- **Performance:** Optimized for vector search
- **Reliability:** ACID transactions, durability
- **Maintainability:** Rust SDK, async/await support
- **Scalability:** Efficient indexing for millions of vectors

The trade-off of single-node deployment is acceptable for Phase 6, with distributed deployment deferred to Phase 7+.

## Consequences

### Positive
- ✅ Persistent long-term memory across server restarts
- ✅ Efficient semantic search at scale
- ✅ Production-ready reliability
- ✅ Minimal operational overhead
- ✅ Foundation for future scaling

### Negative
- ⚠️ New dependency (LanceDB)
- ⚠️ Increased binary size
- ⚠️ Single-node limitation (no built-in replication)
- ⚠️ Requires disk space for vector index

### Mitigation
- Pin LanceDB version and monitor for breaking changes
- Implement comprehensive integration tests
- Add migration tooling for schema changes
- Document backup and recovery procedures
- Plan for distributed deployment in Phase 7+

## Implementation

### Phase 6 Step 1: LanceDB Integration Foundation
- Add `lancedb` crate to dependencies
- Create `LanceDBCortex` implementation of `MemoryStore` trait
- Implement schema: `{id, content, embedding, salience, scope, timestamp, metadata}`
- Add connection pooling for concurrent access
- Maintain backward compatibility with in-memory `CortexMemory`

### Phase 6 Step 2: FastEmbed Integration
- Replace `MockEmbedder` with `FastEmbedEmbedder`
- Use BGE-small model (384-dim)
- Implement embedding caching

### Phase 6 Step 3: Embedding Migration
- Implement `EmbeddingMigration` trait
- Support re-embedding with new models
- Add CLI tool for running migrations

## Related Decisions

- **ADR-002:** FastEmbed for Real Embeddings
- **ADR-003:** Embedding Migration Strategy
- **ADR-004:** Error Handling and Resilience

## References

- [LanceDB Documentation](https://lancedb.com/)
- [Vector Database Comparison](https://github.com/erikbern/ann-benchmarks)
- [Cerebrum Architecture](../architecture.md)

---

# ADR-002: FastEmbed for Real Embeddings

## Status
Proposed

## Context

Cerebrum currently uses `MockEmbedder` for deterministic testing. For production:

**Requirements:**
- Real semantic embeddings for accurate similarity search
- Consistent embedding dimensions (384-dim)
- Fast inference (sub-100ms per embedding)
- Offline capability (no external API calls)
- Deterministic results (same input → same embedding)

**Constraints:**
- Rust ecosystem
- Embedded model (no external service)
- 384-dimensional output (BGE-small model)

## Decision

Use **FastEmbed** with **BGE-small** model (384-dim) for real embeddings.

### Why FastEmbed + BGE-small?

**Pros:**
- ✅ Fast inference (optimized Rust implementation)
- ✅ Offline capability (model weights bundled)
- ✅ 384-dim output (matches current architecture)
- ✅ High-quality embeddings (BGE-small is state-of-the-art for small models)
- ✅ Active development and community support
- ✅ Deterministic results

**Cons:**
- ⚠️ Model download on first run (~100MB)
- ⚠️ CPU-bound inference (no GPU support in Phase 6)
- ⚠️ Requires disk space for model weights

### Alternatives Considered

1. **OpenAI Embeddings API**
   - Pros: High quality, no local compute
   - Cons: Requires API key, external dependency, cost, latency
   - Decision: Rejected (not offline-capable)

2. **Sentence Transformers (Python)**
   - Pros: High quality, flexible
   - Cons: Requires Python runtime, not Rust-native
   - Decision: Rejected (adds complexity)

3. **ONNX Runtime**
   - Pros: Flexible, supports multiple models
   - Cons: More complex setup, larger binary
   - Decision: Rejected (FastEmbed is simpler)

4. **MockEmbedder (current)**
   - Pros: Deterministic, no dependencies
   - Cons: Not real embeddings, poor similarity quality
   - Decision: Rejected (not production-ready)

## Rationale

FastEmbed + BGE-small provides:
- **Quality:** State-of-the-art embeddings for semantic search
- **Performance:** Fast inference, suitable for real-time recall
- **Reliability:** Offline capability, deterministic results
- **Simplicity:** Rust-native, minimal setup
- **Consistency:** 384-dim output matches current architecture

## Consequences

### Positive
- ✅ Real semantic embeddings for accurate search
- ✅ Offline capability (no external dependencies)
- ✅ Fast inference suitable for production
- ✅ Deterministic results for reproducibility

### Negative
- ⚠️ Model download on first run (~100MB)
- ⚠️ Disk space for model weights (~100MB)
- ⚠️ CPU-bound inference (no GPU acceleration in Phase 6)
- ⚠️ Increased binary size

### Mitigation
- Vendor model weights in Nix flake for offline builds
- Implement embedding caching to reduce inference calls
- Add GPU support in Phase 7+ (CUDA/Metal)
- Document model download and caching behavior

## Implementation

### Phase 6 Step 2: FastEmbed Integration
- Add `fastembed` crate to dependencies
- Create `FastEmbedEmbedder` implementation of `Embedder` trait
- Initialize BGE-small model on startup
- Implement embedding caching with LRU eviction
- Keep `MockEmbedder` for testing (feature-gated)

## Related Decisions

- **ADR-001:** LanceDB for Persistent Cortex Storage
- **ADR-003:** Embedding Migration Strategy

## References

- [FastEmbed Documentation](https://github.com/qdrant/fastembed)
- [BGE Model Card](https://huggingface.co/BAAI/bge-small-en-v1.5)
- [Embedding Quality Benchmarks](https://huggingface.co/spaces/mteb/leaderboard)

---

# ADR-003: Embedding Migration Strategy

## Status
Proposed

## Context

Embedding models may change over time due to:
- Better models becoming available
- Dimension changes (e.g., 384 → 768)
- Model deprecation or licensing changes
- Performance optimization

**Requirements:**
- Support changing embedding models without data loss
- Preserve memory content and metadata
- Provide rollback capability
- Minimize downtime during migration
- Audit trail of model changes

**Constraints:**
- Single-server deployment
- Backward compatibility with existing memories
- No external dependencies for migration

## Decision

Implement **pluggable migration strategies** with support for:
1. **ReembedMigration:** Re-embed all memories with new model
2. **PreserveMigration:** Keep old embeddings, add new ones
3. **HybridMigration:** Gradual migration (new memories use new model, old memories migrated on access)

## Rationale

Multiple strategies support different use cases:
- **ReembedMigration:** Clean slate, best for major model changes
- **PreserveMigration:** Backward compatibility, supports A/B testing
- **HybridMigration:** Gradual transition, minimal disruption

## Consequences

### Positive
- ✅ Flexible migration strategies
- ✅ No data loss during model changes
- ✅ Rollback capability
- ✅ Audit trail of model versions

### Negative
- ⚠️ Increased complexity
- ⚠️ Requires disk space for multiple embeddings (PreserveMigration)
- ⚠️ Potential performance impact during migration

### Mitigation
- Implement dry-run mode to preview changes
- Add comprehensive testing for migration scenarios
- Document migration procedures
- Provide CLI tool for running migrations

## Implementation

### Phase 6 Step 3: Embedding Migration
- Create `EmbeddingMigration` trait with strategy pattern
- Implement `ReembedMigration`, `PreserveMigration`, `HybridMigration`
- Add `cerebrum-migrate` CLI tool
- Implement dry-run mode
- Add rollback capability

## Related Decisions

- **ADR-002:** FastEmbed for Real Embeddings
- **ADR-001:** LanceDB for Persistent Cortex Storage

## References

- [Cerebrum Architecture](../architecture.md)
- [Phase 6 Plan](../phase-6-plan.md)

---

# ADR-004: Error Handling and Resilience

## Status
Proposed

## Context

Production deployments require robust error handling:

**Requirements:**
- Handle transient failures (network, temporary unavailability)
- Graceful degradation when components fail
- Clear error messages for debugging
- Observability for monitoring
- Recovery without data loss

**Constraints:**
- Single-server deployment
- No external error tracking service
- Minimal performance overhead

## Decision

Implement **layered error handling** with:
1. **Retry logic:** Exponential backoff with jitter for transient failures
2. **Circuit breaker:** Fail fast after N consecutive failures, retry after timeout
3. **Graceful degradation:** Fall back to Synapse if Cortex unavailable
4. **Structured logging:** Detailed error context for debugging

## Rationale

Layered approach provides:
- **Resilience:** Automatic recovery from transient failures
- **Performance:** Circuit breaker prevents cascading failures
- **Observability:** Structured logging for debugging
- **User experience:** Graceful degradation instead of hard failures

## Consequences

### Positive
- ✅ Automatic recovery from transient failures
- ✅ Prevents cascading failures
- ✅ Clear error messages for debugging
- ✅ Graceful degradation

### Negative
- ⚠️ Increased complexity
- ⚠️ Potential latency from retries
- ⚠️ Requires careful tuning of retry parameters

### Mitigation
- Implement comprehensive testing for error scenarios
- Document retry and circuit breaker parameters
- Add observability for monitoring retry behavior
- Provide configuration options for tuning

## Implementation

### Phase 6 Step 5: Error Handling & Resilience
- Expand error types with detailed context
- Implement retry logic with exponential backoff
- Implement circuit breaker pattern
- Add graceful degradation for Cortex failures
- Add structured logging for all error paths

## Related Decisions

- **ADR-001:** LanceDB for Persistent Cortex Storage
- **ADR-002:** FastEmbed for Real Embeddings

## References

- [Release It! Design and Deploy Production-Ready Software](https://pragprog.com/titles/mnee2/release-it-second-edition/)
- [Cerebrum Architecture](../architecture.md)
