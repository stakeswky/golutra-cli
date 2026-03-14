//! Agent 生命周期管理：spawn / monitor / restart / kill。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::contracts::agent::{AgentConfig, AgentId, AgentMessage, AgentStatus};

use super::interface::{AgentHandle, AgentInterface};

/// 活跃 Agent 条目。
struct LiveAgent {
    config: AgentConfig,
    handle: AgentHandle,
    adapter: Arc<dyn AgentInterface>,
    restart_count: u32,
}

/// Agent 生命周期管理器：统一管理所有活跃 Agent 的创建、监控与销毁。
pub struct AgentLifecycle {
    agents: Mutex<HashMap<AgentId, LiveAgent>>,
    adapters: Mutex<HashMap<String, Arc<dyn AgentInterface>>>,
    max_restarts: u32,
}

impl AgentLifecycle {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
            adapters: Mutex::new(HashMap::new()),
            max_restarts: 3,
        }
    }

    /// 注册工具适配器。
    pub fn register_adapter(&self, adapter: Arc<dyn AgentInterface>) {
        let tool_type = adapter.tool_type().to_string();
        if let Ok(mut adapters) = self.adapters.lock() {
            adapters.insert(tool_type, adapter);
        }
    }

    /// 创建并启动 Agent。
    pub fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, String> {
        let adapter = {
            let adapters = self.adapters.lock().map_err(|e| e.to_string())?;
            adapters
                .get(&config.tool_type)
                .cloned()
                .ok_or_else(|| format!("no adapter for tool_type: {}", config.tool_type))?
        };

        let handle = adapter.spawn(&config)?;
        let agent_id = config.id.clone();

        let live = LiveAgent {
            config,
            handle,
            adapter,
            restart_count: 0,
        };

        let mut agents = self.agents.lock().map_err(|e| e.to_string())?;
        agents.insert(agent_id.clone(), live);
        Ok(agent_id)
    }

    /// 向指定 Agent 发送消息。
    pub fn send_message(&self, agent_id: &str, message: &AgentMessage) -> Result<(), String> {
        let agents = self.agents.lock().map_err(|e| e.to_string())?;
        let live = agents
            .get(agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;
        live.adapter.send(&live.handle, message)
    }

    /// 查询 Agent 状态。
    pub fn agent_status(&self, agent_id: &str) -> Result<AgentStatus, String> {
        let agents = self.agents.lock().map_err(|e| e.to_string())?;
        let live = agents
            .get(agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;
        Ok(live.adapter.status(&live.handle))
    }

    /// 列出所有活跃 Agent。
    pub fn list_agents(&self) -> Vec<(AgentId, String, AgentStatus)> {
        let agents = match self.agents.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        agents
            .iter()
            .map(|(id, live)| {
                let status = live.adapter.status(&live.handle);
                (id.clone(), live.config.role.clone(), status)
            })
            .collect()
    }

    /// 终止指定 Agent。
    pub fn kill_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| e.to_string())?;
        let live = agents
            .remove(agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;
        live.adapter.shutdown(&live.handle)
    }

    /// 尝试重启异常 Agent。
    pub fn restart_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| e.to_string())?;
        let live = agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;

        if live.restart_count >= self.max_restarts {
            return Err(format!(
                "agent {agent_id} exceeded max restarts ({})",
                self.max_restarts
            ));
        }

        let _ = live.adapter.shutdown(&live.handle);
        let new_handle = live.adapter.spawn(&live.config)?;
        live.handle = new_handle;
        live.restart_count += 1;
        Ok(())
    }

    /// 轮询指定 Agent 的输出（非阻塞）。
    pub fn poll_agent_output(&self, agent_id: &str) -> Vec<AgentMessage> {
        let agents = match self.agents.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        let live = match agents.get(agent_id) {
            Some(l) => l,
            None => return Vec::new(),
        };
        let rx = match live.handle.receiver.lock() {
            Ok(rx) => rx,
            Err(_) => return Vec::new(),
        };
        let mut messages = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }
        messages
    }

    /// 轮询所有活跃 Agent 的输出（非阻塞）。
    pub fn poll_all_outputs(&self) -> Vec<(AgentId, AgentMessage)> {
        let agents = match self.agents.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        let mut all = Vec::new();
        for (id, live) in agents.iter() {
            if let Ok(rx) = live.handle.receiver.lock() {
                while let Ok(msg) = rx.try_recv() {
                    all.push((id.clone(), msg));
                }
            }
        }
        all
    }

    /// 终止所有 Agent。
    pub fn shutdown_all(&self) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| e.to_string())?;
        let mut errors = Vec::new();
        for (id, live) in agents.drain() {
            if let Err(err) = live.adapter.shutdown(&live.handle) {
                errors.push(format!("{id}: {err}"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

impl Default for AgentLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
