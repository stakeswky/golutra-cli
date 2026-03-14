//! Agent 契约：跨模块共享的 Agent 数据结构与通信协议。

use serde::{Deserialize, Serialize};

/// Agent 唯一标识。
pub type AgentId = String;

/// Agent 能力声明，用于 Planner 选择合适的 Agent。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// 支持的语言列表（如 ["rust", "python"]）。
    pub languages: Vec<String>,
    /// 能力标签（如 ["refactor", "test", "review"]）。
    pub skills: Vec<String>,
    /// 是否支持长时间运行任务。
    pub long_running: bool,
    /// 最大并发任务数。
    pub max_concurrent_tasks: u32,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            skills: Vec::new(),
            long_running: false,
            max_concurrent_tasks: 1,
        }
    }
}

/// Agent 运行状态。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 等待启动。
    Pending,
    /// 正在初始化（PTY 已 spawn，等待就绪信号）。
    Starting,
    /// 空闲，可接收任务。
    Idle,
    /// 正在执行任务。
    Busy { task_id: String },
    /// 异常，可尝试重启。
    Error { message: String },
    /// 已终止。
    Stopped,
}

/// Agent 配置，用于工厂创建 Agent 实例。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 唯一 ID。
    pub id: AgentId,
    /// 角色名称（如 "重构专家"）。
    pub role: String,
    /// 底层工具类型（claude / gemini / codex / opencode / qwen / shell）。
    pub tool_type: String,
    /// 启动命令覆盖（为空则使用默认命令）。
    pub command: Option<String>,
    /// 工作目录。
    pub cwd: Option<String>,
    /// 系统提示词（注入到 Agent 的初始上下文）。
    pub system_prompt: Option<String>,
    /// 能力声明。
    pub capabilities: AgentCapabilities,
    /// 是否启用无限制访问模式。
    pub unlimited_access: bool,
    /// 记忆作用域。
    pub memory_scope: MemoryScope,
}

/// 记忆作用域：控制 Agent 可访问的记忆范围。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryScope {
    /// 仅自身记忆。
    Private,
    /// 同一任务内的 Agent 共享。
    Task,
    /// 全局共享。
    Global,
}

impl Default for MemoryScope {
    fn default() -> Self {
        Self::Task
    }
}

/// Agent 间通信消息。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentMessage {
    /// 指派任务。
    Task {
        id: String,
        instruction: String,
        context: Vec<ContextItem>,
    },
    /// 任务结果。
    Result {
        task_id: String,
        output: String,
        artifacts: Vec<Artifact>,
    },
    /// 进度更新。
    Progress {
        task_id: String,
        percent: f32,
        detail: String,
    },
    /// 错误报告。
    Error {
        task_id: String,
        code: String,
        message: String,
    },
    /// 记忆同步。
    Memory {
        key: String,
        value: String,
        scope: MemoryScope,
    },
    /// 频道广播。
    Broadcast {
        channel_id: String,
        sender_id: AgentId,
        content: String,
    },
}

/// 上下文条目，附加到任务中供 Agent 参考。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextItem {
    /// 上下文类型（file / memory / output / instruction）。
    pub kind: String,
    /// 内容。
    pub content: String,
    /// 来源标识。
    pub source: Option<String>,
}

/// 任务产物（文件修改、日志等）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    /// 产物类型（file / log / metric）。
    pub kind: String,
    /// 路径或标识。
    pub path: Option<String>,
    /// 内容。
    pub content: String,
}

/// 协作频道定义。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub members: Vec<AgentId>,
    pub channel_type: ChannelType,
}

/// 频道类型。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChannelType {
    /// 任务频道：跟随任务生命周期。
    Task { task_id: String },
    /// 持久频道：手动管理。
    Persistent,
    /// 广播频道：单向通知。
    Broadcast,
}

/// 执行计划：Planner 输出的任务编排方案。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub tasks: Vec<TaskNode>,
    /// 任务依赖关系（task_id → 依赖的 task_id 列表）。
    pub dependencies: Vec<(String, String)>,
}

/// 任务节点。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub instruction: String,
    /// 需要的 Agent 角色。
    pub required_role: String,
    /// 需要的工具类型（为空则由 Planner 决定）。
    pub preferred_tool: Option<String>,
    /// 预估复杂度（1-10）。
    pub complexity: u8,
    /// 上下文依赖。
    pub context_keys: Vec<String>,
}
