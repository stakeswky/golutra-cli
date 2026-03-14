//! 通用 Agent 模板（兜底）。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn general_template() -> AgentTemplate {
    AgentTemplate {
        id: "general".to_string(),
        role: "通用助手".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位通用编程助手。你的职责是：\n",
            "1. 理解并执行分配的编程任务\n",
            "2. 与团队中其他 Agent 协作\n",
            "3. 将关键发现写入共享记忆",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec![
                "rust".into(), "python".into(), "typescript".into(),
                "javascript".into(), "go".into(), "java".into(),
            ],
            skills: vec!["code_generation".into(), "explain".into(), "debug".into()],
            long_running: true,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Task,
        unlimited_access: false,
    }
}
