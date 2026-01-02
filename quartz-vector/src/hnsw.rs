//! HNSW (Hierarchical Navigable Small World) index implementation
//!
//! HNSW is a graph-based algorithm for approximate nearest neighbor search.
//! It builds a multi-layer graph where:
//! - Layer 0 contains all vectors
//! - Higher layers contain progressively fewer vectors
//! - Each vector connects to M neighbors at each layer
//!
//! Search starts at the top layer and greedily navigates to the nearest neighbors,
//! descending through layers until reaching layer 0.

use crate::{DistanceMetric, Result, SearchResult, VectorError, VectorId};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Configuration for HNSW index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Maximum number of bi-directional links per element per layer (M)
    /// Typical values: 5-48
    /// Higher M = better recall, more memory, slower insertions
    pub max_connections: usize,

    /// Maximum number of connections for layer 0 (typically M * 2)
    pub max_connections_layer0: usize,

    /// Size of the dynamic candidate list during construction (ef_construction)
    /// Typical values: 100-500
    /// Higher ef_construction = better quality index, slower construction
    pub ef_construction: usize,

    /// Size of the dynamic candidate list during search (ef_search)
    /// Typical values: 100-500
    /// Higher ef_search = better recall, slower search
    pub ef_search: usize,

    /// Normalization factor for level selection (ml)
    /// Typically 1.0 / ln(M)
    pub level_multiplier: f64,
}

impl Default for HnswConfig {
    fn default() -> Self {
        let m = 16;
        Self {
            max_connections: m,
            max_connections_layer0: m * 2,
            ef_construction: 200,
            ef_search: 100,
            level_multiplier: 1.0 / (m as f64).ln(),
        }
    }
}

impl HnswConfig {
    /// Create a fast configuration (less accuracy, faster search)
    pub fn fast() -> Self {
        let m = 8;
        Self {
            max_connections: m,
            max_connections_layer0: m * 2,
            ef_construction: 100,
            ef_search: 50,
            level_multiplier: 1.0 / (m as f64).ln(),
        }
    }

    /// Create a balanced configuration (default)
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Create a high-quality configuration (better accuracy, slower search)
    pub fn high_quality() -> Self {
        let m = 32;
        Self {
            max_connections: m,
            max_connections_layer0: m * 2,
            ef_construction: 400,
            ef_search: 200,
            level_multiplier: 1.0 / (m as f64).ln(),
        }
    }
}

/// A node in the HNSW graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HnswNode {
    /// Vector ID
    id: VectorId,
    /// Layer at which this node was inserted
    level: usize,
    /// Connections at each layer (layer -> set of neighbor IDs)
    connections: Vec<HashSet<VectorId>>,
}

impl HnswNode {
    fn new(id: VectorId, level: usize) -> Self {
        let connections = (0..=level).map(|_| HashSet::new()).collect();
        Self {
            id,
            level,
            connections,
        }
    }
}

/// HNSW index for fast approximate nearest neighbor search
pub struct HnswIndex {
    /// Configuration
    config: HnswConfig,
    /// Distance metric
    metric: DistanceMetric,
    /// All nodes in the graph
    nodes: HashMap<VectorId, HnswNode>,
    /// Cached vector data for distance calculations
    vectors: HashMap<VectorId, Vec<f32>>,
    /// Entry point (node at highest layer)
    entry_point: Option<VectorId>,
    /// Maximum layer in the graph
    max_layer: usize,
}

impl HnswIndex {
    /// Create a new HNSW index
    pub fn new(config: HnswConfig, metric: DistanceMetric) -> Self {
        Self {
            config,
            metric,
            nodes: HashMap::new(),
            vectors: HashMap::new(),
            entry_point: None,
            max_layer: 0,
        }
    }

    /// Insert a vector into the index
    pub fn insert(&mut self, id: VectorId, vector: &[f32]) -> Result<()> {
        // Store vector data
        self.vectors.insert(id, vector.to_vec());

        // Select layer for this element
        let level = self.select_layer();

        // Create node
        let mut node = HnswNode::new(id, level);

        // If this is the first element
        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_layer = level;
            self.nodes.insert(id, node);
            return Ok(());
        }

        // Find nearest neighbors and add connections
        let entry_id = self.entry_point
            .ok_or_else(|| VectorError::IndexError("Entry point not initialized".to_string()))?;
        let mut current_nearest = vec![entry_id];

        // Search from top layer down to target layer + 1
        for layer in (level + 1..=self.max_layer).rev() {
            current_nearest = self.search_layer(vector, &current_nearest, 1, layer)?;
        }

        // For each layer from target down to 0
        for layer in (0..=level).rev() {
            // Find ef_construction nearest neighbors
            let candidates =
                self.search_layer(vector, &current_nearest, self.config.ef_construction, layer)?;

            // Select M neighbors
            let m = if layer == 0 {
                self.config.max_connections_layer0
            } else {
                self.config.max_connections
            };

            let neighbors = self.select_neighbors(&candidates, m, vector)?;

            // Add bidirectional connections
            for &neighbor_id in &neighbors {
                node.connections[layer].insert(neighbor_id);
            }

            // Now update neighbors (separate loop to avoid borrow issues)
            for &neighbor_id in &neighbors {
                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    // Only update if neighbor has this layer
                    if layer < neighbor.connections.len() {
                        neighbor.connections[layer].insert(id);

                        // Prune connections if necessary
                        if neighbor.connections[layer].len() > m {
                            // Collect data we need before making mutable borrow
                            let connections: Vec<VectorId> =
                                neighbor.connections[layer].iter().copied().collect();
                            let neighbor_vec = match self.vectors.get(&neighbor_id) {
                                Some(v) => v.clone(),
                                None => continue,
                            };

                            // Release mutable borrow
                            let _ = neighbor;

                            // Select neighbors to keep
                            let to_keep_ids =
                                self.select_neighbors(&connections, m, &neighbor_vec)?;

                            // Re-borrow and update
                            if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                                neighbor.connections[layer] = to_keep_ids.into_iter().collect();
                            }
                        }
                    }
                }
            }

            current_nearest = candidates;
        }

        // Update entry point if this node is at a higher layer
        if level > self.max_layer {
            self.max_layer = level;
            self.entry_point = Some(id);
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// Search for k nearest neighbors
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        if self.entry_point.is_none() {
            return Ok(Vec::new());
        }

        let entry_id = self.entry_point
            .ok_or_else(|| VectorError::IndexError("Entry point not initialized".to_string()))?;
        let mut current_nearest = vec![entry_id];

        // Search from top layer down to layer 1
        for layer in (1..=self.max_layer).rev() {
            current_nearest = self.search_layer(query, &current_nearest, 1, layer)?;
        }

        // Search layer 0 with ef_search
        let ef = self.config.ef_search.max(k);
        current_nearest = self.search_layer(query, &current_nearest, ef, 0)?;

        // Return top k results
        let mut results: Vec<SearchResult> = current_nearest
            .iter()
            .take(k)
            .filter_map(|&id| {
                self.vectors.get(&id).map(|vector| {
                    let score = self.metric.calculate(query, vector);
                    SearchResult::new(id, score)
                })
            })
            .collect();

        // Sort by score
        results.sort();
        Ok(results)
    }

    /// Delete a vector from the index
    pub fn delete(&mut self, id: VectorId) -> Result<()> {
        let node = self.nodes.remove(&id).ok_or(VectorError::NotFound(id))?;

        // Remove all connections to this node
        for layer in 0..=node.level {
            for &neighbor_id in &node.connections[layer] {
                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    neighbor.connections[layer].remove(&id);
                }
            }
        }

        // Remove vector data
        self.vectors.remove(&id);

        // Update entry point if necessary
        if self.entry_point == Some(id) {
            self.entry_point = self.nodes.keys().next().copied();
            self.max_layer = self.nodes.values().map(|n| n.level).max().unwrap_or(0);
        }

        Ok(())
    }

    /// Search a single layer for nearest neighbors
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[VectorId],
        num_to_return: usize,
        layer: usize,
    ) -> Result<Vec<VectorId>> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut nearest = BinaryHeap::new();

        // Initialize with entry points
        for &ep in entry_points {
            if visited.insert(ep) {
                let dist = self.distance(query, ep)?;
                candidates.push(Reverse((OrderedFloat(dist), ep)));
                nearest.push((OrderedFloat(dist), ep));
            }
        }

        while let Some(Reverse((OrderedFloat(current_dist), current_id))) = candidates.pop() {
            // If current is farther than the farthest in nearest, stop
            if let Some(&(OrderedFloat(farthest_dist), _)) = nearest.peek()
                && current_dist > farthest_dist
            {
                break;
            }

            // Check all neighbors
            if let Some(node) = self.nodes.get(&current_id)
                && layer < node.connections.len()
            {
                for &neighbor_id in &node.connections[layer] {
                    if visited.insert(neighbor_id) {
                        let dist = self.distance(query, neighbor_id)?;

                        if nearest.len() < num_to_return
                            || dist
                                < nearest
                                    .peek()
                                    .map(|(OrderedFloat(d), _)| *d)
                                    .unwrap_or(f32::MAX)
                        {
                            candidates.push(Reverse((OrderedFloat(dist), neighbor_id)));
                            nearest.push((OrderedFloat(dist), neighbor_id));

                            if nearest.len() > num_to_return {
                                nearest.pop();
                            }
                        }
                    }
                }
            }
        }

        // Extract IDs in order of distance
        let mut result: Vec<_> = nearest.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result.into_iter().map(|(_, id)| id).collect())
    }

    /// Select M neighbors from candidates using heuristic
    fn select_neighbors(
        &self,
        candidates: &[VectorId],
        m: usize,
        query: &[f32],
    ) -> Result<Vec<VectorId>> {
        if candidates.len() <= m {
            return Ok(candidates.to_vec());
        }

        // Simple heuristic: select M nearest
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|&id| {
                let dist = self.distance(query, id).unwrap_or(f32::MAX);
                (OrderedFloat(dist), id)
            })
            .collect();

        scored.sort_by_key(|(d, _)| *d);
        Ok(scored.iter().take(m).map(|(_, id)| *id).collect())
    }

    /// Calculate distance between query and a stored vector
    fn distance(&self, query: &[f32], id: VectorId) -> Result<f32> {
        let vector = self.vectors.get(&id).ok_or(VectorError::NotFound(id))?;

        let score = self.metric.calculate(query, vector);

        // For similarity metrics (higher is better), convert to distance (lower is better)
        // For distance metrics (lower is better), use as-is
        let distance = if self.metric.higher_is_better() {
            1.0 - score // Convert similarity to distance
        } else {
            score // Already a distance
        };

        Ok(distance)
    }

    /// Select a random layer for a new element
    fn select_layer(&self) -> usize {
        let uniform: f64 = rand::random();
        let level = (-uniform.ln() * self.config.level_multiplier).floor() as usize;
        level.min(16) // Cap at 16 layers
    }
}

/// Wrapper for f32 to make it orderable (for BinaryHeap)
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedFloat(f32);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_config_default() {
        let config = HnswConfig::default();
        assert_eq!(config.max_connections, 16);
        assert_eq!(config.max_connections_layer0, 32);
    }

    #[test]
    fn test_hnsw_insert_and_search() {
        let config = HnswConfig::fast();
        let mut index = HnswIndex::new(config, DistanceMetric::Cosine);

        // Insert some vectors
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.9, 0.1, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        index.insert(1, &v1).unwrap();
        index.insert(2, &v2).unwrap();
        index.insert(3, &v3).unwrap();

        // Verify vectors were inserted
        assert_eq!(index.vectors.len(), 3);
        assert_eq!(index.nodes.len(), 3);

        // Search for nearest to v1
        let results = index.search(&v1, 3).unwrap();

        // Should find at least some results
        assert!(!results.is_empty(), "Search returned no results");
        println!("Search results: {:?}", results);

        // Should find vector 1 in top 3 results
        assert!(
            results.iter().any(|r| r.id == 1),
            "Didn't find vector 1 in results"
        );
    }

    #[test]
    fn test_hnsw_delete() {
        let config = HnswConfig::fast();
        let mut index = HnswIndex::new(config, DistanceMetric::Euclidean);

        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];

        index.insert(1, &v1).unwrap();
        index.insert(2, &v2).unwrap();

        assert_eq!(index.nodes.len(), 2);

        index.delete(1).unwrap();
        assert_eq!(index.nodes.len(), 1);
        assert!(!index.nodes.contains_key(&1));
    }
}
