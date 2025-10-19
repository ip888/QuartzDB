use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Array(Box<DataType>),
    Object,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub fields: HashMap<String, DataType>,
    pub indexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub data_type: DataType,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub values: HashMap<String, Value>,
    pub timestamp: i64,
}