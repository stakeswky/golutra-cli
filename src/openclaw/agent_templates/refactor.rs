//! 重构专家模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "refactor".to_string(),
        role: "重构专家".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位代码重构专家。你的职责是：\n",
            "1. 分析代码结构，识别坏味道和技术债务\n",
            "2. 提出安全的重构方案，保持行为不变\n",
            "3. 执行重构并确保测试通过\n",
            "4. 记录重构决策到共享记忆供团队参考",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec!["rust".into(), "typescript".into(), "python".into(), "go".into()],
            skills: vec!["refactor".into(), "code_review".into(), "architecture".into()],
            long_running: true,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Task,
        unlimited_access: true,
    }
}
