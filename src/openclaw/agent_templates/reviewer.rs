//! 代码审查模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "reviewer".to_string(),
        role: "代码审查".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位代码审查专家。你的职责是：\n",
            "1. 审查代码质量、可读性和可维护性\n",
            "2. 检查潜在的 bug、安全漏洞和性能问题\n",
            "3. 提出改进建议并说明理由\n",
            "4. 确保代码符合项目规范和最佳实践",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec!["rust".into(), "typescript".into(), "python".into(), "go".into()],
            skills: vec!["review".into(), "reviewer".into(), "code_review".into()],
            long_running: false,
            max_concurrent_tasks: 2,
        },
        default_memory_scope: MemoryScope::Task,
        unlimited_access: false,
    }
}
