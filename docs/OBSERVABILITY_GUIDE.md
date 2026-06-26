# Observability Guide

This guide covers monitoring, metrics collection, and structured logging in Cerebrum.

## Overview

Cerebrum provides comprehensive observability through:
- **Operation Metrics:** Success rates, timing, and failure tracking
- **Structured Logging:** Integration with `tracing` crate
- **OpenTelemetry Compatible:** Ready for production observability stacks

## Basic Metrics Collection

### Creating an Observability Context

```rust
use cerebrum_core::observability::ObservabilityContext;

let context = ObservabilityContext::new();

// Metrics are automatically collected during operations
let id = orchestrator.remember("Memory".to_string(), HashMap::new()).await?;
let results = orchestrator.recall("query".to_string(), 10).await?;

// View summary
context.log_summary();
```

### Accessing Metrics

```rust
// Get metrics for a specific operation
let remember_metrics = context.remember_metrics();
let recall_metrics = context.recall_metrics();

println!("Remember operations: {}", remember_metrics.total_operations());
println!("Recall operations: {}", recall_metrics.total_operations());
println!("Success rate: {:.2}%", recall_metrics.success_rate());
```

## Operation Metrics

### Available Metrics

Each operation tracks:
- **Total Operations:** Number of operations performed
- **Successful Operations:** Number of successful operations
- **Failed Operations:** Number of failed operations
- **Average Time:** Average duration of operations
- **Success Rate:** Percentage of successful operations

### Tracked Operations

1. **Remember:** Store memories in Synapse
2. **Recall:** Search across both tiers
3. **Memorize:** Promote memories to Cortex
4. **Forget:** Delete memories
5. **Promote:** Promotion strategy execution
6. **Decay:** Memory decay application

### Example: Monitoring Recall Performance

```rust
use cerebrum_core::observability::ObservabilityContext;
use std::time::Duration;

let context = ObservabilityContext::new();

// Perform recall operations
for i in 0..100 {
    let results = orchestrator.recall(
        format!("query {}", i),
        10
    ).await?;
}

// Check metrics
let metrics = context.recall_metrics();
println!("Total recalls: {}", metrics.total_operations());
println!("Successful: {}", metrics.successful_operations());
println!("Failed: {}", metrics.failed_operations());
println!("Success rate: {:.2}%", metrics.success_rate());
println!("Average time: {:?}", metrics.average_time());
```

## Structured Logging

### Enabling Tracing

Add to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Initialize Tracing Subscriber

```rust
use tracing_subscriber;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Your code here
}
```

### Structured Logging Example

```rust
use tracing::{info, warn, error};

// Log memory operations
info!(
    memory_id = %id,
    content_length = content.len(),
    "Memory stored successfully"
);

// Log warnings
warn!(
    query = %search_query,
    results_count = results.len(),
    "Recall returned fewer results than expected"
);

// Log errors
error!(
    error = %err,
    operation = "memorize",
    "Failed to promote memory to Cortex"
);
```

## Performance Monitoring

### Tracking Operation Timing

```rust
use cerebrum_core::observability::OperationTimer;
use std::time::Instant;

let start = Instant::now();

// Perform operation
let results = orchestrator.recall("query".to_string(), 10).await?;

let duration = start.elapsed();
println!("Recall took: {:?}", duration);

// Log with context
info!(
    duration_ms = duration.as_millis(),
    results_count = results.len(),
    "Recall operation completed"
);
```

### Identifying Performance Bottlenecks

```rust
let context = ObservabilityContext::new();

// Run operations
for i in 0..1000 {
    orchestrator.remember(
        format!("Memory {}", i),
        HashMap::new()
    ).await?;
}

// Analyze metrics
let metrics = context.remember_metrics();
if let Some(avg_time) = metrics.average_time() {
    if avg_time > Duration::from_millis(100) {
        warn!("Remember operations are slow: {:?}", avg_time);
    }
}
```

## Error Tracking

### Monitoring Failures

```rust
use cerebrum_core::observability::ObservabilityContext;

let context = ObservabilityContext::new();

// Perform operations that might fail
match orchestrator.recall("query".to_string(), 10).await {
    Ok(results) => {
        info!("Recall succeeded with {} results", results.len());
    }
    Err(e) => {
        error!("Recall failed: {}", e);
        
        // Check failure metrics
        let metrics = context.recall_metrics();
        println!("Failure rate: {:.2}%", 
            100.0 - metrics.success_rate());
    }
}
```

## Circuit Breaker Monitoring

### Monitoring Embedder Health

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;

let embedder = FastEmbedEmbedder::new();

// Check if Ollama is available
if !embedder.is_available().await {
    warn!("Ollama server is not available");
}

// Monitor circuit breaker state
let cb = embedder.circuit_breaker();
match cb.allow_request() {
    Ok(()) => info!("Circuit breaker: CLOSED (healthy)"),
    Err(_) => warn!("Circuit breaker: OPEN (recovering)"),
}

// Monitor embedding metrics
let metrics = embedder.metrics();
println!("Total embeddings: {}", metrics.total_operations());
println!("Success rate: {:.1}%", metrics.success_rate());
println!("Average latency: {:.2}ms", metrics.average_time_ms());

// Alert on low success rate
if metrics.success_rate() < 95.0 {
    error!("Embedding success rate is below 95%: {:.1}%", metrics.success_rate());
}
```

### Circuit Breaker State Transitions

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use tracing::{info, warn};

let embedder = FastEmbedEmbedder::new();
let cb = embedder.circuit_breaker();

// Monitor state transitions
for i in 0..10 {
    match cb.allow_request() {
        Ok(()) => {
            // Attempt operation
            match embedder.embed("test").await {
                Ok(_) => {
                    cb.record_success();
                    info!("Embedding succeeded, circuit: CLOSED");
                }
                Err(e) => {
                    cb.record_failure();
                    warn!("Embedding failed: {}, failures recorded", e);
                }
            }
        }
        Err(_) => {
            warn!("Circuit breaker is OPEN, request denied");
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

### Embedding Metrics Dashboard

```rust
use cerebrum_core::fastembed_embedder::FastEmbedEmbedder;
use std::time::Duration;

let embedder = FastEmbedEmbedder::new();

// Periodic metrics reporting
tokio::spawn(async move {
    loop {
        let metrics = embedder.metrics();
        let cb = embedder.circuit_breaker();
        
        println!("=== Embedding Metrics ===");
        println!("Total operations: {}", metrics.total_operations());
        println!("Successful: {}", metrics.successful_operations());
        println!("Failed: {}", metrics.failed_operations());
        println!("Success rate: {:.1}%", metrics.success_rate());
        println!("Average latency: {:.2}ms", metrics.average_time_ms());
        println!("Circuit breaker: {}", 
            if cb.allow_request().is_ok() { "CLOSED" } else { "OPEN" });
        println!();
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

## Circuit Breaker Monitoring

### Monitoring Embedder Health

```rust
use cerebrum_core::resilience::{CircuitBreaker, CircuitBreakerConfig};

let breaker = CircuitBreaker::new(CircuitBreakerConfig::new());

// Monitor circuit breaker state
loop {
    match breaker.allow_request() {
        Ok(()) => {
            // Circuit is closed or half-open, proceed
            match perform_operation().await {
                Ok(result) => {
                    breaker.record_success();
                    info!("Operation succeeded");
                }
                Err(e) => {
                    breaker.record_failure();
                    error!("Operation failed: {}", e);
                }
            }
        }
        Err(_) => {
            // Circuit is open, skip operation
            warn!("Circuit breaker is open, skipping operation");
        }
    }
}
```

## Integration with OpenTelemetry

### Setup OpenTelemetry

Add to `Cargo.toml`:

```toml
[dependencies]
opentelemetry = "0.20"
opentelemetry-jaeger = "0.19"
tracing-opentelemetry = "0.21"
```

### Initialize OpenTelemetry

```rust
use opentelemetry_jaeger;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;

fn init_tracing() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .install_simple()
        .expect("Failed to install OpenTelemetry tracer");

    let telemetry = OpenTelemetryLayer::new(tracer);
    
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

### Trace Memory Operations

```rust
use tracing::Span;

let span = tracing::info_span!(
    "memory_operation",
    operation = "recall",
    query = %search_query
);

let _guard = span.enter();

let results = orchestrator.recall(search_query, 10).await?;

info!(results_count = results.len(), "Recall completed");
```

## Metrics Export

### Exporting to Prometheus

Add to `Cargo.toml`:

```toml
[dependencies]
prometheus = "0.13"
```

### Collect Metrics for Export

```rust
use cerebrum_core::observability::ObservabilityContext;

let context = ObservabilityContext::new();

// Perform operations...

// Export metrics
fn export_metrics(context: &ObservabilityContext) -> String {
    format!(
        "remember_total: {}\n\
         recall_total: {}\n\
         memorize_total: {}\n\
         forget_total: {}\n\
         promote_total: {}\n\
         decay_total: {}",
        context.remember_metrics().total_operations(),
        context.recall_metrics().total_operations(),
        context.memorize_metrics().total_operations(),
        context.forget_metrics().total_operations(),
        context.promote_metrics().total_operations(),
        context.decay_metrics().total_operations(),
    )
}
```

## Best Practices

### 1. Always Initialize Observability

```rust
let context = ObservabilityContext::new();
// Use context throughout application lifetime
```

### 2. Log at Appropriate Levels

- **ERROR:** Operation failures, system errors
- **WARN:** Degraded performance, unexpected conditions
- **INFO:** Operation summaries, important events
- **DEBUG:** Detailed operation information

```rust
error!("Critical failure: {}", err);
warn!("Performance degradation detected");
info!("Operation completed successfully");
debug!("Detailed operation info: {:?}", details);
```

### 3. Include Context in Logs

```rust
info!(
    memory_id = %id,
    content_length = content.len(),
    tier = "synapse",
    "Memory stored"
);
```

### 4. Monitor Success Rates

```rust
let metrics = context.recall_metrics();
if metrics.success_rate() < 95.0 {
    warn!("Recall success rate below 95%: {:.2}%", metrics.success_rate());
}
```

### 5. Track Performance Trends

```rust
// Periodically log metrics
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        context.log_summary();
    }
});
```

## Troubleshooting

### High Failure Rate

```rust
let metrics = context.recall_metrics();
if metrics.failed_operations() > 0 {
    error!(
        total = metrics.total_operations(),
        failed = metrics.failed_operations(),
        "High failure rate detected"
    );
}
```

### Slow Operations

```rust
if let Some(avg_time) = context.recall_metrics().average_time() {
    if avg_time > Duration::from_millis(500) {
        warn!("Slow recall operations: {:?}", avg_time);
    }
}
```

### Memory Leaks

Monitor operation counts over time:

```rust
let initial_count = context.remember_metrics().total_operations();

// Wait and check again
tokio::time::sleep(Duration::from_secs(60)).await;

let final_count = context.remember_metrics().total_operations();
let ops_per_sec = (final_count - initial_count) / 60;

info!("Operations per second: {}", ops_per_sec);
```

## See Also

- [Migration Guide](MIGRATION_GUIDE.md) - Monitor migrations
- [Architecture](architecture.md) - System design
