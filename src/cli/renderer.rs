//! CLI 终端渲染：Agent 输出的格式化显示。

use std::io::{self, Write};

/// 渲染器：格式化 Agent 输出到终端。
pub struct Renderer {
    use_color: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            use_color: atty::is(atty::Stream::Stderr),
        }
    }

    /// 渲染 Agent 输出。
    pub fn agent_output(&self, agent_id: &str, content: &str) {
        if self.use_color {
            eprint!("\x1b[36m[{}]\x1b[0m ", agent_id);
        } else {
            eprint!("[{}] ", agent_id);
        }
        eprintln!("{}", content);
    }

    /// 渲染进度条。
    pub fn progress(&self, task_id: &str, percent: f32, detail: &str) {
        let bar_width = 20;
        let filled = (percent / 100.0 * bar_width as f32) as usize;
        let empty = bar_width - filled;
        let bar = format!(
            "[{}{}] {:.0}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percent
        );
        if self.use_color {
            eprint!("\x1b[33m{}\x1b[0m {} {}\r", task_id, bar, detail);
        } else {
            eprint!("{} {} {}\r", task_id, bar, detail);
        }
        let _ = io::stderr().flush();
    }

    /// 渲染分隔线。
    pub fn separator(&self) {
        eprintln!("{}", "─".repeat(60));
    }

    /// 渲染执行报告摘要。
    pub fn report_summary(
        &self,
        plan_id: &str,
        task_count: usize,
        agent_count: usize,
        channel_id: Option<&str>,
    ) {
        self.separator();
        eprintln!("执行计划: {}", plan_id);
        eprintln!("任务数: {}  Agent 数: {}", task_count, agent_count);
        if let Some(ch) = channel_id {
            eprintln!("协作频道: {}", ch);
        }
        self.separator();
    }

    /// 渲染 Agent 列表。
    pub fn agent_list(&self, agents: &[(String, String, String)]) {
        if agents.is_empty() {
            eprintln!("(无活跃 Agent)");
            return;
        }
        for (id, role, status) in agents {
            eprintln!("  {} — {} [{}]", id, role, status);
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
