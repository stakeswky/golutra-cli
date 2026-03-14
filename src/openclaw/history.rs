//! 会话持久化：执行历史记录与回放。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::agent::AgentId;

/// 单次执行记录。
#[derive(Clone, Debug)]
pub struct ExecutionRecord {
    pub plan_id: String,
    pub instruction: String,
    pub task_count: usize,
    pub agent_count: usize,
    pub assignments: Vec<(String, AgentId)>,
    pub task_results: HashMap<String, TaskResult>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
}

/// 单个任务的结果。
#[derive(Clone, Debug)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: AgentId,
    pub output: String,
    pub success: bool,
}

/// 执行历史管理器（内存存储，后续可接入 redb）。
pub struct ExecutionHistory {
    records: Mutex<Vec<ExecutionRecord>>,
}

impl ExecutionHistory {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    /// 记录一次执行开始。
    pub fn record_start(
        &self,
        plan_id: &str,
        instruction: &str,
        task_count: usize,
        agent_count: usize,
        assignments: Vec<(String, AgentId)>,
    ) {
        let record = ExecutionRecord {
            plan_id: plan_id.to_string(),
            instruction: instruction.to_string(),
            task_count,
            agent_count,
            assignments,
            task_results: HashMap::new(),
            started_at: now_ts(),
            finished_at: None,
        };
        if let Ok(mut records) = self.records.lock() {
            records.push(record);
        }
    }

    /// 记录任务结果。
    pub fn record_task_result(&self, plan_id: &str, result: TaskResult) {
        if let Ok(mut records) = self.records.lock() {
            if let Some(record) = records.iter_mut().rev().find(|r| r.plan_id == plan_id) {
                record
                    .task_results
                    .insert(result.task_id.clone(), result);
            }
        }
    }

    /// 标记执行完成。
    pub fn record_finish(&self, plan_id: &str) {
        if let Ok(mut records) = self.records.lock() {
            if let Some(record) = records.iter_mut().rev().find(|r| r.plan_id == plan_id) {
                record.finished_at = Some(now_ts());
            }
        }
    }

    /// 获取最近 N 次执行记录。
    pub fn list_recent(&self, limit: usize) -> Vec<ExecutionRecord> {
        match self.records.lock() {
            Ok(records) => records.iter().rev().take(limit).cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 获取指定执行记录。
    pub fn get(&self, plan_id: &str) -> Option<ExecutionRecord> {
        match self.records.lock() {
            Ok(records) => records.iter().find(|r| r.plan_id == plan_id).cloned(),
            Err(_) => None,
        }
    }
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
