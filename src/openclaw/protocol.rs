//! 统一通信协议：Agent 间标准化消息格式与路由。

use crate::contracts::agent::{AgentId, AgentMessage, Channel, ChannelType};
use std::collections::HashMap;
use std::sync::Mutex;

/// 协议路由器：管理频道与消息分发。
pub struct ProtocolRouter {
    channels: Mutex<HashMap<String, Channel>>,
}

impl ProtocolRouter {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
        }
    }

    /// 创建频道。
    pub fn create_channel(
        &self,
        id: String,
        name: String,
        members: Vec<AgentId>,
        channel_type: ChannelType,
    ) -> Result<Channel, String> {
        let channel = Channel {
            id: id.clone(),
            name,
            members,
            channel_type,
        };
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;
        channels.insert(id, channel.clone());
        Ok(channel)
    }

    /// 向频道添加成员。
    pub fn join_channel(&self, channel_id: &str, agent_id: AgentId) -> Result<(), String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;
        let channel = channels
            .get_mut(channel_id)
            .ok_or_else(|| format!("channel not found: {channel_id}"))?;
        if !channel.members.contains(&agent_id) {
            channel.members.push(agent_id);
        }
        Ok(())
    }

    /// 从频道移除成员。
    pub fn leave_channel(&self, channel_id: &str, agent_id: &str) -> Result<(), String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;
        let channel = channels
            .get_mut(channel_id)
            .ok_or_else(|| format!("channel not found: {channel_id}"))?;
        channel.members.retain(|id| id != agent_id);
        Ok(())
    }

    /// 解析消息的目标接收者列表。
    pub fn resolve_recipients(&self, message: &AgentMessage) -> Vec<AgentId> {
        match message {
            AgentMessage::Broadcast {
                channel_id,
                sender_id,
                ..
            } => {
                let channels = match self.channels.lock() {
                    Ok(c) => c,
                    Err(_) => return Vec::new(),
                };
                match channels.get(channel_id) {
                    Some(ch) => ch
                        .members
                        .iter()
                        .filter(|id| *id != sender_id)
                        .cloned()
                        .collect(),
                    None => Vec::new(),
                }
            }
            _ => Vec::new(),
        }
    }

    /// 删除频道。
    pub fn remove_channel(&self, channel_id: &str) -> Result<(), String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;
        channels.remove(channel_id);
        Ok(())
    }

    /// 列出所有频道。
    pub fn list_channels(&self) -> Vec<Channel> {
        match self.channels.lock() {
            Ok(channels) => channels.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 清理已完成任务关联的频道。
    pub fn cleanup_task_channels(&self, task_id: &str) -> Result<usize, String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;
        let to_remove: Vec<String> = channels
            .iter()
            .filter(|(_, ch)| matches!(&ch.channel_type, ChannelType::Task { task_id: tid } if tid == task_id))
            .map(|(id, _)| id.clone())
            .collect();
        let count = to_remove.len();
        for id in to_remove {
            channels.remove(&id);
        }
        Ok(count)
    }
}

impl Default for ProtocolRouter {
    fn default() -> Self {
        Self::new()
    }
}
