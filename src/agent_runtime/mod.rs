//! 统一 Agent 运行时入口：接口定义、生命周期管理、适配器注册与健康检查。

pub(crate) mod interface;
pub(crate) mod lifecycle;
pub(crate) mod adapters;
pub(crate) mod health;

pub(crate) use lifecycle::AgentLifecycle;
