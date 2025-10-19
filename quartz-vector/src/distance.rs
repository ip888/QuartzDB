//! Distance metrics for vector similarity

use serde::{Deserialize, Serialize};

/// Distance metrics for measuring vector similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Cosine similarity (1 - cosine distance)
    /// Range: [-1, 1], where 1 means identical direction
    /// Best for: Text embeddings, normalized vectors
    Cosine,

    /// Euclidean distance (L2 norm)
    /// Range: [0, ∞], where 0 means identical vectors
    /// Best for: Image embeddings, when magnitude matters
    Euclidean,

    /// Dot product (inner product)
    /// Range: (-∞, ∞), higher means more similar
    /// Best for: Normalized vectors, when you want magnitude-weighted similarity
    DotProduct,
}

impl DistanceMetric {
    /// Calculate distance/similarity between two vectors
    ///
    /// Note: For Cosine and DotProduct, higher values mean more similar.
    /// For Euclidean, lower values mean more similar.
    pub fn calculate(&self, v1: &[f32], v2: &[f32]) -> f32 {
        assert_eq!(v1.len(), v2.len(), "Vectors must have same dimension");

        match self {
            DistanceMetric::Cosine => cosine_similarity(v1, v2),
            DistanceMetric::Euclidean => euclidean_distance(v1, v2),
            DistanceMetric::DotProduct => dot_product(v1, v2),
        }
    }

    /// Returns true if higher scores mean more similar (Cosine, DotProduct)
    /// Returns false if lower scores mean more similar (Euclidean)
    pub fn higher_is_better(&self) -> bool {
        matches!(self, DistanceMetric::Cosine | DistanceMetric::DotProduct)
    }
}

/// Calculate cosine similarity between two vectors
///
/// Returns a value in [-1, 1] where:
/// - 1.0 means vectors point in the same direction
/// - 0.0 means vectors are orthogonal
/// - -1.0 means vectors point in opposite directions
#[inline]
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot = dot_product(v1, v2);
    let mag1 = magnitude(v1);
    let mag2 = magnitude(v2);

    if mag1 == 0.0 || mag2 == 0.0 {
        return 0.0;
    }

    dot / (mag1 * mag2)
}

/// Calculate Euclidean distance between two vectors
///
/// Returns the L2 norm of the difference between vectors.
/// Lower values indicate more similar vectors.
#[inline]
pub fn euclidean_distance(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter()
        .zip(v2.iter())
        .map(|(a, b)| {
            let diff = a - b;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

/// Calculate dot product (inner product) of two vectors
///
/// Returns the sum of element-wise products.
/// Higher values indicate more similar vectors (for normalized vectors).
#[inline]
pub fn dot_product(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
}

/// Calculate the magnitude (L2 norm) of a vector
#[inline]
pub fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Normalize a vector to unit length
pub fn normalize(v: &mut [f32]) {
    let mag = magnitude(v);
    if mag > 0.0 {
        for x in v.iter_mut() {
            *x /= mag;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            approx_eq(sim, 1.0),
            "Identical vectors should have similarity 1.0"
        );
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            approx_eq(sim, 0.0),
            "Orthogonal vectors should have similarity 0.0"
        );
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!(
            approx_eq(sim, -1.0),
            "Opposite vectors should have similarity -1.0"
        );
    }

    #[test]
    fn test_euclidean_distance_identical() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let dist = euclidean_distance(&v1, &v2);
        assert!(
            approx_eq(dist, 0.0),
            "Identical vectors should have distance 0.0"
        );
    }

    #[test]
    fn test_euclidean_distance() {
        let v1 = vec![0.0, 0.0, 0.0];
        let v2 = vec![3.0, 4.0, 0.0];
        let dist = euclidean_distance(&v1, &v2);
        assert!(
            approx_eq(dist, 5.0),
            "Distance should be 5.0 (3-4-5 triangle)"
        );
    }

    #[test]
    fn test_dot_product() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![4.0, 5.0, 6.0];
        let dot = dot_product(&v1, &v2);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!(approx_eq(dot, 32.0));
    }

    #[test]
    fn test_magnitude() {
        let v = vec![3.0, 4.0, 0.0];
        let mag = magnitude(&v);
        assert!(approx_eq(mag, 5.0), "Magnitude should be 5.0");
    }

    #[test]
    fn test_normalize() {
        let mut v = vec![3.0, 4.0, 0.0];
        normalize(&mut v);
        let mag = magnitude(&v);
        assert!(
            approx_eq(mag, 1.0),
            "Normalized vector should have magnitude 1.0"
        );
        assert!(approx_eq(v[0], 0.6));
        assert!(approx_eq(v[1], 0.8));
    }

    #[test]
    fn test_distance_metric_calculate() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];

        let cosine = DistanceMetric::Cosine.calculate(&v1, &v2);
        assert!(approx_eq(cosine, 0.0));

        let euclidean = DistanceMetric::Euclidean.calculate(&v1, &v2);
        assert!(approx_eq(euclidean, 2.0_f32.sqrt()));

        let dot = DistanceMetric::DotProduct.calculate(&v1, &v2);
        assert!(approx_eq(dot, 0.0));
    }

    #[test]
    fn test_higher_is_better() {
        assert!(DistanceMetric::Cosine.higher_is_better());
        assert!(DistanceMetric::DotProduct.higher_is_better());
        assert!(!DistanceMetric::Euclidean.higher_is_better());
    }
}
