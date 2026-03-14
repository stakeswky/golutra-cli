//! 任务执行器：分发任务、轮询 Agent 输出、追踪任务状态。

use std::collections::HashMap;
use std::time::Instant;

use crate::agent_runtime::AgentLifecycle;
use crate::contracts::agent::{
    AgentId, AgentMessage, ContextItem, MemoryScope,
};
use crate::memory::context::ContextBuilder;
use crate::memory::SharedMemory;

/// 任务状态。
#[derive(Clone, Debug)]
pub enum TaskState {
    /// 等待执行。
    Pending,
    /// 正在执行。
    Running {
        agent_id: AgentId,
        started_at: Instant,
    },
    /// 执行完成。
    Done {
        agent_id: AgentId,
        output: String,
    },
    /// 执行失败。
    Failed {
        agent_id: AgentId,
        error: String,
    },
}

/// 任务执行器。
pub struct TaskExecutor {
    /// 任务状态追踪。
    task_states: HashMap<String, TaskState>,
    /// 每个任务累积的输出。
    task_outputs: HashMap<String, String>,
    /// agent_id → task_id 的反向映射。
    agent_task_map: HashMap<AgentId, String>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            task_states: HashMap::new(),
            task_outputs: HashMap::new(),
            agent_task_map: HashMap::new(),
        }
    }

    /// 分发任务到指定 Agent，注入上下文。
    pub fn dispatch(
        &mut self,
        lifecycle: &AgentLifecycle,
        agent_id: &str,
        task_id: &str,
        instruction: &str,
        context: Vec<ContextItem>,
    ) -> Result<(), String> {
        let message = AgentMessage::Task {
            id: task_id.to_string(),
            instruction: instruction.to_string(),
            context,
        };
        lifecycle.send_message(agent_id, &message)?;

        self.task_states.insert(
            task_id.to_string(),
            TaskState::Running {
                agent_id: agent_id.to_string(),
                started_at: Instant::now(),
            },
        );
        self.task_outputs
            .insert(task_id.to_string(), String::new());
        self.agent_task_map
            .insert(agent_id.to_string(), task_id.to_string());

        Ok(())
    }

    /// 分发任务并自动注入上下文（从记忆层检索）。
    pub fn dispatch_with_context(
        &mut self,
        lifecycle: &AgentLifecycle,
        agent_id: &str,
        task_id: &str,
        instruction: &str,
        memory: &SharedMemory,
        predecessor_outputs: &HashMap<String, String>,
    ) -> Result<(), String> {
        let mut builder = ContextBuilder::new(memory, 8000)
            .add_instruction(instruction)
            .add_memory_search(instruction, 5);

        // 注入前置任务输出
        for (pred_id, output) in predecessor_outputs {
            builder = builder.add_agent_output(pred_id, output);
        }

        let context = builder.build();
        self.dispatch(lifecycle, agent_id, task_id, instruction, context)
    }

    /// 轮询所有活跃 Agent 的输出，返回 (agent_id, task_id, output_chunk) 列表。
    pub fn poll_outputs(
        &mut self,
        lifecycle: &AgentLifecycle,
    ) -> Vec<(AgentId, String, String)> {
        let all_outputs = lifecycle.poll_all_outputs();
        let mut results = Vec::new();

        for (agent_id, message) in all_outputs {
            let task_id = self
                .agent_task_map
                .get(&agent_id)
                .cloned()
                .unwrap_or_default();

            match &message {
                AgentMessage::Result {
                    output, ..
                } => {
                    // 累积输出
                    if let Some(buf) = self.task_outputs.get_mut(&task_id) {
                        buf.push_str(output);
                    }
                    results.push((agent_id, task_id, output.clone()));
                }
                AgentMessage::Error {
                    message: err_msg, ..
                } => {
                    self.task_states.insert(
                        task_id.clone(),
                        TaskState::Failed {
                            agent_id: agent_id.clone(),
                            error: err_msg.clone(),
                        },
                    );
                    results.push((agent_id, task_id, format!("[ERROR] {err_msg}")));
                }
                AgentMessage::Progress {
                    detail, percent, ..
                } => {
                    results.push((
                        agent_id,
                        task_id,
                        format!("[PROGRESS {percent:.0}%] {detail}"),
                    ));
                }
                _ => {}
            }
        }

        results
    }

    /// 标记任务完成，返回累积输出。
    pub fn complete_task(&mut self, task_id: &str) -> Option<String> {
        let output = self.task_outputs.get(task_id).cloned().unwrap_or_default();
        if let Some(state) = self.task_states.get(task_id) {
            if let TaskState::Running { agent_id, .. } = state {
                let aid = agent_id.clone();
                self.task_states.insert(
                    task_id.to_string(),
                    TaskState::Done {
                        agent_id: aid.clone(),
                        output: output.clone(),
                    },
                );
                self.agent_task_map.remove(&aid);
            }
        }
        Some(output)
    }

    /// 将任务输出写入共享记忆。
    pub fn remember_output(
        &self,
        memory: &SharedMemory,
        task_id: &str,
        agent_id: &str,
    ) {
        if let Some(output) = self.task_outputs.get(task_id) {
            if !output.is_empty() {
                let key = format!("task_output:{task_id}");
                let _ = memory.remember(
                    &agent_id.to_string(),
                    MemoryScope::Task,
                    key,
                    output.clone(),
                );
            }
        }
    }

    /// 获取任务状态。
    pub fn task_state(&self, task_id: &str) -> Option<&TaskState> {
        self.task_states.get(task_id)
    }

    /// 获取所有任务状态。
    pub fn all_task_states(&self) -> &HashMap<String, TaskState> {
        &self.task_states
    }

    /// 获取指定 Agent 的最近输出。
    pub fn agent_output(&self, agent_id: &str) -> Option<&String> {
        let task_id = self.agent_task_map.get(agent_id)?;
        self.task_outputs.get(task_id)
    }

    /// 获取已完成任务的输出（供 DAG 后继任务注入上下文）。
    pub fn completed_outputs(&self) -> HashMap<String, String> {
        let mut out = HashMap::new();
        for (tid, state) in &self.task_states {
            if let TaskState::Done { output, .. } = state {
                out.insert(tid.clone(), output.clone());
            }
        }
        out
    }

    /// 是否有活跃任务。
    pub fn has_active_tasks(&self) -> bool {
        self.task_states
            .values()
            .any(|s| matches!(s, TaskState::Running { .. }))
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}
