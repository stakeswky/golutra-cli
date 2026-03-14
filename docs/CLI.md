# golutra-cli

## 概述

`golutra-cli` 是从 `golutra` 桌面应用中拆出来的命令行版本。
它专注于终端内的多 Agent 编排，不包含 GUI、Tauri runtime 或任何前端资源。

核心链路：

```text
REPL -> Planner -> DAG Scheduler -> Task Executor -> PTY Adapters
                  \\-> Shared Memory -> History
```

## 模块说明

### `src/cli`

- `repl.rs`: 非阻塞输入循环、命令分发、输出轮询
- `commands.rs`: 斜杠命令解析和调用
- `renderer.rs`: 终端输出格式
- `events.rs`: CLI 事件提示

### `src/agent_runtime`

- `lifecycle.rs`: Agent 注册、启动、重启、发送消息、轮询输出
- `health.rs`: 健康检查与自动重启策略
- `adapters/`: 各类 CLI 工具的 PTY 适配器
- `interface.rs`: 生命周期层的统一接口

### `src/openclaw`

- `planner.rs`: 将输入任务拆为 DAG
- `scheduler.rs`: 基于依赖关系找到可并行任务
- `executor.rs`: 向 Agent 分发任务并聚合输出
- `history.rs`: 记录执行过程
- `protocol.rs` / `channel.rs`: 多 Agent 协作通道
- `agent_factory.rs`: 按角色选择或创建 Agent

### `src/memory`

- `store.rs`: `redb` 持久化
- `shared.rs`: 共享记忆读写与搜索
- `index.rs`: 简单检索索引
- `context.rs`: 把记忆组织成 prompt 上下文

## 构建

```bash
cargo check
cargo build --release
```

## 安装

```bash
cargo install --path .
```

## 运行

```bash
golutra-cli
```

自由文本示例：

```text
> Refactor the error handling in src/lib.rs and add tests.
```

命令示例：

```text
> /agents
> /tasks
> /memory auth
> /output agent-01
```

## 支持的 CLI 工具

| 类型 | 可执行文件 |
| --- | --- |
| Claude Code | `claude` |
| Gemini CLI | `gemini` |
| Codex CLI | `codex` |
| OpenCode | `opencode` |
| Qwen Code | `qwen` |
| Shell | 系统 shell |

## 设计边界

- 不包含桌面端 UI、托盘、窗口、多窗口同步或 Tauri IPC。
- 不包含前端构建链路、Web 资源、主题、组件与静态素材。
- 数据目录仍沿用 `golutra` 名称，以兼容已有共享记忆路径。

## 后续可做的清理

- 进一步消除当前未使用方法带来的 `dead_code` 警告。
- 拆分 `openclaw` 内部模块，使 Planner / Executor 更易测试。
- 为各适配器补充集成测试和 fake CLI 测试桩。
