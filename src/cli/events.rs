//! CLI 事件端口实现：替代 Tauri WebView 的事件分发。

use crate::contracts::agent::{AgentId, AgentStatus};

/// CLI 事件发射器：将 Agent 事件输出到终端。
pub struct CliEventEmitter;

impl CliEventEmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn agent_spawned(&self, agent_id: &AgentId, role: &str) {
        eprintln!("[+] Agent 启动: {} ({})", agent_id, role);
    }

    pub fn agent_status_changed(&self, agent_id: &AgentId, status: &AgentStatus) {
        let tag = match status {
            AgentStatus::Idle => "空闲",
            AgentStatus::Busy { task_id } => {
                eprintln!("[~] {} 执行中: {}", agent_id, task_id);
                return;
            }
            AgentStatus::Error { message } => {
                eprintln!("[!] {} 异常: {}", agent_id, message);
                return;
            }
            AgentStatus::Stopped => "已停止",
            AgentStatus::Pending => "等待中",
            AgentStatus::Starting => "启动中",
        };
        eprintln!("[·] {} → {}", agent_id, tag);
    }

    pub fn task_assigned(&self, task_id: &str, agent_id: &AgentId) {
        eprintln!("[→] 任务 {} → {}", task_id, agent_id);
    }

    pub fn task_completed(&self, task_id: &str, agent_id: &AgentId) {
        eprintln!("[✓] 任务 {} 完成 ({})", task_id, agent_id);
    }

    pub fn plan_created(&self, plan_id: &str, task_count: usize, agent_count: usize) {
        eprintln!(
            "[▶] 执行计划 {} — {} 个任务, {} 个 Agent",
            plan_id, task_count, agent_count
        );
    }

    pub fn channel_created(&self, channel_id: &str, member_count: usize) {
        eprintln!("[#] 频道 {} ({} 成员)", channel_id, member_count);
    }

    pub fn error(&self, msg: &str) {
        eprintln!("[✗] {}", msg);
    }

    pub fn info(&self, msg: &str) {
        eprintln!("[i] {}", msg);
    }
}

impl Default for CliEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
