use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LSMTree {
    levels: Vec<Arc<RwLock<Level>>>,
    max_level_size: usize,
}

struct Level {
    files: Vec<SSTable>,
    level: usize,
}

struct SSTable {
    file_id: u64,
    min_key: Vec<u8>,
    max_key: Vec<u8>,
}

impl SSTable {
    pub fn new(file_id: u64, min_key: Vec<u8>, max_key: Vec<u8>) -> Self {
        Self {
            file_id,
            min_key,
            max_key,
        }
    }

    pub fn contains_key(&self, key: &[u8]) -> bool {
        key >= self.min_key.as_slice() && key <= self.max_key.as_slice()
    }
}

impl Level {
    pub fn new(level: usize) -> Self {
        Self {
            files: Vec::new(),
            level,
        }
    }

    pub fn add_sstable(&mut self, sstable: SSTable) {
        self.files.push(sstable);
    }

    pub fn search(&self, key: &[u8]) -> Option<&SSTable> {
        // Search through files, prioritizing newer files in higher levels
        println!(
            "Searching level {} with {} files",
            self.level,
            self.files.len()
        );
        self.files.iter().find(|sstable| sstable.contains_key(key))
    }
}

impl LSMTree {
    pub fn new(max_level_size: usize) -> Self {
        Self {
            levels: vec![Arc::new(RwLock::new(Level::new(0)))],
            max_level_size,
        }
    }

    pub async fn add_level(&mut self) {
        let level_number = self.levels.len();
        self.levels
            .push(Arc::new(RwLock::new(Level::new(level_number))));
    }

    pub async fn search(&self, key: &[u8]) -> Option<u64> {
        for level in &self.levels {
            let level_guard = level.read().await;
            if let Some(sstable) = level_guard.search(key) {
                return Some(sstable.file_id);
            }
        }
        None
    }

    pub fn get_max_level_size(&self) -> usize {
        self.max_level_size
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Demo function to show how SSTable creation and addition works
    pub async fn demo_add_sstable(&mut self, level_idx: usize) -> bool {
        if level_idx < self.levels.len() {
            let sstable = SSTable::new(1, b"key1".to_vec(), b"key9".to_vec());
            let mut level = self.levels[level_idx].write().await;
            level.add_sstable(sstable);
            true
        } else {
            false
        }
    }
}
