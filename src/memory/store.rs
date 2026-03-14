//! 记忆持久化：基于 redb 的键值存储。

use std::path::PathBuf;
use std::sync::Mutex;

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::contracts::agent::{AgentId, MemoryScope};

const MEMORY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("agent_memory");

/// 单条记忆记录。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub key: String,
    pub value: String,
    pub scope: MemoryScope,
    pub owner: AgentId,
    pub created_at: u64,
    pub updated_at: u64,
    pub access_count: u64,
}

/// 记忆存储引擎。
pub struct MemoryStore {
    db: Mutex<Database>,
}

impl MemoryStore {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let db = Database::create(path).map_err(|e| format!("memory db open: {e}"))?;
        {
            let tx = db.begin_write().map_err(|e| e.to_string())?;
            let _ = tx.open_table(MEMORY_TABLE);
            tx.commit().map_err(|e| e.to_string())?;
        }
        Ok(Self { db: Mutex::new(db) })
    }

    /// 写入或更新记忆。
    pub fn put(&self, record: &MemoryRecord) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let tx = db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = tx.open_table(MEMORY_TABLE).map_err(|e| e.to_string())?;
            let storage_key = Self::storage_key(&record.owner, &record.scope, &record.key);
            let bytes = serde_json::to_vec(record).map_err(|e| e.to_string())?;
            table.insert(storage_key.as_str(), bytes.as_slice()).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// 读取记忆。
    pub fn get(&self, owner: &str, scope: &MemoryScope, key: &str) -> Result<Option<MemoryRecord>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let tx = db.begin_read().map_err(|e| e.to_string())?;
        let table = tx.open_table(MEMORY_TABLE).map_err(|e| e.to_string())?;
        let storage_key = Self::storage_key(owner, scope, key);
        match table.get(storage_key.as_str()).map_err(|e| e.to_string())? {
            Some(bytes) => {
                let record: MemoryRecord = serde_json::from_slice(bytes.value()).map_err(|e| e.to_string())?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    /// 按作用域查询所有记忆。
    pub fn list_by_scope(&self, scope: &MemoryScope) -> Result<Vec<MemoryRecord>, String> {
        let prefix = Self::scope_prefix(scope);
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let tx = db.begin_read().map_err(|e| e.to_string())?;
        let table = tx.open_table(MEMORY_TABLE).map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        let iter = table.iter().map_err(|e| e.to_string())?;
        for entry in iter {
            let (k, v) = entry.map_err(|e| e.to_string())?;
            if k.value().starts_with(&prefix) {
                let record: MemoryRecord = serde_json::from_slice(v.value()).map_err(|e| e.to_string())?;
                results.push(record);
            }
        }
        Ok(results)
    }

    /// 删除记忆。
    pub fn delete(&self, owner: &str, scope: &MemoryScope, key: &str) -> Result<bool, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let tx = db.begin_write().map_err(|e| e.to_string())?;
        let removed = {
            let mut table = tx.open_table(MEMORY_TABLE).map_err(|e| e.to_string())?;
            let storage_key = Self::storage_key(owner, scope, key);
            let result = table.remove(storage_key.as_str()).map_err(|e| e.to_string())?;
            result.is_some()
        };
        tx.commit().map_err(|e| e.to_string())?;
        Ok(removed)
    }

    fn storage_key(owner: &str, scope: &MemoryScope, key: &str) -> String {
        format!("{}:{}:{}", Self::scope_prefix(scope), owner, key)
    }

    fn scope_prefix(scope: &MemoryScope) -> String {
        match scope {
            MemoryScope::Private => "priv".to_string(),
            MemoryScope::Task => "task".to_string(),
            MemoryScope::Global => "glob".to_string(),
        }
    }
}
