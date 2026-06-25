use async_trait::async_trait;
use crate::error::Result;
use crate::models::{MemoryEntry, MemoryId};

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait MemoryTier: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> Result<()>;
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn delete(&self, id: &MemoryId) -> Result<()>;
}
