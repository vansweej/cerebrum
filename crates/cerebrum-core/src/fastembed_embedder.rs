use crate::error::{CerebrumError, Result};
use crate::traits::Embedder;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Request body for Ollama embedding API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

/// Response body from Ollama embedding API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Global HTTP client for Ollama requests
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

/// Ollama-based embedder using nomic-embed-text model (384-dimensional).
///
/// Provides real semantic embeddings for accurate similarity search.
/// Uses the nomic-embed-text model which is optimized for performance and quality.
/// Requires Ollama to be running at http://localhost:11434
pub struct FastEmbedEmbedder {
    /// Ollama endpoint URL
    endpoint: String,
    /// Model name (default: nomic-embed-text)
    model: String,
}

impl FastEmbedEmbedder {
    /// Create a new FastEmbed embedder with Ollama backend.
    ///
    /// # Arguments
    /// * `endpoint` - Ollama API endpoint (default: http://localhost:11434)
    /// * `model` - Model name (default: nomic-embed-text)
    pub fn new() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
        }
    }

    /// Create a new FastEmbed embedder with custom endpoint.
    pub fn with_endpoint(endpoint: String) -> Self {
        Self {
            endpoint,
            model: "nomic-embed-text".to_string(),
        }
    }

    /// Create a new FastEmbed embedder with custom endpoint and model.
    pub fn with_config(endpoint: String, model: String) -> Self {
        Self { endpoint, model }
    }

    /// Get the embedding dimension for this model.
    pub fn embedding_dim(&self) -> usize {
        384 // nomic-embed-text produces 384-dimensional embeddings
    }

    /// Check if Ollama is available at the configured endpoint.
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.endpoint);
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            HTTP_CLIENT.get(&url).send(),
        )
        .await
        {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }
}

impl Default for FastEmbedEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embedder for FastEmbedEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.endpoint);

        let request = OllamaEmbedRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = HTTP_CLIENT
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                CerebrumError::Embedding(format!(
                    "Failed to connect to Ollama at {}: {}",
                    self.endpoint, e
                ))
            })?;

        if !response.status().is_success() {
            return Err(CerebrumError::Embedding(format!(
                "Ollama API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let embed_response: OllamaEmbedResponse = response.json().await.map_err(|e| {
            CerebrumError::Embedding(format!("Failed to parse Ollama response: {}", e))
        })?;

        // Verify dimensions
        if embed_response.embedding.len() != 384 {
            return Err(CerebrumError::Validation(format!(
                "Invalid embedding dimension from Ollama: expected 384, got {}",
                embed_response.embedding.len()
            )));
        }

        Ok(embed_response.embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_fastembed_embedder_new() {
        let embedder = FastEmbedEmbedder::new();
        assert_eq!(embedder.endpoint, "http://localhost:11434");
        assert_eq!(embedder.model, "nomic-embed-text");
    }

    #[tokio::test]
    async fn test_fastembed_embedder_default() {
        let embedder = FastEmbedEmbedder::default();
        assert_eq!(embedder.endpoint, "http://localhost:11434");
        assert_eq!(embedder.model, "nomic-embed-text");
    }

    #[tokio::test]
    async fn test_fastembed_embedder_with_endpoint() {
        let embedder = FastEmbedEmbedder::with_endpoint("http://custom:11434".to_string());
        assert_eq!(embedder.endpoint, "http://custom:11434");
        assert_eq!(embedder.model, "nomic-embed-text");
    }

    #[tokio::test]
    async fn test_fastembed_embedder_with_config() {
        let embedder = FastEmbedEmbedder::with_config(
            "http://custom:11434".to_string(),
            "custom-model".to_string(),
        );
        assert_eq!(embedder.endpoint, "http://custom:11434");
        assert_eq!(embedder.model, "custom-model");
    }

    #[tokio::test]
    async fn test_fastembed_embedder_embedding_dim() {
        let embedder = FastEmbedEmbedder::new();
        assert_eq!(embedder.embedding_dim(), 384);
    }

    #[tokio::test]
    async fn test_fastembed_embedder_embed_requires_ollama() {
        let embedder = FastEmbedEmbedder::new();
        let result = embedder.embed("test text").await;

        // This test will fail if Ollama is not running
        // In CI/CD, this should be skipped or Ollama should be available
        match result {
            Ok(embedding) => {
                // Ollama is available
                assert_eq!(embedding.len(), 384);
            }
            Err(CerebrumError::Embedding(msg)) => {
                // Ollama is not available - this is expected in some environments
                assert!(msg.contains("Failed to connect") || msg.contains("Ollama"));
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_fastembed_embedder_consistency_requires_ollama() {
        let embedder = FastEmbedEmbedder::new();

        // Skip if Ollama is not available
        if !embedder.is_available().await {
            return;
        }

        let embedding1 = embedder.embed("hello world").await.unwrap();
        let embedding2 = embedder.embed("hello world").await.unwrap();

        // Same text should produce same embedding
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fastembed_embedder_different_texts_requires_ollama() {
        let embedder = FastEmbedEmbedder::new();

        // Skip if Ollama is not available
        if !embedder.is_available().await {
            return;
        }

        let embedding1 = embedder.embed("hello world").await.unwrap();
        let embedding2 = embedder.embed("goodbye world").await.unwrap();

        // Different texts should produce different embeddings
        assert_ne!(embedding1, embedding2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fastembed_embedder_empty_text_requires_ollama() {
        let embedder = FastEmbedEmbedder::new();

        // Skip if Ollama is not available
        if !embedder.is_available().await {
            return;
        }

        let embedding = embedder.embed("").await;

        // Empty text should still produce an embedding
        assert!(embedding.is_ok());
        let vec = embedding.unwrap();
        assert_eq!(vec.len(), 384);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fastembed_embedder_concurrent_access_requires_ollama() {
        let embedder = Arc::new(FastEmbedEmbedder::new());

        // Skip if Ollama is not available
        if !embedder.is_available().await {
            return;
        }

        // Create multiple concurrent embedding tasks
        let mut handles = vec![];
        for i in 0..3 {
            let embedder_clone = Arc::clone(&embedder);
            let handle = tokio::spawn(async move {
                let text = format!("text {}", i);
                embedder_clone.embed(&text).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            let embedding_result = result.unwrap();
            assert!(embedding_result.is_ok());
            let vec = embedding_result.unwrap();
            assert_eq!(vec.len(), 384);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_fastembed_embedder_normalized_requires_ollama() {
        let embedder = FastEmbedEmbedder::new();

        // Skip if Ollama is not available
        if !embedder.is_available().await {
            return;
        }

        let embedding = embedder.embed("test").await.unwrap();

        // Embedding should be normalized (magnitude close to 1)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }
}
