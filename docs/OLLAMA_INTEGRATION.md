# Ollama Integration Guide

This guide explains how to set up and use real semantic embeddings with Cerebrum using Ollama.

## Overview

Cerebrum integrates with Ollama to provide real semantic embeddings using the **nomic-embed-text** model. This enables accurate semantic similarity search for memory retrieval.

### Key Features

- **Real Semantic Embeddings:** Uses nomic-embed-text model (384-dimensional)
- **HTTP API Integration:** Communicates with Ollama via REST API
- **Automatic Fallback:** Gracefully handles Ollama unavailability
- **Circuit Breaker Protection:** Prevents cascading failures
- **Metrics Tracking:** Monitors embedding latency and success rates

## Setup

### 1. Install Ollama

Download and install Ollama from [https://ollama.ai](https://ollama.ai)

### 2. Start Ollama Server

```bash
ollama serve
```

This starts the Ollama server on `http://localhost:11434` (default).

### 3. Pull the nomic-embed-text Model

In another terminal:

```bash
ollama pull nomic-embed-text
```

This downloads the 274MB nomic-embed-text model (~5 minutes on typical internet).

### 4. Verify Setup

```bash
# Check Ollama is running
curl http://localhost:11434/api/tags

# Should output something like:
# {"models":[{"name":"nomic-embed-text:latest","modified_at":"2024-01-15T10:30:00.000Z","size":274000000}]}
```

## Usage

### Using FastEmbedEmbedder

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use cerebrum_core::orchestrator::MemoryOrchestrator;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create embedder with default Ollama endpoint
    let embedder = Arc::new(FastEmbedEmbedder::new());
    
    // Check if Ollama is available
    if !embedder.is_available().await {
        eprintln!("Ollama not available at http://localhost:11434");
        return Err("Ollama unavailable".into());
    }

    // Create orchestrator with real embeddings
    let orchestrator = MemoryOrchestrator::new(":memory:", embedder).await?;

    // Use orchestrator - embeddings are now real semantic vectors
    let results = orchestrator.recall("query".to_string(), 10).await?;
    
    Ok(())
}
```

### Custom Ollama Endpoint

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;

// Connect to Ollama on a different host/port
let embedder = Arc::new(FastEmbedEmbedder::with_endpoint(
    "http://192.168.1.100:11434".to_string()
));
```

### Monitoring Embeddings

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;

let embedder = FastEmbedEmbedder::new();
let metrics = embedder.metrics();

// Monitor embedding performance
println!("Total embeddings: {}", metrics.total_operations());
println!("Success rate: {:.1}%", metrics.success_rate());
println!("Average latency: {:.2}ms", metrics.average_time_ms());

// Check circuit breaker status
let cb = embedder.circuit_breaker();
match cb.allow_request() {
    Ok(()) => println!("Circuit breaker: CLOSED (healthy)"),
    Err(_) => println!("Circuit breaker: OPEN (recovering)"),
}
```

## Architecture

### Embedding Flow

```
Query Text
    ↓
FastEmbedEmbedder
    ↓
Circuit Breaker Check
    ↓
HTTP POST to Ollama
    ↓
/api/embed endpoint
    ↓
nomic-embed-text model
    ↓
384-dimensional vector
    ↓
Metrics Recording
    ↓
Return to Caller
```

### Circuit Breaker Behavior

The circuit breaker protects against cascading failures:

1. **CLOSED (Normal):** Requests are allowed, failures are counted
2. **OPEN (Failing):** After 5 consecutive failures, requests are denied
3. **HALF_OPEN (Testing):** After 60 seconds, one request is allowed to test recovery
4. **CLOSED (Recovered):** If test succeeds, circuit closes and normal operation resumes

### Metrics Tracking

All embedding operations are tracked:

- **total_operations:** Total embedding requests
- **successful_operations:** Successful embeddings
- **failed_operations:** Failed embeddings
- **total_time_ms:** Cumulative latency
- **success_rate():** Percentage of successful operations
- **average_time_ms():** Average latency per operation

## Troubleshooting

### Ollama Not Running

**Error:** `Cannot connect to Ollama at http://localhost:11434`

**Solution:**
```bash
# Start Ollama
ollama serve

# In another terminal, verify it's running
curl http://localhost:11434/api/tags
```

### Model Not Available

**Error:** `Model not found: nomic-embed-text`

**Solution:**
```bash
# Pull the model
ollama pull nomic-embed-text

# Verify it's available
ollama list
```

### Circuit Breaker Open

**Error:** Circuit breaker is open and rejecting requests

**Explanation:** The system detected 5 consecutive failures and opened the circuit to prevent cascading failures.

**Solution:**
1. Check Ollama server status: `curl http://localhost:11434/api/tags`
2. Wait 60 seconds for circuit breaker to transition to HALF_OPEN
3. Once Ollama recovers, circuit will automatically close

### Slow Embeddings

**Issue:** First embedding request is slow

**Explanation:** Ollama loads the model into memory on first use (~2-5 seconds).

**Solution:**
- Subsequent requests are fast (model stays in memory)
- Monitor metrics to track average latency
- Consider pre-warming by embedding a test string on startup

### Memory Issues

**Issue:** Ollama crashes or becomes unresponsive

**Solution:**
1. Check available system memory: `free -h` (Linux) or `vm_stat` (macOS)
2. Increase Ollama's memory allocation if needed
3. Restart Ollama: `killall ollama && ollama serve`

## Performance Tuning

### Batch Embeddings

For better throughput, batch multiple embeddings:

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let embedder = Arc::new(FastEmbedEmbedder::new());
    
    let texts = vec![
        "First memory",
        "Second memory",
        "Third memory",
    ];
    
    // Embed in parallel
    let futures: Vec<_> = texts.iter()
        .map(|text| embedder.embed(text))
        .collect();
    
    let embeddings = futures::future::try_join_all(futures).await?;
    
    Ok(())
}
```

### Caching Embeddings

Cerebrum automatically caches embeddings in MemoryEntry:

```rust
// Embedding is computed once and cached
let entry = MemoryEntry::builder(id, "content".to_string())
    .embedding(embedder.embed("content").await?)
    .build();

// Subsequent retrievals use cached embedding
let results = synapse.retrieve("query", 10).await?;
```

## API Reference

### FastEmbedEmbedder

```rust
pub struct FastEmbedEmbedder {
    endpoint: String,      // Ollama endpoint URL
    model: String,         // Model name (default: nomic-embed-text)
    metrics: Arc<OperationMetrics>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl FastEmbedEmbedder {
    // Create with default endpoint (http://localhost:11434)
    pub fn new() -> Self

    // Create with custom endpoint
    pub fn with_endpoint(endpoint: String) -> Self

    // Create with custom endpoint and model
    pub fn with_config(endpoint: String, model: String) -> Self

    // Get embedding dimension (384 for nomic-embed-text)
    pub fn embedding_dim(&self) -> usize

    // Get metrics for monitoring
    pub fn metrics(&self) -> Arc<OperationMetrics>

    // Get circuit breaker for status checking
    pub fn circuit_breaker(&self) -> Arc<CircuitBreaker>

    // Check if Ollama is available (2-second timeout)
    pub async fn is_available(&self) -> bool

    // Embed text (implements Embedder trait)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>>
}
```

### OperationMetrics

```rust
pub struct OperationMetrics {
    pub total_operations: Arc<AtomicU64>,
    pub successful_operations: Arc<AtomicU64>,
    pub failed_operations: Arc<AtomicU64>,
    pub total_time_ms: Arc<AtomicU64>,
}

impl OperationMetrics {
    pub fn total_operations(&self) -> u64
    pub fn successful_operations(&self) -> u64
    pub fn failed_operations(&self) -> u64
    pub fn total_time_ms(&self) -> u64
    pub fn average_time_ms(&self) -> f64
    pub fn success_rate(&self) -> f64
    pub fn record_success(&self, duration_ms: u64)
    pub fn record_failure(&self, duration_ms: u64)
    pub fn reset(&self)
}
```

### CircuitBreaker

```rust
pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
    config: CircuitBreakerConfig,
    failure_count: Arc<AtomicU64>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self
    pub fn allow_request(&self) -> Result<()>
    pub fn record_success(&self)
    pub fn record_failure(&self)
    pub fn reset(&self)
}
```

## Best Practices

1. **Always check availability before using real embeddings:**
   ```rust
   if embedder.is_available().await {
       // Use real embeddings
   } else {
       // Fall back to mock embeddings
   }
   ```

2. **Monitor metrics in production:**
   ```rust
   let metrics = embedder.metrics();
   if metrics.success_rate() < 95.0 {
       eprintln!("Warning: embedding success rate is low");
   }
   ```

3. **Handle circuit breaker state:**
   ```rust
   match embedder.circuit_breaker().allow_request() {
       Ok(()) => { /* proceed */ },
       Err(_) => { /* circuit is open, retry later */ },
   }
   ```

4. **Pre-warm Ollama on startup:**
   ```rust
   // Embed a test string to load model into memory
   let _ = embedder.embed("test").await;
   ```

5. **Use appropriate timeouts:**
   - Embed timeout: 5 seconds (default)
   - Availability check: 2 seconds (default)
   - Circuit breaker recovery: 60 seconds (default)

## See Also

- [Architecture](architecture.md) - System design overview
- [Observability Guide](OBSERVABILITY_GUIDE.md) - Metrics and monitoring
- [README](../README.md) - Quick start guide
