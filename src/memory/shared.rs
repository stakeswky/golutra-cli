//! 跨 Agent 共享记忆接口。

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::agent::{AgentId, MemoryScope};

use super::index::MemoryIndex;
use super::store::{MemoryRecord, MemoryStore};

/// 共享记忆层：提供跨 Agent 的记忆读写与检索。
pub struct SharedMemory {
    store: Arc<MemoryStore>,
    index: Mutex<MemoryIndex>,
}

impl SharedMemory {
    pub fn new(store: Arc<MemoryStore>) -> Result<Self, String> {
        let mut index = MemoryIndex::new();
        index.rebuild_from_store(&store)?;
        Ok(Self {
            store,
            index: Mutex::new(index),
        })
    }

    /// 写入记忆（自动更新索引）。
    pub fn remember(
        &self,
        owner: &AgentId,
        scope: MemoryScope,
        key: String,
        value: String,
    ) -> Result<(), String> {
        let now = now_ts();
        let existing = self.store.get(owner, &scope, &key)?;
        let record = MemoryRecord {
            key,
            value,
            scope,
            owner: owner.clone(),
            created_at: existing.as_ref().map_or(now, |r| r.created_at),
            updated_at: now,
            access_count: existing.as_ref().map_or(0, |r| r.access_count),
        };

        self.store.put(&record)?;
        if let Ok(mut idx) = self.index.lock() {
            if let Some(old) = &existing {
                idx.remove_record(old);
            }
            idx.index_record(&record);
        }
        Ok(())
    }

    /// 读取记忆（自动增加访问计数）。
    pub fn recall(
        &self,
        owner: &str,
        scope: &MemoryScope,
        key: &str,
    ) -> Result<Option<String>, String> {
        match self.store.get(owner, scope, key)? {
            Some(mut record) => {
                record.access_count += 1;
                record.updated_at = now_ts();
                let _ = self.store.put(&record);
                Ok(Some(record.value))
            }
            None => Ok(None),
        }
    }

    /// 语义搜索记忆。
    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        match self.index.lock() {
            Ok(idx) => idx.search(query, limit),
            Err(_) => Vec::new(),
        }
    }

    /// 列出指定作用域的所有记忆。
    pub fn list_scope(&self, scope: &MemoryScope) -> Result<Vec<MemoryRecord>, String> {
        self.store.list_by_scope(scope)
    }

    /// 删除记忆。
    pub fn forget(
        &self,
        owner: &str,
        scope: &MemoryScope,
        key: &str,
    ) -> Result<bool, String> {
        if let Ok(Some(record)) = self.store.get(owner, scope, key) {
            if let Ok(mut idx) = self.index.lock() {
                idx.remove_record(&record);
            }
        }
        self.store.delete(owner, scope, key)
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
