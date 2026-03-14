//! 文档撰写模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "writer".to_string(),
        role: "文档撰写".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位技术文档撰写专家。你的职责是：\n",
            "1. 编写清晰、准确的技术文档\n",
            "2. 生成 API 文档、用户指南和架构说明\n",
            "3. 保持文档与代码同步\n",
            "4. 使用恰当的格式和结构组织内容",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec![],
            skills: vec!["write".into(), "writer".into(), "document".into(), "doc".into()],
            long_running: false,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Task,
        unlimited_access: false,
    }
}
