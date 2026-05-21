//! 安全规则统一注册中心
//!
//! 把散落在 danger.rs / injection.rs / redact.rs / skill_scanner.rs / checks/*.rs
//! 5 类规则注册为统一的 `RuleDescriptor`，供 UI 配置菜单一站式管理。
//!
//! 注意：本模块只做"列出 + 描述"，不持有 enabled 状态。
//! 用户的 enable / disable / action 覆盖存在 `security_rule_overrides` 表，
//! pipeline 调用前查 override map。

use serde::Serialize;

/// 规则种类 — 决定它从哪个常量集合来源 + 默认动作如何回退。
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// danger.rs::BUILTIN_RULES — 危险指令
    Danger,
    /// injection.rs::PATTERNS — 提示词注入
    Injection,
    /// redact.rs::PATTERNS — 凭据指纹
    Credential,
    /// skill_scanner.rs::RULES — SkillGuard
    Skill,
    /// security/checks/*.rs — 80 项 AI 安全审计
    Audit,
}

impl RuleKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleKind::Danger => "danger",
            RuleKind::Injection => "injection",
            RuleKind::Credential => "credential",
            RuleKind::Skill => "skill",
            RuleKind::Audit => "audit",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RuleKind::Danger => "危险指令",
            RuleKind::Injection => "提示词注入",
            RuleKind::Credential => "凭据指纹",
            RuleKind::Skill => "技能供应链",
            RuleKind::Audit => "AI 安全审计",
        }
    }
}

/// 默认触发动作
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefaultAction {
    /// 命中即 Block · score 直接归零（SkillGuard HardTrigger 等）
    HardBlock,
    /// 命中即 Block
    Block,
    /// 命中告警不拦截
    Warn,
    /// 加权扣分类（SkillGuard Weighted）
    Weighted,
    /// 当前未实现 · 默认 Skipped
    Skipped,
}

impl DefaultAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            DefaultAction::HardBlock => "hard_block",
            DefaultAction::Block => "block",
            DefaultAction::Warn => "warn",
            DefaultAction::Weighted => "weighted",
            DefaultAction::Skipped => "skipped",
        }
    }
}

/// 统一规则描述符（不可变 · 来自代码常量）
#[derive(Debug, Clone, Serialize)]
pub struct RuleDescriptor {
    pub kind: RuleKind,
    pub id: String,
    pub category: Option<String>,    // injection 的 5 类 / audit 的 8 类
    pub description: String,
    pub default_action: DefaultAction,
    pub pattern_hint: Option<String>, // 简短模式提示（regex 或 needle 摘要）
    pub remediation: Option<String>,
}

/// 列出所有 builtin 规则，供 UI 配置菜单展示。
pub fn list_all_descriptors() -> Vec<RuleDescriptor> {
    let mut out = Vec::with_capacity(160);
    out.extend(danger_descriptors());
    out.extend(injection_descriptors());
    out.extend(credential_descriptors());
    out.extend(skill_descriptors());
    out.extend(audit_descriptors());
    out
}

// ──────────────────────────────────────────────────────────────────
// danger.rs
// ──────────────────────────────────────────────────────────────────
fn danger_descriptors() -> Vec<RuleDescriptor> {
    use crate::security::danger::BUILTIN_RULES;
    BUILTIN_RULES
        .iter()
        .map(|r| RuleDescriptor {
            kind: RuleKind::Danger,
            id: r.id.to_string(),
            category: None,
            description: r.description.to_string(),
            default_action: DefaultAction::Block,
            pattern_hint: Some(r.pattern_raw.to_string()),
            remediation: None,
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────
// injection.rs
// ──────────────────────────────────────────────────────────────────
fn injection_descriptors() -> Vec<RuleDescriptor> {
    use crate::security::injection::PATTERNS;
    PATTERNS
        .iter()
        .map(|p| {
            // execution / override / tool_description / jailbreak 默认 Block
            // exfiltration / tool_abuse 默认 Warn
            let action = match p.category {
                "exfiltration" | "tool_abuse" => DefaultAction::Warn,
                _ => DefaultAction::Block,
            };
            let needles_joined = p.needles.join(" / ");
            RuleDescriptor {
                kind: RuleKind::Injection,
                id: p.id.to_string(),
                category: Some(p.category.to_string()),
                description: format!("注入模式：{}", needles_joined),
                default_action: action,
                pattern_hint: Some(needles_joined),
                remediation: None,
            }
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────
// redact.rs
// ──────────────────────────────────────────────────────────────────
fn credential_descriptors() -> Vec<RuleDescriptor> {
    use crate::security::redact::PATTERNS;
    PATTERNS
        .iter()
        .map(|p| RuleDescriptor {
            kind: RuleKind::Credential,
            id: p.id.to_string(),
            category: Some(p.class.to_string()),
            description: format!("识别 {} 类凭据指纹（{}*）", p.class, p.prefix),
            default_action: DefaultAction::Warn, // DLP 默认脱敏 + 记事件，不阻断
            pattern_hint: Some(p.regex_like.to_string()),
            remediation: None,
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────
// skill_scanner.rs
// ──────────────────────────────────────────────────────────────────
fn skill_descriptors() -> Vec<RuleDescriptor> {
    use crate::security::skill_scanner::{RuleKind as SkRuleKind, RULES};
    RULES
        .iter()
        .map(|r| {
            let action = match r.kind {
                SkRuleKind::HardTrigger => DefaultAction::HardBlock,
                SkRuleKind::Weighted => DefaultAction::Weighted,
            };
            RuleDescriptor {
                kind: RuleKind::Skill,
                id: r.id.to_string(),
                category: Some(format!("{:?}", r.kind).to_lowercase()),
                description: r.description.to_string(),
                default_action: action,
                pattern_hint: Some(r.needles.join(" / ")),
                remediation: None,
            }
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────
// security/checks/*.rs
// ──────────────────────────────────────────────────────────────────
fn audit_descriptors() -> Vec<RuleDescriptor> {
    use crate::security::scanner::all_checks;
    all_checks()
        .into_iter()
        .map(|check| {
            let category_str = format!("{:?}", check.category());
            RuleDescriptor {
                kind: RuleKind::Audit,
                id: check.id().to_string(),
                category: Some(category_str),
                description: check.description().to_string(),
                default_action: DefaultAction::Warn, // 扫描时按结果再决定
                pattern_hint: None,
                remediation: None,
            }
        })
        .collect()
}
