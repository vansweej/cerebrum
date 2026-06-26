use crate::error::Result;
use crate::models::{MemoryEntry, MemoryId, MemoryScope};
use async_trait::async_trait;

/// Trait for embedding text into vector space.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Embed text into a vector.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

/// Trait for a memory storage tier (Synapse or Cortex).
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a memory entry.
    async fn store(&self, entry: MemoryEntry) -> Result<()>;

    /// Retrieve memories matching a query, up to a limit.
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;

    /// Retrieve memories matching a query and scope, up to a limit.
    async fn retrieve_by_scope(
        &self,
        query: &str,
        scope: &MemoryScope,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>>;

    /// Delete a memory by ID.
    async fn delete(&self, id: &MemoryId) -> Result<()>;

    /// List all memories in the store.
    async fn list(&self) -> Result<Vec<MemoryEntry>> {
        // Default implementation: retrieve with a wildcard query and max limit
        // Use a non-empty query to avoid embedder validation errors
        self.retrieve("*", usize::MAX).await
    }

    /// Get the number of memories in the store.
    async fn len(&self) -> Result<usize> {
        // Default implementation: count items in list
        Ok(self.list().await?.len())
    }

    /// Check if the store is empty.
    async fn is_empty(&self) -> Result<bool> {
        // Default implementation: check if len is 0
        Ok(self.len().await? == 0)
    }
}
