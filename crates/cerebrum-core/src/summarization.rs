//! Memory summarization strategies for compressing and distilling memories.
//!
//! This module provides pluggable summarization strategies that determine how
//! memories are compressed or distilled when promoted from Synapse to Cortex.

use crate::models::MemoryEntry;

/// Trait for memory summarization strategies.
///
/// Implementations define different approaches to summarizing or compressing
/// memories while preserving their essential meaning.
pub trait Summarizer: Send + Sync {
    /// Summarize a memory entry.
    ///
    /// Returns a new MemoryEntry with summarized content.
    /// The summarized entry should preserve the original ID, timestamp, and metadata.
    fn summarize(&self, entry: &MemoryEntry) -> MemoryEntry;

    /// Get the name of this summarizer.
    fn name(&self) -> &str;
}

/// Identity summarizer that returns the memory unchanged.
///
/// Useful as a no-op summarizer or for testing.
pub struct IdentitySummarizer;

impl Summarizer for IdentitySummarizer {
    fn summarize(&self, entry: &MemoryEntry) -> MemoryEntry {
        entry.clone()
    }

    fn name(&self) -> &str {
        "Identity"
    }
}

/// Length-based summarizer that truncates memories to a maximum length.
///
/// Preserves the beginning of the memory up to the specified length.
pub struct LengthBasedSummarizer {
    /// Maximum length of summarized content in characters
    pub max_length: usize,
}

impl LengthBasedSummarizer {
    /// Create a new length-based summarizer.
    ///
    /// # Arguments
    /// * `max_length` - Maximum length of summarized content in characters
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Summarizer for LengthBasedSummarizer {
    fn summarize(&self, entry: &MemoryEntry) -> MemoryEntry {
        let mut summarized = entry.clone();

        if entry.content.len() > self.max_length {
            // Truncate to max_length, ensuring we don't cut in the middle of a character
            summarized.content = entry
                .content
                .chars()
                .take(self.max_length)
                .collect::<String>();

            // Add ellipsis to indicate truncation
            if summarized.content.len() > 3 {
                summarized.content.truncate(summarized.content.len() - 3);
                summarized.content.push_str("...");
            }
        }

        summarized
    }

    fn name(&self) -> &str {
        "LengthBased"
    }
}

/// Keyword-based summarizer that extracts key terms and creates a summary.
///
/// Identifies important words and creates a condensed summary.
pub struct KeywordSummarizer {
    /// Maximum number of keywords to extract
    pub max_keywords: usize,
}

impl KeywordSummarizer {
    /// Create a new keyword-based summarizer.
    ///
    /// # Arguments
    /// * `max_keywords` - Maximum number of keywords to extract
    pub fn new(max_keywords: usize) -> Self {
        Self { max_keywords }
    }

    /// Extract keywords from text.
    ///
    /// Simple heuristic: words longer than 4 characters that aren't common stop words.
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "can",
            "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "what",
            "which", "who", "when", "where", "why", "how",
        ];

        let mut keywords: Vec<String> = text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| word.len() > 4 && !stop_words.contains(word))
            .map(|s| s.to_string())
            .collect();

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        keywords.retain(|k| seen.insert(k.clone()));

        keywords.truncate(self.max_keywords);
        keywords
    }
}

impl Summarizer for KeywordSummarizer {
    fn summarize(&self, entry: &MemoryEntry) -> MemoryEntry {
        let mut summarized = entry.clone();

        let keywords = self.extract_keywords(&entry.content);

        if !keywords.is_empty() {
            // Create a summary from keywords
            let summary = format!("Keywords: {}", keywords.join(", "));
            summarized.content = summary;
        }

        summarized
    }

    fn name(&self) -> &str {
        "KeywordBased"
    }
}

/// Sentence-based summarizer that keeps the first N sentences.
///
/// Useful for preserving the main idea while reducing length.
pub struct SentenceBasedSummarizer {
    /// Maximum number of sentences to keep
    pub max_sentences: usize,
}

impl SentenceBasedSummarizer {
    /// Create a new sentence-based summarizer.
    ///
    /// # Arguments
    /// * `max_sentences` - Maximum number of sentences to keep
    pub fn new(max_sentences: usize) -> Self {
        Self { max_sentences }
    }
}

impl Summarizer for SentenceBasedSummarizer {
    fn summarize(&self, entry: &MemoryEntry) -> MemoryEntry {
        let mut summarized = entry.clone();

        // Split by sentence delimiters
        let sentences: Vec<&str> = entry
            .content
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .take(self.max_sentences)
            .collect();

        if !sentences.is_empty() && sentences.len() < entry.content.split('.').count() {
            // Reconstruct with periods
            summarized.content = sentences.join(". ") + ".";
        }

        summarized
    }

    fn name(&self) -> &str {
        "SentenceBased"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MemoryId, MemoryTier};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_entry(content: &str) -> MemoryEntry {
        MemoryEntry {
            id: MemoryId::new(),
            content: content.to_string(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            salience: 0.5,
            tier: MemoryTier::Synapse,
            embedding: None,
            source_session_id: None,
            scope: crate::models::MemoryScope::Global,
        }
    }

    #[test]
    fn test_identity_summarizer() {
        let summarizer = IdentitySummarizer;
        let entry = create_test_entry("This is a test memory");

        let summarized = summarizer.summarize(&entry);
        assert_eq!(summarized.content, entry.content);
        assert_eq!(summarized.id, entry.id);
    }

    #[test]
    fn test_length_based_summarizer_short_content() {
        let summarizer = LengthBasedSummarizer::new(100);
        let entry = create_test_entry("Short");

        let summarized = summarizer.summarize(&entry);
        assert_eq!(summarized.content, "Short");
    }

    #[test]
    fn test_length_based_summarizer_long_content() {
        let summarizer = LengthBasedSummarizer::new(20);
        let entry = create_test_entry("This is a very long memory that should be truncated");

        let summarized = summarizer.summarize(&entry);
        assert!(summarized.content.len() <= 23); // 20 + "..."
        assert!(summarized.content.ends_with("..."));
    }

    #[test]
    fn test_keyword_summarizer() {
        let summarizer = KeywordSummarizer::new(5);
        let entry = create_test_entry("The important project deadline is tomorrow morning");

        let summarized = summarizer.summarize(&entry);
        assert!(summarized.content.contains("Keywords:"));
        assert!(summarized.content.contains("important") || summarized.content.contains("project"));
    }

    #[test]
    fn test_keyword_summarizer_empty() {
        let summarizer = KeywordSummarizer::new(5);
        let entry = create_test_entry("a an the");

        let summarized = summarizer.summarize(&entry);
        // Should keep original if no keywords found
        assert!(!summarized.content.is_empty());
    }

    #[test]
    fn test_sentence_based_summarizer_single_sentence() {
        let summarizer = SentenceBasedSummarizer::new(1);
        let entry = create_test_entry("First sentence. Second sentence. Third sentence.");

        let summarized = summarizer.summarize(&entry);
        assert!(summarized.content.contains("First sentence"));
        assert!(!summarized.content.contains("Second"));
    }

    #[test]
    fn test_sentence_based_summarizer_multiple_sentences() {
        let summarizer = SentenceBasedSummarizer::new(2);
        let entry = create_test_entry("First sentence. Second sentence. Third sentence.");

        let summarized = summarizer.summarize(&entry);
        assert!(summarized.content.contains("First"));
        assert!(summarized.content.contains("Second"));
        assert!(!summarized.content.contains("Third"));
    }

    #[test]
    fn test_summarizer_name() {
        let identity = IdentitySummarizer;
        assert_eq!(identity.name(), "Identity");

        let length = LengthBasedSummarizer::new(100);
        assert_eq!(length.name(), "LengthBased");

        let keyword = KeywordSummarizer::new(5);
        assert_eq!(keyword.name(), "KeywordBased");

        let sentence = SentenceBasedSummarizer::new(2);
        assert_eq!(sentence.name(), "SentenceBased");
    }

    #[test]
    fn test_summarizer_preserves_metadata() {
        let summarizer = LengthBasedSummarizer::new(10);
        let mut entry = create_test_entry("This is a very long memory");
        entry
            .metadata
            .insert("key".to_string(), "value".to_string());

        let summarized = summarizer.summarize(&entry);
        assert_eq!(summarized.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_summarizer_preserves_id() {
        let summarizer = KeywordSummarizer::new(5);
        let entry = create_test_entry("Test memory content");

        let summarized = summarizer.summarize(&entry);
        assert_eq!(summarized.id, entry.id);
    }
}
