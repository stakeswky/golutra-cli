//! Agent 健康检查与自愈策略。

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::contracts::agent::{AgentId, AgentStatus};

use super::lifecycle::AgentLifecycle;

/// 健康检查配置。
pub struct HealthCheckConfig {
    /// 检查间隔。
    pub interval: Duration,
    /// 连续失败多少次触发重启。
    pub failure_threshold: u32,
    /// 连续失败多少次标记为死亡（不再重启）。
    pub dead_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            failure_threshold: 3,
            dead_threshold: 10,
        }
    }
}

/// 单个 Agent 的健康状态追踪。
struct AgentHealthState {
    consecutive_failures: u32,
    last_check: Instant,
    marked_dead: bool,
}

/// 健康检查器：定期检查 Agent 状态并触发自愈。
pub struct HealthChecker {
    config: HealthCheckConfig,
    states: HashMap<AgentId, AgentHealthState>,
}

impl HealthChecker {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
        }
    }

    /// 对所有 Agent 执行一轮健康检查，返回需要通知的事件。
    pub fn check_all(&mut self, lifecycle: &AgentLifecycle) -> Vec<HealthEvent> {
        let mut events = Vec::new();
        let agents = lifecycle.list_agents();

        for (agent_id, role, status) in &agents {
            let state = self.states.entry(agent_id.clone()).or_insert(AgentHealthState {
                consecutive_failures: 0,
                last_check: Instant::now(),
                marked_dead: false,
            });

            if state.marked_dead {
                continue;
            }

            if Instant::now().duration_since(state.last_check) < self.config.interval {
                continue;
            }
            state.last_check = Instant::now();

            match status {
                AgentStatus::Error { message } => {
                    state.consecutive_failures += 1;
                    if state.consecutive_failures >= self.config.dead_threshold {
                        state.marked_dead = true;
                        events.push(HealthEvent::AgentDead {
                            agent_id: agent_id.clone(),
                            role: role.clone(),
                            reason: message.clone(),
                        });
                    } else if state.consecutive_failures >= self.config.failure_threshold {
                        events.push(HealthEvent::AgentRestart {
                            agent_id: agent_id.clone(),
                            role: role.clone(),
                            attempt: state.consecutive_failures,
                        });
                    }
                }
                AgentStatus::Stopped => {
                    state.marked_dead = true;
                    events.push(HealthEvent::AgentStopped {
                        agent_id: agent_id.clone(),
                        role: role.clone(),
                    });
                }
                _ => {
                    state.consecutive_failures = 0;
                }
            }
        }

        // 清理已不存在的 Agent 记录。
        let active_ids: std::collections::HashSet<_> =
            agents.iter().map(|(id, _, _)| id.clone()).collect();
        self.states.retain(|id, _| active_ids.contains(id));

        events
    }

    /// 移除指定 Agent 的健康追踪。
    pub fn remove(&mut self, agent_id: &str) {
        self.states.remove(agent_id);
    }
}

/// 健康事件。
#[derive(Clone, Debug)]
pub enum HealthEvent {
    AgentRestart {
        agent_id: AgentId,
        role: String,
        attempt: u32,
    },
    AgentDead {
        agent_id: AgentId,
        role: String,
        reason: String,
    },
    AgentStopped {
        agent_id: AgentId,
        role: String,
    },
}
