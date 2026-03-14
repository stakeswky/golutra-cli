//! OpenClaw 指挥层：中心协调器，自动创建 Agent、分配角色、生成协作频道。

pub(crate) mod agent_factory;
pub(crate) mod agent_templates;
pub(crate) mod channel;
pub(crate) mod executor;
pub(crate) mod history;
pub(crate) mod planner;
pub(crate) mod protocol;
pub(crate) mod scheduler;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::agent_runtime::health::{HealthCheckConfig, HealthChecker, HealthEvent};
use crate::agent_runtime::AgentLifecycle;
use crate::contracts::agent::{AgentId, AgentMessage, ExecutionPlan};
use crate::memory::SharedMemory;

use self::agent_factory::AgentFactory;
use self::channel::ChannelManager;
use self::executor::TaskExecutor;
use self::history::ExecutionHistory;
use self::planner::Planner;
use self::protocol::ProtocolRouter;
use self::scheduler::DagScheduler;

/// OpenClaw 协调器：接收用户指令 → 分解任务 → 创建 Agent 团队 → 分发执行。
pub struct OpenClaw {
    lifecycle: Arc<AgentLifecycle>,
    factory: AgentFactory,
    router: ProtocolRouter,
    memory: Arc<SharedMemory>,
    executor: Mutex<TaskExecutor>,
    history: Arc<ExecutionHistory>,
    cwd: Option<String>,
    /// 健康检查后台线程的停止信号。
    health_stop: Arc<std::sync::atomic::AtomicBool>,
    /// 健康事件接收通道。
    health_rx: Mutex<std::sync::mpsc::Receiver<HealthEvent>>,
}

impl OpenClaw {
    pub fn new(
        lifecycle: Arc<AgentLifecycle>,
        memory: Arc<SharedMemory>,
        cwd: Option<String>,
    ) -> Self {
        let health_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (health_tx, health_rx) = std::sync::mpsc::channel();

        // 启动健康检查后台线程
        let lc = Arc::clone(&lifecycle);
        let stop = Arc::clone(&health_stop);
        thread::Builder::new()
            .name("openclaw-health".to_string())
            .spawn(move || {
                let mut checker = HealthChecker::new(HealthCheckConfig::default());
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let events = checker.check_all(&lc);
                    for event in &events {
                        match event {
                            HealthEvent::AgentRestart { agent_id, .. } => {
                                let _ = lc.restart_agent(agent_id);
                            }
                            HealthEvent::AgentDead { agent_id, .. } => {
                                log::warn!("openclaw: agent {agent_id} marked dead");
                            }
                            _ => {}
                        }
                        let _ = health_tx.send(event.clone());
                    }
                    thread::sleep(Duration::from_secs(5));
                }
            })
            .ok();

        Self {
            lifecycle,
            factory: AgentFactory::new(),
            router: ProtocolRouter::new(),
            memory,
            executor: Mutex::new(TaskExecutor::new()),
            history: Arc::new(ExecutionHistory::new()),
            cwd,
            health_stop,
            health_rx: Mutex::new(health_rx),
        }
    }

    /// 核心入口：接收用户指令，自动编排执行（使用 DAG 调度）。
    pub fn execute(&self, instruction: &str) -> Result<ExecutionReport, String> {
        // 1. 分解任务
        let tasks = Planner::decompose(instruction);
        let plan = Planner::plan(tasks);
        let assessment = Planner::assess(&plan);

        log::info!(
            "openclaw: plan={} tasks={} complexity={} agents={}",
            plan.id,
            plan.tasks.len(),
            assessment.score,
            assessment.suggested_agent_count
        );

        // 2. 为每个任务节点创建/复用 Agent
        let mut agent_assignments: Vec<(String, AgentId)> = Vec::new();
        for task in &plan.tasks {
            let agent_id = self.ensure_agent_for_role(
                &task.required_role,
                task.preferred_tool.as_deref(),
            )?;
            agent_assignments.push((task.id.clone(), agent_id));
        }

        // 3. 按需创建协作频道
        let channel_id = if assessment.needs_channel {
            let members: Vec<AgentId> = agent_assignments
                .iter()
                .map(|(_, aid)| aid.clone())
                .collect();
            let mgr = ChannelManager::new(&self.router);
            Some(mgr.create_task_channel(&plan.id, members)?)
        } else {
            None
        };

        // 4. 记录执行历史
        self.history.record_start(
            &plan.id,
            instruction,
            plan.tasks.len(),
            agent_assignments.len(),
            agent_assignments.clone(),
        );

        // 5. 使用 DAG 调度器分发初始可执行任务
        let mut scheduler = DagScheduler::new(&plan);
        self.dispatch_ready_tasks(&plan, &mut scheduler, &agent_assignments)?;

        Ok(ExecutionReport {
            plan_id: plan.id,
            task_count: plan.tasks.len(),
            agent_count: agent_assignments.len(),
            channel_id,
            assignments: agent_assignments,
        })
    }

    /// 分发 DAG 中当前就绪的任务。
    fn dispatch_ready_tasks(
        &self,
        plan: &ExecutionPlan,
        scheduler: &mut DagScheduler,
        assignments: &[(String, AgentId)],
    ) -> Result<(), String> {
        let ready = scheduler.ready_tasks();
        let mut exec = self.executor.lock().map_err(|e| e.to_string())?;

        for task_id in &ready {
            let task_node = plan.tasks.iter().find(|t| &t.id == task_id);
            let agent_id = assignments.iter().find(|(tid, _)| tid == task_id).map(|(_, aid)| aid);

            if let (Some(task), Some(aid)) = (task_node, agent_id) {
                let predecessor_outputs = exec.completed_outputs();
                if let Err(e) = exec.dispatch_with_context(
                    &self.lifecycle,
                    aid,
                    task_id,
                    &task.instruction,
                    &self.memory,
                    &predecessor_outputs,
                ) {
                    log::warn!("openclaw: failed to dispatch {task_id} to {aid}: {e}");
                }
                scheduler.mark_dispatched(task_id);
            }
        }
        Ok(())
    }

    /// 轮询 Agent 输出，返回 (agent_id, task_id, output_chunk) 列表。
    pub fn poll_outputs(&self) -> Vec<(AgentId, String, String)> {
        match self.executor.lock() {
            Ok(mut exec) => exec.poll_outputs(&self.lifecycle),
            Err(_) => Vec::new(),
        }
    }

    /// 是否有活跃任务。
    pub fn has_active_tasks(&self) -> bool {
        match self.executor.lock() {
            Ok(exec) => exec.has_active_tasks(),
            Err(_) => false,
        }
    }

    /// 获取所有任务状态（供 /tasks 命令使用）。
    pub fn task_states(&self) -> Vec<(String, String)> {
        match self.executor.lock() {
            Ok(exec) => exec
                .all_task_states()
                .iter()
                .map(|(id, state)| (id.clone(), format!("{:?}", state)))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 获取指定 Agent 的最近输出。
    pub fn agent_output(&self, agent_id: &str) -> Option<String> {
        match self.executor.lock() {
            Ok(exec) => exec.agent_output(agent_id).cloned(),
            Err(_) => None,
        }
    }

    /// 手动向 Agent 发送消息。
    pub fn send_to_agent(&self, agent_id: &str, message: &str) -> Result<(), String> {
        let msg = AgentMessage::Task {
            id: format!("manual-{}", ulid::Ulid::new()),
            instruction: message.to_string(),
            context: Vec::new(),
        };
        self.lifecycle.send_message(agent_id, &msg)
    }

    /// 手动写入记忆。
    pub fn remember(&self, key: &str, value: &str) -> Result<(), String> {
        self.memory.remember(
            &"user".to_string(),
            crate::contracts::agent::MemoryScope::Global,
            key.to_string(),
            value.to_string(),
        )
    }

    /// 获取健康事件（非阻塞）。
    pub fn poll_health_events(&self) -> Vec<HealthEvent> {
        match self.health_rx.lock() {
            Ok(rx) => {
                let mut events = Vec::new();
                while let Ok(event) = rx.try_recv() {
                    events.push(event);
                }
                events
            }
            Err(_) => Vec::new(),
        }
    }

    /// 获取执行历史。
    pub fn history(&self) -> &ExecutionHistory {
        &self.history
    }

    /// 确保指定角色有可用 Agent，没有则自动创建。
    fn ensure_agent_for_role(
        &self,
        role: &str,
        preferred_tool: Option<&str>,
    ) -> Result<AgentId, String> {
        let existing = self.lifecycle.list_agents();
        for (id, agent_role, status) in &existing {
            if agent_role == role
                && matches!(
                    status,
                    crate::contracts::agent::AgentStatus::Idle
                )
            {
                return Ok(id.clone());
            }
        }

        let config = if let Some(template) = self.factory.auto_select(role, preferred_tool) {
            self.factory
                .create_from_template(&template.id, self.cwd.clone())?
        } else {
            self.factory.create_from_template("general", self.cwd.clone())?
        };

        self.lifecycle.spawn_agent(config)
    }

    /// 获取生命周期管理器引用。
    pub fn lifecycle(&self) -> &AgentLifecycle {
        &self.lifecycle
    }

    /// 获取协议路由器引用（供外部频道操作）。
    pub fn router(&self) -> &ProtocolRouter {
        &self.router
    }

    /// 获取工厂引用。
    pub fn factory(&self) -> &AgentFactory {
        &self.factory
    }

    /// 关闭所有 Agent 和频道，停止健康检查。
    pub fn shutdown(&self) -> Result<(), String> {
        self.health_stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.lifecycle.shutdown_all()
    }
}

/// 执行报告。
#[derive(Clone, Debug)]
pub struct ExecutionReport {
    pub plan_id: String,
    pub task_count: usize,
    pub agent_count: usize,
    pub channel_id: Option<String>,
    pub assignments: Vec<(String, AgentId)>,
}
