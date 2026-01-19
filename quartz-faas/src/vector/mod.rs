//! Vector search module for QuartzDB FaaS
//!
//! Provides HNSW-based approximate nearest neighbor search optimized for WASM.
//!
//! # Performance Optimizations
//!
//! - **SIMD**: 4x faster distance calculations using WASM128
//! - **Cache-Friendly**: Sequential memory access patterns
//! - **Adaptive Search**: Early exit when good results found

pub mod hnsw;
pub mod simd;

pub use hnsw::{DistanceMetric, HnswConfig, HnswIndex, SearchResult, VectorEntry, IndexStats};
pub use simd::{euclidean_distance_simd, cosine_distance_simd, dot_product_distance_simd};
