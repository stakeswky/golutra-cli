//! DevOps 自动化模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "devops".to_string(),
        role: "DevOps 工程师".to_string(),
        preferred_tool: "shell".to_string(),
        system_prompt: concat!(
            "你是一位 DevOps 自动化工程师。你的职责是：\n",
            "1. 执行构建、测试、部署流水线\n",
            "2. 管理基础设施配置\n",
            "3. 监控服务健康状态\n",
            "4. 将运维知识沉淀到共享记忆",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec!["bash".into(), "python".into(), "yaml".into()],
            skills: vec!["devops".into(), "deploy".into(), "monitor".into(), "ci_cd".into()],
            long_running: false,
            max_concurrent_tasks: 4,
        },
        default_memory_scope: MemoryScope::Global,
        unlimited_access: true,
    }
}
