//! Agent 适配器入口：各 CLI 工具的统一接入层。

pub(crate) mod pty_common;
pub(crate) mod claude;
pub(crate) mod gemini;
pub(crate) mod codex;
pub(crate) mod opencode;
pub(crate) mod qwen;
pub(crate) mod shell;

use std::sync::Arc;
use crate::agent_runtime::interface::AgentInterface;

/// 注册所有内置适配器。
pub(crate) fn all_adapters() -> Vec<Arc<dyn AgentInterface>> {
    vec![
        Arc::new(claude::ClaudeAdapter::new()),
        Arc::new(gemini::GeminiAdapter::new()),
        Arc::new(codex::CodexAdapter::new()),
        Arc::new(opencode::OpenCodeAdapter::new()),
        Arc::new(qwen::QwenAdapter::new()),
        Arc::new(shell::ShellAdapter::new()),
    ]
}
