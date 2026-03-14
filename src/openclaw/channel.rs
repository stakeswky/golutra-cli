//! 频道管理：任务频道的自动创建与生命周期管理。

use crate::contracts::agent::{AgentId, ChannelType};

use super::protocol::ProtocolRouter;

/// 频道管理器：封装频道的自动创建/销毁逻辑。
pub struct ChannelManager<'a> {
    router: &'a ProtocolRouter,
}

impl<'a> ChannelManager<'a> {
    pub fn new(router: &'a ProtocolRouter) -> Self {
        Self { router }
    }

    /// 为任务自动创建协作频道。
    pub fn create_task_channel(
        &self,
        task_id: &str,
        members: Vec<AgentId>,
    ) -> Result<String, String> {
        let channel_id = format!("ch-task-{task_id}");
        let name = format!("Task: {task_id}");
        self.router.create_channel(
            channel_id.clone(),
            name,
            members,
            ChannelType::Task {
                task_id: task_id.to_string(),
            },
        )?;
        Ok(channel_id)
    }

    /// 创建持久协作频道（跨任务）。
    pub fn create_persistent_channel(
        &self,
        name: &str,
        members: Vec<AgentId>,
    ) -> Result<String, String> {
        let channel_id = format!("ch-{}", name.to_lowercase().replace(' ', "-"));
        self.router.create_channel(
            channel_id.clone(),
            name.to_string(),
            members,
            ChannelType::Persistent,
        )?;
        Ok(channel_id)
    }

    /// 创建广播频道。
    pub fn create_broadcast_channel(
        &self,
        name: &str,
        members: Vec<AgentId>,
    ) -> Result<String, String> {
        let channel_id = format!("ch-bc-{}", name.to_lowercase().replace(' ', "-"));
        self.router.create_channel(
            channel_id.clone(),
            name.to_string(),
            members,
            ChannelType::Broadcast,
        )?;
        Ok(channel_id)
    }

    /// 清理已完成任务的频道。
    pub fn cleanup_task(&self, task_id: &str) -> Result<usize, String> {
        self.router.cleanup_task_channels(task_id)
    }
}
