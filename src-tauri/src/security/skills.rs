//! 在线技能黑/灰名单 — 拦截运行时调用
//!
//! 与 [`super::skill_scanner`] 区分：scanner 是**安装前**检查，
//! 本模块是**运行时**调用检查（基于已扫描的评分 + 用户手动 disable）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillSafetyLabel {
    Safe,
    Warn,
    Disabled,
    Unaudited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub slug: String,
    pub label: SkillSafetyLabel,
    pub score: u32,
    pub user_enabled: bool,
}

/// 运行时调用前评估：是否允许加载该技能。
pub fn evaluate_call(skill: &SkillRecord) -> bool {
    if !skill.user_enabled { return false; }
    if skill.label == SkillSafetyLabel::Disabled { return false; }
    if skill.score < 30 { return false; } // hard floor
    true
}
