//! OpenCode 适配器。

use crate::contracts::agent::{AgentCapabilities, AgentConfig, AgentMessage, AgentStatus};
use crate::agent_runtime::interface::{AgentHandle, AgentInterface};

use super::pty_common::{self, PtySpawnOpts};

pub(crate) struct OpenCodeAdapter {
    capabilities: AgentCapabilities,
}

impl OpenCodeAdapter {
    pub(crate) fn new() -> Self {
        Self {
            capabilities: AgentCapabilities {
                languages: vec!["go".into(), "rust".into(), "python".into()],
                skills: vec!["code_generation".into(), "explain".into()],
                long_running: true,
                max_concurrent_tasks: 1,
            },
        }
    }
}

impl AgentInterface for OpenCodeAdapter {
    fn id(&self) -> &str { "opencode" }
    fn capabilities(&self) -> &AgentCapabilities { &self.capabilities }

    fn spawn(&self, config: &AgentConfig) -> Result<AgentHandle, String> {
        let program = config.command.as_deref().unwrap_or("opencode").to_string();
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
        log::info!("opencode adapter shutdown: id={}", handle.id);
        pty_common::kill_pty(handle)
    }

    fn tool_type(&self) -> &str { "opencode" }
}
