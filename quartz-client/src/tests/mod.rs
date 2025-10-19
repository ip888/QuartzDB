use super::*;
use quartz_core::query::QueryType;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_client_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/validate"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "records": [],
            "total": 0,
            "execution_time": 0.0
        })))
        .mount(&mock_server)
        .await;

    let client = QuartzClient::new(mock_server.uri(), Duration::from_secs(1));
    let query = Query {
        query_type: QueryType::Select,
        collection: "test".to_string(),
        filter: None,
        limit: None,
        offset: None,
    };

    let result = client.query(query).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.total, 0);
    assert_eq!(result.records.len(), 0);
    assert_eq!(result.execution_time, 0.0);
}

#[tokio::test]
async fn test_client_batch_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/validate"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/batch"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                "records": [],
                "total": 0,
                "execution_time": 0.0
            }])),
        )
        .mount(&mock_server)
        .await;

    let client = QuartzClient::new(mock_server.uri(), Duration::from_secs(1));
    let queries = vec![Query {
        query_type: QueryType::Select,
        collection: "test".to_string(),
        filter: None,
        limit: None,
        offset: None,
    }];

    let result = client.batch_query(queries).await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].total, 0);
    assert_eq!(results[0].records.len(), 0);
    assert_eq!(results[0].execution_time, 0.0);
}
