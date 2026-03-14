//! 上下文窗口管理：为 Agent 组装任务上下文。

use crate::contracts::agent::{ContextItem, MemoryScope};

use super::shared::SharedMemory;

/// 上下文构建器：根据任务需求从记忆层提取相关上下文。
pub struct ContextBuilder<'a> {
    memory: &'a SharedMemory,
    items: Vec<ContextItem>,
    max_tokens: usize,
    current_tokens: usize,
}

impl<'a> ContextBuilder<'a> {
    pub fn new(memory: &'a SharedMemory, max_tokens: usize) -> Self {
        Self {
            memory,
            items: Vec::new(),
            max_tokens,
            current_tokens: 0,
        }
    }

    /// 添加指令上下文。
    pub fn add_instruction(mut self, instruction: &str) -> Self {
        self.push_item("instruction", instruction, None);
        self
    }

    /// 从记忆中按关键词检索并注入上下文。
    pub fn add_memory_search(mut self, query: &str, limit: usize) -> Self {
        let keys = self.memory.search(query, limit);
        for key in keys {
            // key 格式: scope:owner:actual_key
            let parts: Vec<&str> = key.splitn(3, ':').collect();
            if parts.len() == 3 {
                let scope = match parts[0] {
                    "priv" => MemoryScope::Private,
                    "task" => MemoryScope::Task,
                    _ => MemoryScope::Global,
                };
                if let Ok(Some(value)) = self.memory.recall(parts[1], &scope, parts[2]) {
                    self.push_item("memory", &value, Some(key));
                }
            }
        }
        self
    }

    /// 添加文件内容上下文。
    pub fn add_file(mut self, path: &str, content: &str) -> Self {
        self.push_item("file", content, Some(path.to_string()));
        self
    }

    /// 添加其他 Agent 的输出作为上下文。
    pub fn add_agent_output(mut self, agent_id: &str, output: &str) -> Self {
        self.push_item("output", output, Some(agent_id.to_string()));
        self
    }

    /// 构建最终上下文列表。
    pub fn build(self) -> Vec<ContextItem> {
        self.items
    }

    fn push_item(&mut self, kind: &str, content: &str, source: Option<String>) {
        let token_estimate = content.len() / 4; // 粗略估算
        if self.current_tokens + token_estimate > self.max_tokens {
            return;
        }
        self.current_tokens += token_estimate;
        self.items.push(ContextItem {
            kind: kind.to_string(),
            content: content.to_string(),
            source,
        });
    }
}
