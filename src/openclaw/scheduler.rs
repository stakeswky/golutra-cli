//! DAG 并行调度器：支持无依赖任务并行执行，有依赖的按拓扑序执行。

use std::collections::{HashMap, HashSet};

use crate::contracts::agent::ExecutionPlan;

/// DAG 调度器。
pub struct DagScheduler {
    /// 任务 ID 列表。
    task_ids: Vec<String>,
    /// 入度表：task_id → 未完成的前置依赖数。
    in_degree: HashMap<String, usize>,
    /// 后继表：task_id → 依赖它的任务列表。
    successors: HashMap<String, Vec<String>>,
    /// 已完成的任务。
    completed: HashSet<String>,
    /// 已分发（正在执行）的任务。
    dispatched: HashSet<String>,
}

impl DagScheduler {
    /// 从执行计划构建调度器。
    pub fn new(plan: &ExecutionPlan) -> Self {
        let task_ids: Vec<String> = plan.tasks.iter().map(|t| t.id.clone()).collect();
        let mut in_degree: HashMap<String, usize> = task_ids.iter().map(|id| (id.clone(), 0)).collect();
        let mut successors: HashMap<String, Vec<String>> = task_ids.iter().map(|id| (id.clone(), Vec::new())).collect();

        // dependencies 格式: (task_id, depends_on_task_id)
        for (task_id, dep_id) in &plan.dependencies {
            *in_degree.entry(task_id.clone()).or_insert(0) += 1;
            successors
                .entry(dep_id.clone())
                .or_default()
                .push(task_id.clone());
        }

        Self {
            task_ids,
            in_degree,
            successors,
            completed: HashSet::new(),
            dispatched: HashSet::new(),
        }
    }

    /// 返回当前可执行的任务（入度为 0 且未分发/完成）。
    pub fn ready_tasks(&self) -> Vec<String> {
        self.task_ids
            .iter()
            .filter(|id| {
                !self.completed.contains(*id)
                    && !self.dispatched.contains(*id)
                    && self.in_degree.get(*id).copied().unwrap_or(0) == 0
            })
            .cloned()
            .collect()
    }

    /// 标记任务已分发。
    pub fn mark_dispatched(&mut self, task_id: &str) {
        self.dispatched.insert(task_id.to_string());
    }

    /// 标记任务完成，减少后继任务的入度。
    pub fn complete_task(&mut self, task_id: &str) {
        self.completed.insert(task_id.to_string());
        self.dispatched.remove(task_id);

        if let Some(succs) = self.successors.get(task_id) {
            for succ in succs.clone() {
                if let Some(deg) = self.in_degree.get_mut(&succ) {
                    *deg = deg.saturating_sub(1);
                }
            }
        }
    }

    /// 所有任务是否已完成。
    pub fn is_done(&self) -> bool {
        self.completed.len() == self.task_ids.len()
    }

    /// 是否有正在执行的任务。
    pub fn has_in_flight(&self) -> bool {
        !self.dispatched.is_empty()
    }

    /// 已完成任务数。
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// 总任务数。
    pub fn total_count(&self) -> usize {
        self.task_ids.len()
    }
}
