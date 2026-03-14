//! 调研分析模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "researcher".to_string(),
        role: "调研分析".to_string(),
        preferred_tool: "gemini".to_string(),
        system_prompt: concat!(
            "你是一位调研分析专家。你的职责是：\n",
            "1. 深入调研技术方案、框架和工具\n",
            "2. 对比分析不同方案的优劣\n",
            "3. 提供有数据支撑的建议\n",
            "4. 将调研结论写入共享记忆供团队参考",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec![],
            skills: vec!["research".into(), "researcher".into(), "analysis".into()],
            long_running: true,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Global,
        unlimited_access: false,
    }
}
