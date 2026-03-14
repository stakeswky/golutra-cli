//! 任务规划器：分解任务、评估复杂度、生成执行计划（DAG）。

use crate::contracts::agent::{ExecutionPlan, TaskNode};

/// 复杂度评估结果。
#[derive(Clone, Debug)]
pub struct ComplexityAssessment {
    /// 总复杂度（1-10）。
    pub score: u8,
    /// 建议的 Agent 数量。
    pub suggested_agent_count: u8,
    /// 是否需要协作频道。
    pub needs_channel: bool,
    /// 可并行的任务组。
    pub parallel_groups: Vec<Vec<String>>,
}

/// 任务规划器。
pub struct Planner;

impl Planner {
    /// 分解用户指令为任务节点列表。
    ///
    /// 当前为基于规则的简单分解，后续可接入 LLM 做语义分解。
    pub fn decompose(instruction: &str) -> Vec<TaskNode> {
        let lines: Vec<&str> = instruction
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() <= 1 {
            return vec![TaskNode {
                id: "task-0".to_string(),
                instruction: instruction.to_string(),
                required_role: "general".to_string(),
                preferred_tool: None,
                complexity: Self::estimate_complexity(instruction),
                context_keys: Vec::new(),
            }];
        }

        lines
            .iter()
            .enumerate()
            .map(|(i, line)| TaskNode {
                id: format!("task-{i}"),
                instruction: line.to_string(),
                required_role: Self::infer_role(line),
                preferred_tool: None,
                complexity: Self::estimate_complexity(line),
                context_keys: Vec::new(),
            })
            .collect()
    }

    /// 从任务节点列表生成执行计划。
    ///
    /// 识别独立子任务生成并行依赖关系，而非纯顺序链。
    pub fn plan(tasks: Vec<TaskNode>) -> ExecutionPlan {
        let dependencies = Self::build_dependencies(&tasks);
        ExecutionPlan {
            id: format!("plan-{}", ulid::Ulid::new()),
            tasks,
            dependencies,
        }
    }

    /// 构建任务依赖关系。
    ///
    /// 策略：相同角色的任务可并行，不同角色间按出现顺序建立依赖。
    /// 单任务或全部同角色时无依赖（全并行）。
    fn build_dependencies(tasks: &[TaskNode]) -> Vec<(String, String)> {
        if tasks.len() <= 1 {
            return Vec::new();
        }

        // 按角色分组
        let mut role_groups: std::collections::HashMap<&str, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, task) in tasks.iter().enumerate() {
            role_groups
                .entry(&task.required_role)
                .or_default()
                .push(i);
        }

        // 如果所有任务角色相同，全部并行
        if role_groups.len() == 1 {
            return Vec::new();
        }

        // 同角色内部并行，不同角色组之间：后出现的组依赖前一个组的最后一个任务
        let mut deps = Vec::new();
        let mut group_order: Vec<(&&str, &Vec<usize>)> = role_groups.iter().collect();
        group_order.sort_by_key(|(_, indices)| indices[0]);

        for window in group_order.windows(2) {
            let (_, prev_indices) = window[0];
            let (_, curr_indices) = window[1];
            // 当前组的每个任务依赖前一组的最后一个任务
            if let Some(&last_prev) = prev_indices.last() {
                for &curr_idx in curr_indices {
                    deps.push((
                        tasks[curr_idx].id.clone(),
                        tasks[last_prev].id.clone(),
                    ));
                }
            }
        }

        deps
    }

    /// 评估整体复杂度。
    pub fn assess(plan: &ExecutionPlan) -> ComplexityAssessment {
        let max_complexity = plan.tasks.iter().map(|t| t.complexity).max().unwrap_or(1);
        let total_tasks = plan.tasks.len();

        let suggested_agents = match total_tasks {
            0..=1 => 1,
            2..=3 => 2,
            4..=6 => 3,
            _ => 4,
        };

        // 找出无依赖关系的任务组（可并行）
        let dep_targets: Vec<&str> = plan
            .dependencies
            .iter()
            .map(|(target, _)| target.as_str())
            .collect();
        let independent: Vec<String> = plan
            .tasks
            .iter()
            .filter(|t| !dep_targets.contains(&t.id.as_str()))
            .map(|t| t.id.clone())
            .collect();

        let parallel_groups = if independent.len() > 1 {
            vec![independent]
        } else {
            Vec::new()
        };

        ComplexityAssessment {
            score: max_complexity,
            suggested_agent_count: suggested_agents as u8,
            needs_channel: total_tasks > 2 || max_complexity > 5,
            parallel_groups,
        }
    }

    fn estimate_complexity(instruction: &str) -> u8 {
        let len = instruction.len();
        let keywords = [
            "refactor", "重构", "migrate", "迁移", "architecture", "架构",
            "security", "安全", "audit", "审计", "performance", "性能",
        ];
        let keyword_hits = keywords
            .iter()
            .filter(|k| instruction.to_lowercase().contains(*k))
            .count();

        let base = match len {
            0..=50 => 2,
            51..=200 => 4,
            201..=500 => 6,
            _ => 8,
        };

        (base + keyword_hits as u8).min(10)
    }

    fn infer_role(instruction: &str) -> String {
        let lower = instruction.to_lowercase();
        if lower.contains("test") || lower.contains("测试") {
            "tester".to_string()
        } else if lower.contains("review") || lower.contains("审查") {
            "reviewer".to_string()
        } else if lower.contains("refactor") || lower.contains("重构") {
            "refactor".to_string()
        } else if lower.contains("deploy") || lower.contains("部署") || lower.contains("devops") {
            "devops".to_string()
        } else if lower.contains("audit") || lower.contains("审计") {
            "auditor".to_string()
        } else {
            "general".to_string()
        }
    }
}
