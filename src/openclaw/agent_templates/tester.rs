//! 测试专家模板。

use crate::contracts::agent::{AgentCapabilities, MemoryScope};
use crate::openclaw::agent_factory::AgentTemplate;

pub fn template() -> AgentTemplate {
    AgentTemplate {
        id: "tester".to_string(),
        role: "测试专家".to_string(),
        preferred_tool: "claude".to_string(),
        system_prompt: concat!(
            "你是一位测试专家。你的职责是：\n",
            "1. 分析代码逻辑，设计全面的测试用例\n",
            "2. 编写单元测试、集成测试和端到端测试\n",
            "3. 识别边界条件和异常路径\n",
            "4. 确保测试覆盖率达标并记录测试策略",
        )
        .to_string(),
        capabilities: AgentCapabilities {
            languages: vec!["rust".into(), "typescript".into(), "python".into()],
            skills: vec!["test".into(), "tester".into(), "qa".into()],
            long_running: false,
            max_concurrent_tasks: 1,
        },
        default_memory_scope: MemoryScope::Task,
        unlimited_access: true,
    }
}
