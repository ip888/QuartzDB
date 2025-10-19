use quartz_core::types::*;

#[tokio::test]
async fn test_data_types() {
    let schema = Schema {
        fields: [
            ("id".to_string(), DataType::String),
            ("age".to_string(), DataType::Integer),
            ("name".to_string(), DataType::String),
        ]
        .into_iter()
        .collect(),
        indexes: vec!["id".to_string()],
    };

    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.indexes.len(), 1);
}

#[tokio::test]
async fn test_record_creation() {
    let record = Record {
        id: "test1".to_string(),
        values: [(
            "name".to_string(),
            Value {
                data_type: DataType::String,
                value: b"John Doe".to_vec(),
            },
        )]
        .into_iter()
        .collect(),
        timestamp: 1234567890,
    };

    assert_eq!(record.id, "test1");
    assert_eq!(record.values.len(), 1);
}
