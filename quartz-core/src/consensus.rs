use async_trait::async_trait;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub address: String,
    pub is_leader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub term: u64,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait ConsensusProtocol {
    async fn initialize(&self) -> Result<()>;
    async fn propose(&self, data: Vec<u8>) -> Result<()>;
    async fn get_leader(&self) -> Result<Node>;
    async fn get_cluster_state(&self) -> Result<HashMap<String, Node>>;
}