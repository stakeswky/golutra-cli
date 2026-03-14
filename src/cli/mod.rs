//! CLI 模块：交互式命令行界面，替代 Tauri GUI。

pub(crate) mod commands;
pub(crate) mod events;
pub(crate) mod renderer;
pub(crate) mod repl;

pub(crate) use repl::Repl;
