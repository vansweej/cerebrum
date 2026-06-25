use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MemoryId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
