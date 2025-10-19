use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub operations: Vec<Operation>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        collection: String,
        data: Vec<u8>,
    },
    Update {
        collection: String,
        id: String,
        data: Vec<u8>,
    },
    Delete {
        collection: String,
        id: String,
    },
}

#[async_trait]
pub trait TransactionManager {
    async fn begin(&self) -> Result<Transaction>;
    async fn commit(&self, transaction: Transaction) -> Result<()>;
    async fn rollback(&self, transaction: Transaction) -> Result<()>;
}
