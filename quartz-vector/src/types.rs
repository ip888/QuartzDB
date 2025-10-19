//! Vector type and ID definitions

use serde::{Deserialize, Serialize};

/// Unique identifier for a vector
pub type VectorId = u64;

/// Result of a vector similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The ID of the found vector
    pub id: VectorId,
    /// Distance or similarity score (interpretation depends on metric)
    pub score: f32,
    /// Optional metadata associated with the vector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

impl SearchResult {
    pub fn new(id: VectorId, score: f32) -> Self {
        Self {
            id,
            score,
            metadata: None,
        }
    }

    pub fn with_metadata(id: VectorId, score: f32, metadata: String) -> Self {
        Self {
            id,
            score,
            metadata: Some(metadata),
        }
    }
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && (self.score - other.score).abs() < f32::EPSILON
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by score (descending for similarity, ascending for distance)
        // We use reverse ordering to get highest scores first
        other.score.partial_cmp(&self.score).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_ordering() {
        let mut results = vec![
            SearchResult::new(1, 0.5),
            SearchResult::new(2, 0.9),
            SearchResult::new(3, 0.2),
        ];

        results.sort();

        assert_eq!(results[0].id, 2); // Highest score first
        assert_eq!(results[1].id, 1);
        assert_eq!(results[2].id, 3);
    }

    #[test]
    fn test_search_result_with_metadata() {
        let result = SearchResult::with_metadata(1, 0.9, "test metadata".to_string());
        assert_eq!(result.id, 1);
        assert_eq!(result.score, 0.9);
        assert_eq!(result.metadata, Some("test metadata".to_string()));
    }
}
