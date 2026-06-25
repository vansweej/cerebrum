pub mod embedder;
pub mod error;
pub mod models;
pub mod synapse;
pub mod cortex;
pub mod traits;
pub mod utils;

// Re-export commonly used types
pub use embedder::{Embedder, MockEmbedder};
pub use error::{CerebrumError, Result};
pub use models::{MemoryEntry, MemoryId, MemoryTier};
pub use synapse::SynapseMemory;
pub use cortex::CortexMemory;
pub use traits::MemoryStore;
