//! Network layer implementation for QuartzDB
//!
//! This module handles all network communication between nodes,
//! including cluster coordination and data transfer.

use async_trait::async_trait;
use quartz_core::Result;

#[async_trait]
pub trait NetworkTransport {
    async fn connect(&self, addr: &str) -> Result<()>;
    async fn send(&self, data: Vec<u8>) -> Result<()>;
    async fn receive(&self) -> Result<Vec<u8>>;
}

pub struct Node {
    id: String,
    addr: String,
}

impl Node {
    pub fn new(id: String, addr: String) -> Self {
        Self { id, addr }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_address(&self) -> &str {
        &self.addr
    }

    pub fn update_address(&mut self, new_addr: String) {
        self.addr = new_addr;
    }
}
