//! Shell 适配器：通用 shell 命令执行。

use crate::contracts::agent::{AgentCapabilities, AgentConfig, AgentMessage, AgentStatus};
use crate::agent_runtime::interface::{AgentHandle, AgentInterface};

use super::pty_common::{self, PtySpawnOpts};

pub(crate) struct ShellAdapter {
    capabilities: AgentCapabilities,
}

impl ShellAdapter {
    pub(crate) fn new() -> Self {
        Self {
            capabilities: AgentCapabilities {
                languages: vec!["bash".into(), "zsh".into(), "sh".into()],
                skills: vec!["execute".into(), "script".into(), "devops".into()],
                long_running: false,
                max_concurrent_tasks: 4,
            },
        }
    }
}

impl AgentInterface for ShellAdapter {
    fn id(&self) -> &str { "shell" }
    fn capabilities(&self) -> &AgentCapabilities { &self.capabilities }

    fn spawn(&self, config: &AgentConfig) -> Result<AgentHandle, String> {
        let program = config.command.as_deref().unwrap_or("sh").to_string();
        pty_common::spawn_pty_agent(config, PtySpawnOpts {
            program,
            args: Vec::new(),
            cwd: config.cwd.clone(),
            cols: 120,
            rows: 40,
        })
    }

    fn send(&self, handle: &AgentHandle, message: &AgentMessage) -> Result<(), String> {
        let text = match message {
            AgentMessage::Task { instruction, .. } => instruction.clone(),
            AgentMessage::Broadcast { content, .. } => content.clone(),
            _ => serde_json::to_string(message).map_err(|e| e.to_string())?,
        };
        pty_common::write_to_pty(handle, &text)
    }

    fn status(&self, handle: &AgentHandle) -> AgentStatus {
        if handle.alive.load(std::sync::atomic::Ordering::Relaxed) {
            AgentStatus::Idle
        } else {
            AgentStatus::Stopped
        }
    }

    fn shutdown(&self, handle: &AgentHandle) -> Result<(), String> {
        log::info!("shell adapter shutdown: id={}", handle.id);
        let _ = pty_common::write_to_pty(handle, "exit");
        pty_common::kill_pty(handle)
    }

    fn tool_type(&self) -> &str { "shell" }
}
