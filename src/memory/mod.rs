//! 深度记忆层：跨 Agent 长期共享记忆，支持持久化、语义索引与上下文组装。

pub(crate) mod store;
pub(crate) mod index;
pub(crate) mod shared;
pub(crate) mod context;

pub(crate) use shared::SharedMemory;
pub(crate) use store::MemoryStore;
