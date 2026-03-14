//! 交互式 REPL：CLI 主循环（非阻塞事件驱动）。

use std::io::{self, Write};
use std::sync::mpsc as std_mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::agent_runtime::health::HealthEvent;
use crate::agent_runtime::AgentLifecycle;
use crate::memory::SharedMemory;
use crate::openclaw::OpenClaw;

use super::commands::{self, CliCommand};
use super::events::CliEventEmitter;
use super::renderer::Renderer;

/// REPL 主循环。
pub struct Repl {
    openclaw: OpenClaw,
    renderer: Renderer,
    events: CliEventEmitter,
    memory: Arc<SharedMemory>,
}

impl Repl {
    pub fn new(
        lifecycle: Arc<AgentLifecycle>,
        memory: Arc<SharedMemory>,
        cwd: Option<String>,
    ) -> Self {
        let openclaw = OpenClaw::new(lifecycle, Arc::clone(&memory), cwd);
        Self {
            openclaw,
            renderer: Renderer::new(),
            events: CliEventEmitter::new(),
            memory,
        }
    }

    /// 启动非阻塞 REPL。
    ///
    /// stdin 在独立线程中读取，主循环通过 try_recv 轮询用户输入和 Agent 输出。
    pub fn run(&self) -> Result<(), String> {
        self.print_banner();

        // stdin reader 线程
        let (stdin_tx, stdin_rx) = std_mpsc::channel::<String>();
        thread::Builder::new()
            .name("repl-stdin".to_string())
            .spawn(move || {
                let stdin = io::stdin();
                loop {
                    let mut line = String::new();
                    match stdin.read_line(&mut line) {
                        Ok(0) => {
                            let _ = stdin_tx.send("\x04".to_string()); // EOF
                            break;
                        }
                        Ok(_) => {
                            if stdin_tx.send(line).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|e| format!("failed to spawn stdin reader: {e}"))?;

        let mut need_prompt = true;

        loop {
            // 显示提示符
            if need_prompt {
                eprint!("\x1b[32mgolutra>\x1b[0m ");
                let _ = io::stderr().flush();
                need_prompt = false;
            }

            // 轮询用户输入
            match stdin_rx.try_recv() {
                Ok(line) => {
                    if line == "\x04" {
                        break; // EOF
                    }
                    let should_quit = self.handle_input(&line);
                    if should_quit {
                        break;
                    }
                    need_prompt = true;
                }
                Err(std_mpsc::TryRecvError::Disconnected) => break,
                Err(std_mpsc::TryRecvError::Empty) => {}
            }

            // 轮询 Agent 输出
            let outputs = self.openclaw.poll_outputs();
            if !outputs.is_empty() {
                // 输出前清除当前行（提示符）
                if !need_prompt {
                    eprint!("\r\x1b[K");
                }
                for (agent_id, _task_id, chunk) in &outputs {
                    self.renderer.agent_output(agent_id, chunk);
                }
                need_prompt = true;
            }

            // 轮询健康事件
            let health_events = self.openclaw.poll_health_events();
            for event in &health_events {
                if !need_prompt {
                    eprint!("\r\x1b[K");
                }
                match event {
                    HealthEvent::AgentRestart {
                        agent_id,
                        role,
                        attempt,
                    } => {
                        eprintln!(
                            "[!] Agent {} ({}) 重启中 (第 {} 次)",
                            agent_id, role, attempt
                        );
                    }
                    HealthEvent::AgentDead {
                        agent_id,
                        role,
                        reason,
                    } => {
                        eprintln!(
                            "[✗] Agent {} ({}) 已死亡: {}",
                            agent_id, role, reason
                        );
                    }
                    HealthEvent::AgentStopped { agent_id, role } => {
                        eprintln!("[·] Agent {} ({}) 已停止", agent_id, role);
                    }
                }
                need_prompt = true;
            }

            // 避免忙等
            thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }

    /// 处理一行输入，返回是否退出。
    fn handle_input(&self, line: &str) -> bool {
        match commands::parse(line) {
            CliCommand::Execute(instruction) => self.handle_execute(&instruction),
            CliCommand::ListAgents => self.handle_list_agents(),
            CliCommand::AgentStatus(id) => self.handle_agent_status(&id),
            CliCommand::KillAgent(id) => self.handle_kill_agent(&id),
            CliCommand::ListTemplates => self.handle_list_templates(),
            CliCommand::ListChannels => self.handle_list_channels(),
            CliCommand::SearchMemory(query) => self.handle_search_memory(&query),
            CliCommand::AgentOutput(id) => self.handle_agent_output(&id),
            CliCommand::Tasks => self.handle_tasks(),
            CliCommand::SendMessage(id, msg) => self.handle_send(&id, &msg),
            CliCommand::Remember(key, value) => self.handle_remember(&key, &value),
            CliCommand::History => self.handle_history(),
            CliCommand::Help => self.print_help(),
            CliCommand::Quit => {
                self.events.info("正在关闭...");
                let _ = self.openclaw.shutdown();
                return true;
            }
            CliCommand::Unknown(msg) => {
                if !msg.is_empty() {
                    self.events.error(&msg);
                }
            }
        }
        false
    }

    fn handle_execute(&self, instruction: &str) {
        match self.openclaw.execute(instruction) {
            Ok(report) => {
                self.events.plan_created(
                    &report.plan_id,
                    report.task_count,
                    report.agent_count,
                );
                for (task_id, agent_id) in &report.assignments {
                    self.events.task_assigned(task_id, agent_id);
                }
                if let Some(ch) = &report.channel_id {
                    self.events.channel_created(ch, report.agent_count);
                }
                self.renderer.report_summary(
                    &report.plan_id,
                    report.task_count,
                    report.agent_count,
                    report.channel_id.as_deref(),
                );
            }
            Err(e) => self.events.error(&e),
        }
    }

    fn handle_list_agents(&self) {
        let agents = self.openclaw.lifecycle().list_agents();
        if agents.is_empty() {
            eprintln!("(无活跃 Agent)");
            return;
        }
        let display: Vec<(String, String, String)> = agents
            .into_iter()
            .map(|(id, role, status)| (id, role, format!("{:?}", status)))
            .collect();
        self.renderer.agent_list(&display);
    }

    fn handle_agent_status(&self, id: &str) {
        match self.openclaw.lifecycle().agent_status(id) {
            Ok(status) => eprintln!("  Agent {} — {:?}", id, status),
            Err(e) => self.events.error(&e),
        }
    }

    fn handle_kill_agent(&self, id: &str) {
        match self.openclaw.lifecycle().kill_agent(id) {
            Ok(()) => eprintln!("  Agent {} 已终止", id),
            Err(e) => self.events.error(&e),
        }
    }

    fn handle_list_templates(&self) {
        let templates = self.openclaw.factory().list_templates();
        for t in templates {
            eprintln!("  {} — {} [{}]", t.id, t.role, t.preferred_tool);
        }
    }

    fn handle_list_channels(&self) {
        let channels = self.openclaw.router().list_channels();
        if channels.is_empty() {
            eprintln!("(无活跃频道)");
            return;
        }
        for ch in &channels {
            eprintln!("  {} — {} ({} 成员)", ch.id, ch.name, ch.members.len());
        }
    }

    fn handle_search_memory(&self, query: &str) {
        let results = self.memory.search(query, 10);
        if results.is_empty() {
            eprintln!("(无匹配记忆)");
            return;
        }
        for key in &results {
            eprintln!("  {}", key);
        }
    }

    fn handle_agent_output(&self, agent_id: &str) {
        match self.openclaw.agent_output(agent_id) {
            Some(output) if !output.is_empty() => {
                self.renderer.separator();
                eprintln!("Agent {} 输出:", agent_id);
                eprintln!("{}", output);
                self.renderer.separator();
            }
            _ => eprintln!("(Agent {} 无输出)", agent_id),
        }
    }

    fn handle_tasks(&self) {
        let states = self.openclaw.task_states();
        if states.is_empty() {
            eprintln!("(无任务)");
            return;
        }
        for (id, state) in &states {
            eprintln!("  {} — {}", id, state);
        }
    }

    fn handle_send(&self, agent_id: &str, message: &str) {
        match self.openclaw.send_to_agent(agent_id, message) {
            Ok(()) => eprintln!("  消息已发送至 {}", agent_id),
            Err(e) => self.events.error(&e),
        }
    }

    fn handle_remember(&self, key: &str, value: &str) {
        match self.openclaw.remember(key, value) {
            Ok(()) => eprintln!("  已记忆: {} = {}", key, value),
            Err(e) => self.events.error(&e),
        }
    }

    fn handle_history(&self) {
        let records = self.openclaw.history().list_recent(10);
        if records.is_empty() {
            eprintln!("(无执行历史)");
            return;
        }
        self.renderer.separator();
        for record in &records {
            let status = if record.finished_at.is_some() {
                "完成"
            } else {
                "进行中"
            };
            eprintln!(
                "  {} — {} 个任务, {} 个 Agent [{}]",
                record.plan_id, record.task_count, record.agent_count, status
            );
            eprintln!("    指令: {}", truncate(&record.instruction, 60));
        }
        self.renderer.separator();
    }

    fn print_banner(&self) {
        eprintln!();
        eprintln!("  \x1b[1;36mgolutra\x1b[0m — AI Agent 协作引擎");
        eprintln!("  输入任务指令开始，/help 查看命令");
        eprintln!();
    }

    fn print_help(&self) {
        eprintln!();
        eprintln!("命令:");
        eprintln!("  <任务描述>              直接输入任务，OpenClaw 自动编排执行");
        eprintln!("  /agents                 列出活跃 Agent");
        eprintln!("  /status <id>            查看 Agent 状态");
        eprintln!("  /kill <id>              终止 Agent");
        eprintln!("  /output <agent_id>      查看 Agent 最近输出");
        eprintln!("  /tasks                  查看当前任务状态");
        eprintln!("  /send <id> <message>    手动向 Agent 发送消息");
        eprintln!("  /templates              列出可用 Agent 模板");
        eprintln!("  /channels               列出协作频道");
        eprintln!("  /memory <query>         搜索共享记忆");
        eprintln!("  /remember <key> <value> 手动写入记忆");
        eprintln!("  /history                查看执行历史");
        eprintln!("  /help                   显示帮助");
        eprintln!("  /quit                   退出");
        eprintln!();
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
