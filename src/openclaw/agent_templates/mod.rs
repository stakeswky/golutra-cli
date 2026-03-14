//! 内置 Agent 模板：预定义的行业/场景专用 Agent。

pub(crate) mod refactor;
pub(crate) mod audit;
pub(crate) mod devops;
pub(crate) mod custom;
pub(crate) mod tester;
pub(crate) mod reviewer;
pub(crate) mod researcher;
pub(crate) mod writer;

use super::agent_factory::AgentTemplate;

/// 返回所有内置模板。
pub fn builtin_templates() -> Vec<AgentTemplate> {
    vec![
        refactor::template(),
        audit::template(),
        devops::template(),
        tester::template(),
        reviewer::template(),
        researcher::template(),
        writer::template(),
        custom::general_template(),
    ]
}
