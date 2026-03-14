//! 适配器共享的 PTY 启动逻辑。

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use crate::contracts::agent::{AgentConfig, AgentMessage};
use crate::agent_runtime::interface::AgentHandle;

/// PTY 启动参数。
pub(crate) struct PtySpawnOpts {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

/// 通过 PTY 启动 Agent 进程，返回 AgentHandle。
/// 后台线程读取 stdout 并通过 tx_output 发送给调用方。
pub(crate) fn spawn_pty_agent(
    config: &AgentConfig,
    opts: PtySpawnOpts,
) -> Result<AgentHandle, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: opts.rows,
            cols: opts.cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("failed to open pty: {e}"))?;

    let mut cmd = CommandBuilder::new(&opts.program);
    if !opts.args.is_empty() {
        cmd.args(&opts.args);
    }
    if let Some(dir) = opts.cwd.as_deref().filter(|d| !d.is_empty()) {
        cmd.cwd(dir);
    }
    cmd.env("TERM", "xterm-256color");

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("failed to spawn agent process: {e}"))?;

    let master = pair.master;
    let mut reader = master
        .try_clone_reader()
        .map_err(|e| format!("failed to clone pty reader: {e}"))?;
    let writer = master
        .take_writer()
        .map_err(|e| format!("failed to take pty writer: {e}"))?;
    let writer = Arc::new(Mutex::new(writer));
    let killer = child.clone_killer();

    let alive = Arc::new(AtomicBool::new(true));
    let alive_clone = Arc::clone(&alive);

    // 结构化消息通道（用于 lifecycle 层）
    let (tx, rx_agent) = mpsc::channel();
    let (_tx_agent, _rx) = mpsc::channel();

    let agent_id = config.id.clone();

    // 后台线程：读取 PTY stdout，转为 AgentMessage::Result 发送
    thread::Builder::new()
        .name(format!("pty-reader-{}", agent_id))
        .spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        alive_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.send(AgentMessage::Result {
                            task_id: String::new(),
                            output,
                            artifacts: Vec::new(),
                        });
                    }
                    Err(_) => {
                        alive_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
        .map_err(|e| format!("failed to spawn reader thread: {e}"))?;

    log::info!(
        "pty agent spawned: id={} program={} args={:?}",
        config.id,
        opts.program,
        opts.args
    );

    Ok(AgentHandle {
        id: config.id.clone(),
        role: config.role.clone(),
        tool_type: config.tool_type.clone(),
        pty_writer: Some(Arc::clone(&writer)),
        killer: Some(killer),
        alive,
        sender: _tx_agent,
        receiver: Arc::new(Mutex::new(rx_agent)),
    })
}

/// 向 PTY 写入文本（追加换行）。
pub(crate) fn write_to_pty(handle: &AgentHandle, text: &str) -> Result<(), String> {
    if let Some(writer) = &handle.pty_writer {
        let mut w = writer.lock().map_err(|e| e.to_string())?;
        use std::io::Write;
        w.write_all(text.as_bytes())
            .and_then(|_| w.write_all(b"\n"))
            .and_then(|_| w.flush())
            .map_err(|e| format!("pty write failed: {e}"))
    } else {
        Err("no pty writer available".to_string())
    }
}

/// 终止 PTY 进程。
pub(crate) fn kill_pty(handle: &AgentHandle) -> Result<(), String> {
    handle.alive.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Some(killer) = &handle.killer {
        killer.clone_killer().kill().map_err(|e| format!("kill failed: {e}"))
    } else {
        Ok(())
    }
}
