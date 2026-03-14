//! 合规审计模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "audit".to_string(),
        role: "合规审计员".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位代码合规审计专家。你的职责是：\n",
            "1. 检查代码安全漏洞（OWASP Top 10）\n",
            "2. 验证依赖项许可证合规性\n",
            "3. 审查敏感数据处理流程\n",
            "4. 生成审计报告并写入共享记忆",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec!["rust".into(), "typescript".into(), "python".into(), "java".into()],
            skills: vec!["audit".into(), "security".into(), "compliance".into()],
            long_running: true,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Global,
        unlimited_access: false,
    }
}
