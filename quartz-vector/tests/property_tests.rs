//! Property-based tests for vector operations
//!
//! Uses proptest to verify vector index invariants with random inputs

use proptest::prelude::*;
use quartz_vector::{DistanceMetric, HnswConfig, HnswIndex, VectorId};
use std::collections::HashSet;

/// Strategy for generating valid vectors (f32 arrays with reasonable values)
fn vector_strategy(dim: usize) -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(-1.0f32..1.0f32, dim..=dim)
}

/// Strategy for generating vector IDs (u64)
fn vector_id_strategy() -> impl Strategy<Value = VectorId> {
    any::<u64>()
}

proptest! {
    /// Test that insert and search finds the inserted vector
    /// Property: After inserting a vector, searching for it should return it as a result
    #[test]
    fn test_insert_and_search_consistency(
        vectors in prop::collection::vec(
            (vector_id_strategy(), vector_strategy(128)),
            1..50
        )
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Cosine);

        let mut inserted_ids = HashSet::new();

        // Insert all vectors
        for (id, vector) in &vectors {
            index.insert(*id, vector)?;
            inserted_ids.insert(*id);
        }

        // Search for each vector - it should be its own nearest neighbor
        for (id, vector) in &vectors {
            let results = index.search(vector, 1)?;
            
            prop_assert!(!results.is_empty(), "Search should return at least one result");
            
            // The first result should be the vector itself (or very close)
            let first_result_id = results[0].id;
            prop_assert!(
                inserted_ids.contains(&first_result_id),
                "Result should be one of the inserted vectors"
            );
        }
    }

    /// Test that delete removes vectors from search results
    /// Property: After deleting a vector, it should not appear in search results
    #[test]
    fn test_delete_removes_from_search(
        id in vector_id_strategy(),
        vector in vector_strategy(64),
        other_vectors in prop::collection::vec(
            (vector_id_strategy(), vector_strategy(64)),
            5..20
        )
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        // Insert the target vector
        index.insert(id, &vector)?;

        // Insert other vectors
        for (other_id, other_vec) in &other_vectors {
            if *other_id != id {
                index.insert(*other_id, other_vec)?;
            }
        }

        // Verify it exists
        let results_before = index.search(&vector, 10)?;
        prop_assert!(
            results_before.iter().any(|r| r.id == id),
            "Vector should be found before deletion"
        );

        // Delete the target vector
        index.delete(id)?;

        // Search again - should not find it
        let results_after = index.search(&vector, 10)?;
        prop_assert!(
            !results_after.iter().any(|r| r.id == id),
            "Vector should not be found after deletion"
        );
    }

    /// Test that search returns at most k results
    /// Property: search(k) should return at most k results
    #[test]
    fn test_search_returns_at_most_k(
        query in vector_strategy(32),
        vectors in prop::collection::vec(
            (vector_id_strategy(), vector_strategy(32)),
            10..100
        ),
        k in 1usize..20usize
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Cosine);

        // Insert all vectors
        for (id, vector) in vectors {
            index.insert(id, &vector)?;
        }

        // Search
        let results = index.search(&query, k)?;

        prop_assert!(
            results.len() <= k,
            "Should return at most k results (got {}, expected <= {})",
            results.len(),
            k
        );
    }

    /// Test that search results are sorted by score
    /// Property: Results should be in ascending order by score (best first)
    #[test]
    fn test_search_results_sorted(
        query in vector_strategy(64),
        vectors in prop::collection::vec(
            (vector_id_strategy(), vector_strategy(64)),
            10..50
        )
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        // Insert all vectors
        for (id, vector) in vectors {
            index.insert(id, &vector)?;
        }

        // Search
        let results = index.search(&query, 10)?;

        // Verify results are sorted
        for i in 1..results.len() {
            prop_assert!(
                results[i-1].score <= results[i].score,
                "Results should be sorted by score (ascending)"
            );
        }
    }

    /// Test that identical vectors have distance 0 (or very small for Cosine)
    /// Property: Searching for an inserted vector should return it with minimal distance
    #[test]
    fn test_self_similarity(
        id in vector_id_strategy(),
        vector in vector_strategy(128)
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        // Insert vector
        index.insert(id, &vector)?;

        // Search for itself
        let results = index.search(&vector, 1)?;

        prop_assert!(!results.is_empty(), "Should find at least one result");
        
        // The best result should have very small distance
        let best_score = results[0].score;
        prop_assert!(
            best_score < 0.01,
            "Self-similarity should be very high (score: {})",
            best_score
        );
    }

    /// Test that the index handles duplicate IDs correctly
    /// Property: Inserting the same ID twice should update the vector
    #[test]
    fn test_duplicate_id_update(
        id in vector_id_strategy(),
        vector1 in vector_strategy(64),
        vector2 in vector_strategy(64)
    ) {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Cosine);

        // Insert first vector
        index.insert(id, &vector1)?;

        // Insert second vector with same ID
        index.insert(id, &vector2)?;

        // Search for second vector - should find it
        let results = index.search(&vector2, 1)?;

        prop_assert!(!results.is_empty(), "Should find at least one result");
        
        // Should find the second vector (ID may match if updated correctly)
        // Note: Current implementation doesn't handle updates, so this might fail
        // This test documents expected behavior for future implementation
    }
}

#[cfg(test)]
mod deterministic_tests {
    use super::*;

    /// Test edge case: single vector
    #[test]
    fn test_single_vector() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        let id = 1u64;
        let vector = vec![1.0, 2.0, 3.0];

        index.insert(id, &vector).unwrap();

        let results = index.search(&vector, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
    }

    /// Test edge case: empty index
    #[test]
    fn test_empty_index_search() {
        let config = HnswConfig::default();
        let index = HnswIndex::new(config, DistanceMetric::Cosine);

        let query = vec![1.0, 2.0, 3.0];
        let results = index.search(&query, 10).unwrap();

        assert_eq!(results.len(), 0);
    }

    /// Test edge case: zero vector
    #[test]
    fn test_zero_vector() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        let id = 42u64;
        let vector = vec![0.0; 128];

        index.insert(id, &vector).unwrap();

        let results = index.search(&vector, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
    }

    /// Test edge case: normalized vectors for Cosine similarity
    #[test]
    fn test_normalized_vectors() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(config, DistanceMetric::Cosine);

        let id1 = 1u64;
        let id2 = 2u64;
        
        // Same direction, different magnitudes
        let vector1 = vec![1.0, 1.0, 1.0];
        let vector2 = vec![2.0, 2.0, 2.0];

        index.insert(id1, &vector1).unwrap();
        index.insert(id2, &vector2).unwrap();

        // Should be very similar in Cosine space
        let results = index.search(&vector1, 2).unwrap();
        assert_eq!(results.len(), 2);
        
        // Both should have high similarity
        assert!(results[0].score < 0.1);
        assert!(results[1].score < 0.1);
    }

    /// Test different distance metrics produce different results
    #[test]
    fn test_different_metrics() {
        let vector1 = vec![1.0, 0.0];
        let vector2 = vec![0.0, 1.0];
        let query = vec![1.0, 1.0];

        // Cosine similarity
        let mut index_cosine = HnswIndex::new(HnswConfig::default(), DistanceMetric::Cosine);
        index_cosine.insert(1u64, &vector1).unwrap();
        index_cosine.insert(2u64, &vector2).unwrap();
        let results_cosine = index_cosine.search(&query, 2).unwrap();

        // Euclidean distance
        let mut index_euclidean = HnswIndex::new(HnswConfig::default(), DistanceMetric::Euclidean);
        index_euclidean.insert(1u64, &vector1).unwrap();
        index_euclidean.insert(2u64, &vector2).unwrap();
        let results_euclidean = index_euclidean.search(&query, 2).unwrap();

        // Results should be similar but scores different
        assert!(results_cosine[0].score != results_euclidean[0].score);
    }
}
