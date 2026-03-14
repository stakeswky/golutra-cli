# golutra-cli

`golutra-cli` 是从 `golutra` 桌面端仓库抽离出来的无头多 Agent 编排引擎。
它保留了 OpenClaw 的任务分解、DAG 调度、共享记忆和 PTY Agent 生命周期管理，但不再依赖 Tauri、WebView 或前端代码。

## 特性

- 纯终端运行，单进程管理多 Agent
- 兼容现有 CLI 工具：Claude、Gemini、Codex、OpenCode、Qwen、Shell
- OpenClaw 自动做任务分解、角色分配和并行调度
- 基于 `redb` 的跨 Agent 共享记忆
- 无 GUI 依赖，可直接 `cargo build` / `cargo install`

## 前置条件

- Rust >= `1.77.2`
- 至少一个可执行的 AI CLI 已加入 `PATH`

| Agent | CLI 命令 | 安装方式 |
| --- | --- | --- |
| Claude Code | `claude` | `npm i -g @anthropic-ai/claude-code` |
| Gemini CLI | `gemini` | `npm i -g @anthropic-ai/gemini-cli` |
| Codex CLI | `codex` | `npm i -g @openai/codex` |
| OpenCode | `opencode` | 参考其官方仓库 |
| Qwen Code | `qwen` | 参考其官方仓库 |
| Shell | 系统 shell | 内置 |

## 编译

```bash
cargo build --release
```

构建产物：

```bash
target/release/golutra-cli
```

## 运行

```bash
./target/release/golutra-cli
```

或者安装到全局：

```bash
cargo install --path .
golutra-cli
```

启动后会进入 REPL：

```text
🐾 golutra-cli — OpenClaw Agent Orchestrator
输入任务指令，或 /help 查看命令列表。
>
```

## 交互方式

1. 直接输入自然语言任务，交给 OpenClaw 自动拆解和分发。
2. 使用斜杠命令查询 Agent、任务、共享记忆和执行历史。

常用命令：

- `/agents` 或 `/ls`
- `/status <agent_id>`
- `/kill <agent_id>`
- `/tasks`
- `/output <agent_id>`
- `/send <agent_id> <msg>`
- `/memory <query>`
- `/remember <key> <value>`
- `/history`
- `/help`
- `/quit`

## 数据目录

默认数据目录：

- macOS: `~/Library/Application Support/golutra/`
- Linux: `~/.local/share/golutra/`
- Windows: `%APPDATA%\\golutra\\`

其中 `memory.redb` 用于持久化共享记忆。

## 仓库结构

```text
src/
  agent_runtime/   PTY 适配器与 Agent 生命周期
  cli/             REPL、命令解析、终端输出
  contracts/       CLI 运行时共享协议
  memory/          redb 存储、索引与上下文拼装
  openclaw/        planner / scheduler / executor / history
```

## 说明

- 这个仓库只包含 CLI 相关代码，不再包含桌面端、前端、Tauri 配置和发布资源。
- 许可证沿用原项目的 `Business Source License 1.1`。
- 更详细的使用和架构说明见 [docs/CLI.md](docs/CLI.md)。
