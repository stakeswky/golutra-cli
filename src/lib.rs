//! CLI-only library entry for golutra-cli.

mod agent_runtime;
mod cli;
mod contracts;
mod memory;
mod openclaw;

use std::sync::Arc;

use agent_runtime::adapters::all_adapters;
use agent_runtime::AgentLifecycle;
use cli::Repl;
use memory::{MemoryStore, SharedMemory};

/// Bootstrap the standalone CLI runtime.
pub fn cli_bootstrap(
    data_dir: std::path::PathBuf,
    cwd: Option<String>,
) -> Result<(), String> {
    let memory_path = data_dir.join("memory.redb");
    let store = Arc::new(MemoryStore::open(memory_path)?);
    let shared_memory = Arc::new(SharedMemory::new(store)?);

    let lifecycle = Arc::new(AgentLifecycle::new());
    for adapter in all_adapters() {
        lifecycle.register_adapter(adapter);
    }

    let repl = Repl::new(lifecycle, shared_memory, cwd);
    repl.run()
}
