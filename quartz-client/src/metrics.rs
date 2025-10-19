use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct ClientMetrics {
    requests_total: Arc<AtomicU64>,
    requests_failed: Arc<AtomicU64>,
    connections_active: Arc<AtomicU64>,
    connection_acquire_time: Arc<AtomicU64>,
    query_execution_time: Arc<AtomicU64>,
}

impl Default for ClientMetrics {
    fn default() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_failed: Arc::new(AtomicU64::new(0)),
            connections_active: Arc::new(AtomicU64::new(0)),
            connection_acquire_time: Arc::new(AtomicU64::new(0)),
            query_execution_time: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl ClientMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_active_connections(&self, count: u64) {
        self.connections_active.store(count, Ordering::Relaxed);
    }

    pub fn record_connection_acquire_time(&self, duration_ms: u64) {
        self.connection_acquire_time
            .store(duration_ms, Ordering::Relaxed);
    }

    pub fn record_query_execution_time(&self, duration_ms: u64) {
        self.query_execution_time
            .store(duration_ms, Ordering::Relaxed);
    }

    pub fn get_total_requests(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    pub fn get_failed_requests(&self) -> u64 {
        self.requests_failed.load(Ordering::Relaxed)
    }

    pub fn get_active_connections(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }

    pub fn get_connection_acquire_time(&self) -> u64 {
        self.connection_acquire_time.load(Ordering::Relaxed)
    }

    pub fn get_query_execution_time(&self) -> u64 {
        self.query_execution_time.load(Ordering::Relaxed)
    }
}
