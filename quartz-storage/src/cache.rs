use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CacheManager {
    cache: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    max_size: usize,
}

impl CacheManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.cache.read().await.get(key).cloned()
    }

    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) {
        let mut cache = self.cache.write().await;
        if cache.len() >= self.max_size
            && let Some((k, _)) = cache.iter().next()
        {
            let key_to_remove = k.clone();
            cache.remove(&key_to_remove);
        }
        cache.insert(key, value);
    }

    pub async fn remove(&self, key: &[u8]) {
        self.cache.write().await.remove(key);
    }
}
