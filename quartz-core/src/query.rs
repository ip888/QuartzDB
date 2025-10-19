use crate::{Result, types::Record};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query_type: QueryType,
    pub collection: String,
    pub filter: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub records: Vec<Record>,
    pub total: usize,
    pub execution_time: f64,
}

#[async_trait]
pub trait QueryExecutor {
    async fn execute(&self, query: Query) -> Result<QueryResult>;
    async fn prepare(&self, query: Query) -> Result<()>;
    async fn explain(&self, query: Query) -> Result<String>;
}
