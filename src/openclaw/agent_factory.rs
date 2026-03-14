//! Agent 工厂：根据模板动态创建专用 Agent。

use crate::contracts::agent::{AgentCapabilities, AgentConfig, AgentId, MemoryScope};

use super::agent_templates;

/// Agent 模板定义。
#[derive(Clone, Debug)]
pub struct AgentTemplate {
    /// 模板 ID（如 "refactor", "audit", "devops"）。
    pub id: String,
    /// 角色名称。
    pub role: String,
    /// 推荐的底层工具类型。
    pub preferred_tool: String,
    /// 系统提示词。
    pub system_prompt: String,
    /// 能力声明。
    pub capabilities: AgentCapabilities,
    /// 默认记忆作用域。
    pub default_memory_scope: MemoryScope,
    /// 是否启用无限制访问。
    pub unlimited_access: bool,
}

/// Agent 工厂：从模板创建 AgentConfig。
pub struct AgentFactory {
    templates: Vec<AgentTemplate>,
    counter: std::sync::atomic::AtomicU64,
}

impl AgentFactory {
    pub fn new() -> Self {
        Self {
            templates: agent_templates::builtin_templates(),
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 根据模板 ID 创建 AgentConfig。
    pub fn create_from_template(
        &self,
        template_id: &str,
        cwd: Option<String>,
    ) -> Result<AgentConfig, String> {
        let template = self
            .templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| format!("template not found: {template_id}"))?;

        let seq = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let agent_id = format!("{}-{}", template.id, seq);

        Ok(AgentConfig {
            id: agent_id,
            role: template.role.clone(),
            tool_type: template.preferred_tool.clone(),
            command: None,
            cwd,
            system_prompt: Some(template.system_prompt.clone()),
            capabilities: template.capabilities.clone(),
            unlimited_access: template.unlimited_access,
            memory_scope: template.default_memory_scope.clone(),
        })
    }

    /// 根据角色需求自动选择最佳模板。
    pub fn auto_select(&self, required_role: &str, preferred_tool: Option<&str>) -> Option<&AgentTemplate> {
        // 优先精确匹配 role
        if let Some(t) = self.templates.iter().find(|t| t.role == required_role) {
            return Some(t);
        }
        // 其次匹配 skills
        if let Some(t) = self.templates.iter().find(|t| {
            t.capabilities.skills.iter().any(|s| s == required_role)
        }) {
            return Some(t);
        }
        // 最后按 preferred_tool 匹配
        if let Some(tool) = preferred_tool {
            return self.templates.iter().find(|t| t.preferred_tool == tool);
        }
        None
    }

    /// 从自定义参数创建 AgentConfig（不依赖模板）。
    pub fn create_custom(
        &self,
        role: String,
        tool_type: String,
        system_prompt: Option<String>,
        capabilities: AgentCapabilities,
        cwd: Option<String>,
    ) -> AgentConfig {
        let seq = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let agent_id: AgentId = format!("custom-{}", seq);

        AgentConfig {
            id: agent_id,
            role,
            tool_type,
            command: None,
            cwd,
            system_prompt,
            capabilities,
            unlimited_access: false,
            memory_scope: MemoryScope::Task,
        }
    }

    /// 列出所有可用模板。
    pub fn list_templates(&self) -> &[AgentTemplate] {
        &self.templates
    }

    /// 注册自定义模板。
    pub fn register_template(&mut self, template: AgentTemplate) {
        self.templates.push(template);
    }
}

impl Default for AgentFactory {
    fn default() -> Self {
        Self::new()
    }
}
