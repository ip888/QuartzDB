use crate::lsm::LSMTree;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CompactionManager {
    lsm: Arc<LSMTree>,
    compaction_threshold: usize,
    is_compacting: Mutex<bool>,
}

impl CompactionManager {
    pub fn new(lsm: Arc<LSMTree>, compaction_threshold: usize) -> Self {
        Self {
            lsm,
            compaction_threshold,
            is_compacting: Mutex::new(false),
        }
    }

    pub async fn check_and_compact(&self) {
        let mut is_compacting = self.is_compacting.lock().await;
        if *is_compacting {
            return;
        }
        *is_compacting = true;

        // Check if compaction is needed
        if self.should_compact() {
            // Future implementation: trigger compaction process
            println!("Compaction triggered for {} levels", self.lsm.level_count());
        }

        *is_compacting = false;
    }

    fn should_compact(&self) -> bool {
        self.lsm.level_count() > self.compaction_threshold
    }
}
