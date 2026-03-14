//! 统一 Agent 接口：标准化所有 AI CLI 工具的接入协议。

use std::io::Write;
use std::sync::Arc;

use crate::contracts::agent::{AgentCapabilities, AgentConfig, AgentId, AgentMessage, AgentStatus};

/// Agent 运行时句柄，持有与底层 PTY 进程的通信通道。
pub struct AgentHandle {
    pub id: AgentId,
    pub role: String,
    pub tool_type: String,
    /// PTY writer（向 Agent stdin 写入）。
    pub pty_writer: Option<Arc<std::sync::Mutex<Box<dyn Write + Send>>>>,
    /// 子进程 killer。
    pub killer: Option<Box<dyn portable_pty::ChildKiller + Send + Sync>>,
    /// 进程是否存活。
    pub alive: Arc<std::sync::atomic::AtomicBool>,
    /// 向 Agent 发送结构化消息的通道。
    pub sender: std::sync::mpsc::Sender<AgentMessage>,
    /// 从 Agent 接收结构化消息的通道。
    pub receiver: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<AgentMessage>>>,
}

/// 统一 Agent 接口 trait。
/// 每个 CLI 工具适配器（Claude/Gemini/Codex 等）实现此 trait。
pub trait AgentInterface: Send + Sync {
    /// Agent 标识。
    fn id(&self) -> &str;

    /// 能力声明。
    fn capabilities(&self) -> &AgentCapabilities;

    /// 启动 Agent 进程，返回运行时句柄。
    fn spawn(&self, config: &AgentConfig) -> Result<AgentHandle, String>;

    /// 向 Agent 发送指令（通过 PTY stdin）。
    fn send(&self, handle: &AgentHandle, message: &AgentMessage) -> Result<(), String>;

    /// 查询 Agent 当前状态。
    fn status(&self, handle: &AgentHandle) -> AgentStatus;

    /// 终止 Agent 进程。
    fn shutdown(&self, handle: &AgentHandle) -> Result<(), String>;

    /// 工具类型标识（claude / gemini / codex / opencode / qwen / shell）。
    fn tool_type(&self) -> &str;
}
