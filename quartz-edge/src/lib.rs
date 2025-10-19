//! Edge computing components for QuartzDB
//!
//! This module implements edge caching and computation capabilities.

use async_trait::async_trait;
use quartz_core::Result;

#[async_trait]
pub trait EdgeNode {
    async fn init(&self) -> Result<()>;
    async fn cache_data(&self, key: &str, data: Vec<u8>) -> Result<()>;
    async fn get_cached(&self, key: &str) -> Result<Option<Vec<u8>>>;
}

pub struct EdgeManager {
    node_id: String,
    cache_size: usize,
}

impl EdgeManager {
    pub fn new(node_id: String, cache_size: usize) -> Self {
        Self {
            node_id,
            cache_size,
        }
    }

    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache_size
    }

    pub fn resize_cache(&mut self, new_size: usize) {
        self.cache_size = new_size;
    }
}
