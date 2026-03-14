//! 语义索引：基于关键词的记忆检索（轻量级，无外部向量库依赖）。

use std::collections::HashMap;

use super::store::{MemoryRecord, MemoryStore};
use crate::contracts::agent::MemoryScope;

/// 记忆索引器：提供基于关键词的语义检索能力。
pub struct MemoryIndex {
    /// 倒排索引：keyword → Vec<(scope_prefix, owner, key)>
    inverted: HashMap<String, Vec<String>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            inverted: HashMap::new(),
        }
    }

    /// 对记忆记录建立索引。
    pub fn index_record(&mut self, record: &MemoryRecord) {
        let doc_id = format!("{}:{}:{}", scope_tag(&record.scope), record.owner, record.key);
        let tokens = tokenize(&record.key, &record.value);
        for token in tokens {
            self.inverted
                .entry(token)
                .or_default()
                .push(doc_id.clone());
        }
    }

    /// 从索引中移除记录。
    pub fn remove_record(&mut self, record: &MemoryRecord) {
        let doc_id = format!("{}:{}:{}", scope_tag(&record.scope), record.owner, record.key);
        for entries in self.inverted.values_mut() {
            entries.retain(|id| id != &doc_id);
        }
    }

    /// 关键词搜索，返回匹配的存储键列表（按匹配度降序）。
    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let query_tokens = tokenize(query, "");
        let mut scores: HashMap<String, usize> = HashMap::new();

        for token in &query_tokens {
            if let Some(doc_ids) = self.inverted.get(token) {
                for doc_id in doc_ids {
                    *scores.entry(doc_id.clone()).or_default() += 1;
                }
            }
        }

        let mut ranked: Vec<(String, usize)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.into_iter().take(limit).map(|(id, _)| id).collect()
    }

    /// 从 MemoryStore 重建索引。
    pub fn rebuild_from_store(&mut self, store: &MemoryStore) -> Result<(), String> {
        self.inverted.clear();
        for scope in &[MemoryScope::Private, MemoryScope::Task, MemoryScope::Global] {
            let records = store.list_by_scope(scope)?;
            for record in &records {
                self.index_record(record);
            }
        }
        Ok(())
    }
}

impl Default for MemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn scope_tag(scope: &MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Private => "priv",
        MemoryScope::Task => "task",
        MemoryScope::Global => "glob",
    }
}

/// 简单分词：按空白和标点拆分，转小写，过滤短词。
fn tokenize(key: &str, value: &str) -> Vec<String> {
    let combined = format!("{} {}", key, value);
    combined
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|s| s.to_lowercase())
        .filter(|s| s.len() >= 2)
        .collect()
}
