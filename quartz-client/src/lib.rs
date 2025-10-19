//! Client SDK for QuartzDB
//!
//! This module provides the client interface for interacting with QuartzDB.

use async_trait::async_trait;
use dashmap::DashMap;
use quartz_core::{
    Error, Result,
    query::{Query, QueryResult},
};
use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Semaphore, time::Instant};

mod metrics;
use metrics::ClientMetrics;

const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(100);
const DEFAULT_MAX_CONNECTIONS: usize = 32;
const DEFAULT_QUERY_VALIDATION_TIMEOUT: Duration = Duration::from_secs(5);

#[async_trait]
pub trait Client: Send + Sync {
    async fn connect(&self, url: &str) -> Result<()>;
    async fn query(&self, query: Query) -> Result<QueryResult>;
    async fn batch_query(&self, queries: Vec<Query>) -> Result<Vec<QueryResult>>;
    async fn health_check(&self) -> Result<()>;
    async fn validate_query(&self, query: &Query) -> Result<()>;
    async fn get_metrics(&self) -> ClientMetrics;
}

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub timeout: Duration,
    pub max_connections: usize,
    pub validation_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            retry_delay: DEFAULT_RETRY_DELAY,
            timeout: Duration::from_secs(30),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            validation_timeout: DEFAULT_QUERY_VALIDATION_TIMEOUT,
        }
    }
}

pub struct QuartzClient {
    endpoint: String,
    config: ClientConfig,
    client: ReqwestClient,
    connection_pool: Arc<Semaphore>,
    metrics: Arc<ClientMetrics>,
    query_cache: Arc<DashMap<String, (QueryResult, Instant)>>,
}

impl QuartzClient {
    pub fn builder(endpoint: String) -> QuartzClientBuilder {
        QuartzClientBuilder::new(endpoint)
    }

    pub fn new(endpoint: String, timeout: Duration) -> Self {
        Self::builder(endpoint).with_timeout(timeout).build()
    }

    async fn with_retry<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    if attempts < self.config.max_retries {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Network("Unknown error".to_string())))
    }
}

pub struct QuartzClientBuilder {
    endpoint: String,
    config: ClientConfig,
}

impl QuartzClientBuilder {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            config: ClientConfig::default(),
        }
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn with_retry_delay(mut self, retry_delay: Duration) -> Self {
        self.config.retry_delay = retry_delay;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    pub fn with_validation_timeout(mut self, timeout: Duration) -> Self {
        self.config.validation_timeout = timeout;
        self
    }

    pub fn build(self) -> QuartzClient {
        let config = self.config.clone();
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(config.max_connections)
            .build()
            .expect("Failed to build HTTP client");

        let config = self.config.clone();
        QuartzClient {
            endpoint: self.endpoint,
            config,
            client,
            connection_pool: Arc::new(Semaphore::new(self.config.max_connections)),
            metrics: Arc::new(ClientMetrics::new()),
            query_cache: Arc::new(DashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests;

#[async_trait]
impl Client for QuartzClient {
    async fn connect(&self, url: &str) -> Result<()> {
        self.with_retry(|| async {
            let start = Instant::now();
            let health_url = format!("{}/health", url);
            let _permit = self.connection_pool.acquire().await;

            self.metrics
                .record_connection_acquire_time(start.elapsed().as_millis() as u64);

            let response = self
                .client
                .get(&health_url)
                .send()
                .await
                .map_err(|e| Error::ConnectionError(e.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                self.metrics.record_failure();
                Err(Error::ConnectionError(
                    "Failed to connect to server".to_string(),
                ))
            }
        })
        .await
    }

    async fn query(&self, query: Query) -> Result<QueryResult> {
        self.validate_query(&query).await?;

        // Check cache first
        let cache_key = format!("{:?}", query.clone());
        if let Some(cached) = self.query_cache.get(&cache_key)
            && cached.1.elapsed() < self.config.validation_timeout
        {
            return Ok(cached.0.clone());
        }

        let query = Arc::new(query);
        let cache_key = Arc::new(cache_key);

        self.with_retry(|| {
            let query = Arc::clone(&query);
            let cache_key = Arc::clone(&cache_key);
            async move {
                let start = Instant::now();
                let query_url = format!("{}/query", self.endpoint);
                let _permit = self.connection_pool.acquire().await;

                self.metrics.record_request();
                self.metrics
                    .record_connection_acquire_time(start.elapsed().as_millis() as u64);

                let response = self
                    .client
                    .post(&query_url)
                    .json(&*query)
                    .send()
                    .await
                    .map_err(|e| Error::Network(e.to_string()))?;

                self.metrics
                    .record_query_execution_time(start.elapsed().as_millis() as u64);

                if response.status().is_success() {
                    let result = response
                        .json::<QueryResult>()
                        .await
                        .map_err(|e| Error::Network(e.to_string()))?;

                    // Cache the result
                    self.query_cache
                        .insert((*cache_key).clone(), (result.clone(), Instant::now()));

                    Ok(result)
                } else {
                    self.metrics.record_failure();
                    let error = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(Error::Query(error))
                }
            }
        })
        .await
    }

    async fn batch_query(&self, queries: Vec<Query>) -> Result<Vec<QueryResult>> {
        // Validate all queries first
        for query in &queries {
            self.validate_query(query).await?;
        }

        self.with_retry(|| async {
            let start = Instant::now();
            let batch_url = format!("{}/batch", self.endpoint);
            let _permit = self.connection_pool.acquire().await;

            self.metrics.record_request();
            self.metrics
                .record_connection_acquire_time(start.elapsed().as_millis() as u64);

            let response = self
                .client
                .post(&batch_url)
                .json(&queries)
                .send()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;

            self.metrics
                .record_query_execution_time(start.elapsed().as_millis() as u64);

            if response.status().is_success() {
                response
                    .json::<Vec<QueryResult>>()
                    .await
                    .map_err(|e| Error::Network(e.to_string()))
            } else {
                self.metrics.record_failure();
                let error = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(Error::Query(error))
            }
        })
        .await
    }

    async fn health_check(&self) -> Result<()> {
        self.with_retry(|| async {
            let start = Instant::now();
            let _permit = self.connection_pool.acquire().await;

            self.metrics.record_request();
            self.metrics
                .record_connection_acquire_time(start.elapsed().as_millis() as u64);

            let health_url = format!("{}/health", self.endpoint);
            let response = self
                .client
                .get(&health_url)
                .send()
                .await
                .map_err(|e| Error::ConnectionError(e.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                self.metrics.record_failure();
                Err(Error::ConnectionError("Health check failed".to_string()))
            }
        })
        .await
    }

    async fn validate_query(&self, query: &Query) -> Result<()> {
        let validate_url = format!("{}/validate", self.endpoint);
        let response = self
            .client
            .post(&validate_url)
            .json(query)
            .timeout(self.config.validation_timeout)
            .send()
            .await
            .map_err(|e| Error::Query(format!("Query validation failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown validation error".to_string());
            Err(Error::Query(format!("Query validation failed: {}", error)))
        }
    }

    async fn get_metrics(&self) -> ClientMetrics {
        let metrics = (*self.metrics).clone();
        metrics.set_active_connections(self.connection_pool.available_permits() as u64);
        metrics
    }
}
