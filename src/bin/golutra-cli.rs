//! golutra CLI 入口：独立运行的 Agent 协作引擎（不依赖 GUI）。

use std::path::PathBuf;

use golutra_cli::cli_bootstrap;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("golutra");

    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("无法创建数据目录 {:?}: {}", data_dir, e);
        std::process::exit(1);
    }

    if let Err(e) = cli_bootstrap(data_dir, cwd) {
        eprintln!("启动失败: {e}");
        std::process::exit(1);
    }
}
