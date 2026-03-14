//! CLI 命令解析。

/// 用户输入的 CLI 命令。
#[derive(Clone, Debug)]
pub enum CliCommand {
    /// 执行任务指令（发送给 OpenClaw）。
    Execute(String),
    /// 列出活跃 Agent。
    ListAgents,
    /// 查看 Agent 状态。
    AgentStatus(String),
    /// 终止指定 Agent。
    KillAgent(String),
    /// 列出可用模板。
    ListTemplates,
    /// 列出频道。
    ListChannels,
    /// 搜索记忆。
    SearchMemory(String),
    /// 查看指定 Agent 的最近输出。
    AgentOutput(String),
    /// 查看当前任务状态。
    Tasks,
    /// 手动向 Agent 发送消息。
    SendMessage(String, String),
    /// 手动写入记忆。
    Remember(String, String),
    /// 查看执行历史。
    History,
    /// 显示帮助。
    Help,
    /// 退出。
    Quit,
    /// 未知命令。
    Unknown(String),
}

/// 解析用户输入为 CLI 命令。
pub fn parse(input: &str) -> CliCommand {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return CliCommand::Unknown(String::new());
    }

    if !trimmed.starts_with('/') {
        return CliCommand::Execute(trimmed.to_string());
    }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default();

    match cmd {
        "/agents" | "/ls" => CliCommand::ListAgents,
        "/status" => {
            if arg.is_empty() {
                CliCommand::ListAgents
            } else {
                CliCommand::AgentStatus(arg)
            }
        }
        "/kill" => {
            if arg.is_empty() {
                CliCommand::Unknown("用法: /kill <agent_id>".to_string())
            } else {
                CliCommand::KillAgent(arg)
            }
        }
        "/templates" => CliCommand::ListTemplates,
        "/channels" => CliCommand::ListChannels,
        "/memory" | "/search" => {
            if arg.is_empty() {
                CliCommand::Unknown("用法: /memory <query>".to_string())
            } else {
                CliCommand::SearchMemory(arg)
            }
        }
        "/output" => {
            if arg.is_empty() {
                CliCommand::Unknown("用法: /output <agent_id>".to_string())
            } else {
                CliCommand::AgentOutput(arg)
            }
        }
        "/tasks" => CliCommand::Tasks,
        "/send" => {
            let send_parts: Vec<&str> = arg.splitn(2, ' ').collect();
            if send_parts.len() < 2 || send_parts[1].trim().is_empty() {
                CliCommand::Unknown("用法: /send <agent_id> <message>".to_string())
            } else {
                CliCommand::SendMessage(
                    send_parts[0].to_string(),
                    send_parts[1].trim().to_string(),
                )
            }
        }
        "/remember" => {
            let rem_parts: Vec<&str> = arg.splitn(2, ' ').collect();
            if rem_parts.len() < 2 || rem_parts[1].trim().is_empty() {
                CliCommand::Unknown("用法: /remember <key> <value>".to_string())
            } else {
                CliCommand::Remember(
                    rem_parts[0].to_string(),
                    rem_parts[1].trim().to_string(),
                )
            }
        }
        "/history" => CliCommand::History,
        "/help" | "/?" => CliCommand::Help,
        "/quit" | "/exit" | "/q" => CliCommand::Quit,
        _ => CliCommand::Unknown(format!("未知命令: {cmd}")),
    }
}
